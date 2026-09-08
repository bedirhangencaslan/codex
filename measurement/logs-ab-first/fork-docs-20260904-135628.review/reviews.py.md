# Review: `sepet/api/reviews.py`

## 1. Purpose

This module implements the restaurant review feature at `POST /api/restaurants/{restaurant_id}/reviews`. It combines authentication, business rules, and persistence in one route handler.

The router is mounted with the `/api` prefix (`reviews.py:10`). The route requires a customer identity through `require_customer` (`reviews.py:22`), verifies that the restaurant exists and that this customer has an approved order from it, prevents duplicate reviews, then inserts a row in `restaurant_review` and returns a small response object. It also exposes `displayed_rating`, a helper for averaging a restaurant's seeded rating with approved review rows (`reviews.py:13-15`).

Because the response includes the reviewer's email and the route directly accesses database tables, this file is both a user-facing workflow and a data-exposure boundary. Its transactional and status-literal assumptions also connect it to order-state handling elsewhere in the backend.

## 2. Walkthrough

### `displayed_rating(seed_rating: float, review_rows) -> float` - `reviews.py:13-15`

- Parameters: `seed_rating`, a numeric initial rating; `review_rows`, any iterable of objects supporting row indexing where `row["rating"]` is numeric.
- Returns: the arithmetic mean of `seed_rating` and every review rating, rounded to one decimal place.
- Control flow: `reviews.py:14` builds one list starting with `seed_rating`, followed by each review's `rating` value. `reviews.py:15` sums that list, divides by its length, and applies Python's `round(..., 1)`.
- The function trusts both inputs to be non-empty (the initial seed guarantees length at least one) and numeric. It does not filter status, visibility, deleted state, or rating bounds; those responsibilities belong to the caller.

### `POST /api/restaurants/{restaurant_id}/reviews` route: `create_review(...)` - `reviews.py:18-50`

- Decorator and path: registers `POST` under `/api/restaurants/{restaurant_id}/reviews` and sets success status to 201 (`reviews.py:18`).
- Parameters: `restaurant_id` is the integer path parameter; `payload` is a parsed `ReviewInput` body; `user` is injected by `require_customer`.
- Returns: on success, a dictionary containing the new review `id`, submitted `rating`, trimmed `comment`, and `user_email` (`reviews.py:50`).
- Errors: HTTP 400 for a blank comment, HTTP 404 for a nonexistent restaurant, HTTP 403 when no approved order exists, HTTP 409 for a duplicate review.
- Control flow:
  1. `payload.comment.strip()` is checked first (`reviews.py:24`). If it becomes empty, the route raises HTTP 400 (`reviews.py:25`).
  2. A database connection is opened (`reviews.py:26`), and the route checks that the restaurant id exists (`reviews.py:27-29`).
  3. A missing restaurant produces HTTP 404 (`reviews.py:30-31`).
  4. The route queries `"order"` for at least one row where `customer_id` equals the current user, `restaurant_id` equals the path value, and `status` is the literal `onaylandı` (`reviews.py:32-36`).
  5. If there is no matching approved order, it raises HTTP 403 (`reviews.py:37-38`).
  6. It checks whether the same user already has a row in `restaurant_review` for this restaurant (`reviews.py:39-42`) and raises HTTP 409 if so (`reviews.py:43`).
  7. It inserts the restaurant id, user id, submitted rating, trimmed comment, and current UTC timestamp (`reviews.py:44-49`).
  8. The context manager commits the insert when it exits. The route then builds a response from `cursor.lastrowid` and values taken from the request/user (`reviews.py:50`).

Important sequencing details: the existence, eligibility, and duplicate checks plus the insert are all in the same connection scope (`reviews.py:26-49`), but the route does not explicitly lock the relevant rows or create a unique constraint at this layer. Under concurrent submissions, duplicate prevention is not guaranteed unless the database schema enforces uniqueness.

## 3. Failure modes

- Blank comments are rejected, but whitespace-only and very long comments are different concerns: `reviews.py:24-25` only tests emptiness after trimming. Any maximum length must be enforced by `ReviewInput` validation or schema constraints; this route does not.
- A nonexistent restaurant is returned as 404 (`reviews.py:27-31`). That is reasonable, but it also reveals to an authenticated customer that the id does not exist before order eligibility is evaluated.
- Approved-order eligibility depends on a status literal: `status = 'onaylandı'` is embedded here (`reviews.py:34`). If schema, seed data, or other backend modules use a different Unicode representation or casing, eligible customers receive HTTP 403 and cannot review.
- One approved order grants a review across all past orders from that restaurant: the query uses `SELECT 1` (`reviews.py:32-36`), not a specific order id, and no per-order review constraint is visible here.
- Duplicate check is not atomic: the route checks and then inserts (`reviews.py:39-49`). Two simultaneous requests can both pass the check and insert unless the table has a database-level `(restaurant_id, user_id)` unique constraint.
- Foreign-key integrity depends on connection configuration: the route relies on the insert succeeding (`reviews.py:44-49`). If SQLite foreign keys are off in `get_db()`, malformed caller data or race conditions could persist inconsistent rows.
- `cursor.lastrowid` is used outside the transaction context (`reviews.py:50`). For normal single-row SQLite inserts this is valid, but the cursor and connection remain open at response construction, and this pattern would not work for a bulk or different insert backend.
- Rating limits are not checked in this file: `payload.rating` is stored directly (`reviews.py:48`). If `ReviewInput` does not constrain it, values such as negative numbers, zero, huge floats, or non-standard numeric types could corrupt averages.
- `displayed_rating` has no visibility filter (`reviews.py:13-15`). If callers pass every review row, hidden, pending, or deleted reviews could alter public ratings.
- Returned email may be unnecessary: the success response always includes `user_email` (`reviews.py:50`). For public display, this can expose personal contact data; for the author, it may be redundant.
- No explicit pagination or denial-of-service limit applies to review creation beyond authentication. A customer can create one review per eligible restaurant (`reviews.py:39-43`), but an account with many approved orders can accumulate reviews across restaurants.
- Error message and status choice: having any approved history is enough, so deleting a specific order later does not revoke review rights unless separate cleanup logic updates reviews.
- Unicode status risk can also cause operational stalls after re-seeding because status values are persisted strings (`reviews.py:34`).

## 4. Security

- Authentication: `user=Depends(require_customer)` (`reviews.py:22`) ensures an unauthenticated or non-customer request cannot reach route logic. The dependency itself raises 401/403.
- Authorization: the route restricts review creation to customers (`reviews.py:22`) and then restricts eligibility to reviews for restaurants from which that same user has an approved order (`reviews.py:32-38`). It does not let a restaurant or courier create reviews.
- Ownership: the current user id is taken from the server-resolved identity, not from the request body, and both the order check and inserted `user_id` use it (`reviews.py:33-35`, `reviews.py:48`).
- SQL injection: all values are bound parameters (`reviews.py:28`, `reviews.py:35`, `reviews.py:41`, `reviews.py:47-48`). No user input is concatenated into SQL.
- Enumeration exposure: a missing restaurant is distinguished as 404 before order eligibility (`reviews.py:30-31`), allowing an authenticated customer to probe restaurant ids. Given restaurant listings are likely public, this is usually low risk, but it is a deliberate observable distinction.
- Data exposure: `user_email` is returned by the creation response (`reviews.py:50`). Since the endpoint is customer-protected, this most directly exposes the caller's own email; if other users can call it for an existing review id, this response does not support that path. Any future list endpoint must not copy this field blindly.
- Integrity: duplicate prevention is application-level unless schema constraints exist (`reviews.py:39-49`). Race conditions can bypass business rules.
- Status authorization: a customer's right to review is established by order history (`reviews.py:32-36`). If status transitions are inconsistently encoded elsewhere, authorization can become either too permissive or too restrictive.
- Input validation: comment emptiness is checked (`reviews.py:24-25`), but the route depends entirely on `ReviewInput` and database schema for rating range, comment length, and encoding rules.
- Auditability: `created_at` is generated server-side with `utc_now()` (`reviews.py:48`), reducing reliance on client-supplied timestamps.

## 5. Test checklist

1. Setup: valid customer token and existing restaurant. Action: submit rating `5` and comment `"Test"`. Expected: HTTP 201, response id, rating, trimmed comment, and caller email match; database row exists.
2. Setup: valid token. Action: submit the request without credentials. Expected: HTTP 401.
3. Setup: restaurant user token. Action: call the same endpoint. Expected: HTTP 403 from `require_customer`.
4. Setup: courier user token. Action: call the same endpoint. Expected: HTTP 403.
5. Setup: valid customer token. Action: submit comment containing only spaces. Expected: HTTP 400 and no database row.
6. Setup: valid customer token. Action: use a restaurant id with no row. Expected: HTTP 404 and no review row.
7. Setup: customer has no order from the restaurant. Action: submit a valid review. Expected: HTTP 403 and no review row.
8. Setup: customer has an order whose status is not the exact approved literal. Action: submit a review. Expected: HTTP 403; then normalize all status literals in the database and backend, reseed/restart, and verify eligible orders now permit review.
9. Setup: customer has a delivered/approved order from restaurant A but not restaurant B. Action: attempt reviews against both. Expected: restaurant A succeeds, restaurant B returns HTTP 403.
10. Setup: customer already has a review for the restaurant. Action: submit another valid review. Expected: HTTP 409 and only one row remains.
11. Setup: same customer submits two concurrent first-time reviews with no schema unique constraint. Action: send both requests simultaneously. Expected: current implementation may create duplicates; adding a database unique constraint must make the loser fail with HTTP 409 or an `IntegrityError` mapped cleanly to 409.
12. Setup: `ReviewInput` permits ratings 1 through 5. Action: submit ratings `1`, `5`, and outside-range values if validation permits. Expected: boundary values persist; invalid values are rejected by validation before route logic.
13. Setup: call `displayed_rating(4.0, [])`. Action: verify result. Expected: `4.0`.
14. Setup: seed rating `3.5`, review rows `4.0`, `4.5`. Action: call `displayed_rating`. Expected: result is rounded to one decimal and matches `(3.5 + 4.0 + 4.5) / 3`.
15. Setup: review row for a hidden or pending review. Action: pass that row directly to `displayed_rating`. Expected: it affects the mean; the caller must explicitly filter rows if it should not.