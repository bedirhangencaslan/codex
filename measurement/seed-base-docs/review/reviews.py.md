# Code Review: `sepet/api/reviews.py`

## 1. Purpose

This module owns the customer restaurant-review API and one reusable rating calculation. It defines an APIRouter under `/api` and registers the endpoint `POST /api/restaurants/{restaurant_id}/reviews`. The route enforces customer authentication, verifies that the target restaurant exists, requires an approved order between that customer and restaurant, prevents duplicate reviews, and inserts the review. The `displayed_rating` helper combines a restaurant's seeded base rating with actual review ratings and is imported by the main module for list and detail rendering.

Dependencies are concentrated and explicit: `require_customer` supplies an authenticated customer row, `get_db` supplies database access, `utc_now` supplies creation timestamps, and `ReviewInput` validates the request body. The database schema complements this module by enforcing a rating range of 1 through 5 and uniqueness of `(restaurant_id, user_id)`.

## 2. Walkthrough

### Module setup — lines 1-10

The imports establish dependency wiring. `Any` is used for the response annotation (`reviews.py:1`); FastAPI objects and dependencies are imported at line 3; the customer gate, database helper, timestamp helper, and request model are imported at lines 5-7. Line 10 creates `router = APIRouter(prefix="/api")`, so all decorators in the module add paths under `/api`.

### `displayed_rating(seed_rating: float, review_rows) -> float` — lines 13-15

Parameters:

- `seed_rating`: the restaurant's base rating before real reviews.
- `review_rows`: an iterable of SQLite-like rows or mappings with a `rating` field.

Returns the arithmetic mean of the seed rating and every review rating, rounded to one decimal place.

Control flow:

1. Line 14 builds a list beginning with `seed_rating`, followed by each `row["rating"]`.
2. Line 15 computes `sum(ratings) / len(ratings)` and rounds it to one decimal.

If `review_rows` is empty, the list contains only the seed and the returned value is the rounded seed. If any row lacks `rating`, indexing raises `KeyError`. If `review_rows` is `None`, iteration raises `TypeError`.

### `POST /api/restaurants/{restaurant_id}/reviews` via `create_review(restaurant_id, payload, user)` — lines 18-50

Parameters:

- `restaurant_id`: integer path parameter identifying the restaurant.
- `payload`: JSON body parsed as `ReviewInput`, containing `rating: int` constrained to 1-5 and `comment: str`.
- `user`: authenticated customer row injected by `require_customer`.

Returns a dictionary with the new review `id`, submitted `rating`, trimmed `comment`, and the customer's `user_email`. The route declares HTTP status 201.

Control flow:

1. Line 24 strips the comment and checks whether it is empty. If so, lines 25 raises HTTP 400 `"Yorum metni zorunludur"`.
2. Line 26 opens the database connection in a `with` block.
3. Lines 27-29 query `restaurant.id` by the supplied ID.
4. Lines 30-31 raise HTTP 404 `"Restoran bulunamadi"` when no restaurant row is found.
5. Lines 32-36 query for an `"order"` row where the current customer and restaurant match and status is exactly `'onaylandı'`.
6. Lines 37-38 raise HTTP 403 when there is no approved order.
7. Lines 39-42 query for an existing `restaurant_review` with the same restaurant and user.
8. Lines 43 raise HTTP 409 `"Bu restorana yorum yaptiniz"` if one exists.
9. Lines 44-49 insert the review with restaurant ID, user ID, submitted rating, trimmed comment, and `utc_now()`.
10. Line 50 returns the inserted `lastrowid`, original submitted rating, trimmed comment, and user email after the connection context exits.

Pydantic validates `rating` before the function body runs, returning a FastAPI validation response for out-of-range values. The schema's `CHECK` and `UNIQUE` constraints provide database-level backstops.

## 3. Failure modes

1. **Exact status literal creates a fragile contract.** The approved-order query requires exactly `'onaylandı'` at `reviews.py:34`. If the order was stored with a different Unicode sequence, different casing, an alternate label, or encoding corruption, the same delivered order is treated as ineligible and review creation returns 403 at `reviews.py:37-38`.
2. **No transaction boundary around eligibility and insert.** Lines 32-48 perform checks and insertion on one connection but do not begin an explicit transaction or re-check uniqueness atomically. Concurrent duplicate requests can race; one request may pass the check before another commits. The schema's unique constraint then turns the second insert into an unhandled constraint error at `reviews.py:44-49`.
3. **Constraint errors become HTTP 500.** Duplicate review insertion can raise `sqlite3.IntegrityError` instead of the intended HTTP 409 if it slips through the precheck at `reviews.py:39-43`. Likewise, a schema mismatch or database corruption surfaces uncontrolled.
4. **`cursor.lastrowid` is accessed outside the connection context.** The insert is executed inside `with` (`reviews.py:44-49`), while `cursor.lastrowid` is read after the context exits at `reviews.py:50`. For sqlite3's context manager this typically remains usable, but exposing the cursor past its database block is brittle and depends on implementation details.
5. **Whitespace-only comments are rejected, but whitespace is otherwise lost from returned and stored text consistently.** A one-character space fails at `reviews.py:24-25`; multi-space comments are trimmed at line 48 and line 50. Callers expecting the exact submitted string may be surprised, though this is likely intentional.
6. **Comment length is not limited in this endpoint.** `ReviewInput.comment` is a plain string (`reviews.py:7`, schema at `sepet/api/schema.sql:39`). Extremely large comments can be accepted and stored subject to database limits, affecting payload size and UI rendering.
7. **Any approved historical order qualifies.** The query at `reviews.py:32-36` does not filter by delivery confirmation, refund state, time window, or order item relevance. If business policy changes, this may permit reviews from orders users consider invalid.
8. **Non-integer restaurant path inputs are rejected by FastAPI routing/validation**, not by the function itself (`reviews.py:19-20`). Negative integers are accepted by the route and then simply find no restaurant at `reviews.py:27-31`.
9. **Rating is redundant between layers.** Pydantic constrains it to 1-5 (`sepet/api/panels.py:31`) and the database also enforces 1-5 (`sepet/api/schema.sql:38`). Good defense in depth, but drift between the two can create confusing validation behavior.
10. **`displayed_rating` trusts caller-provided row shape.** It assumes every element is subscriptable and has `rating` (`reviews.py:14`); malformed data causes an exception in any importing endpoint.
11. **Average weighting is implicit.** Every review and the seed count equally (`reviews.py:13-15`). If product policy wants a minimum review count or seed weighting, this helper silently does not implement it.
12. **No explicit ownership leak guard for `user_email`.** The response exposes the reviewer's email to the creator (`reviews.py:50`). If this return value is later cached or logged broadly, it may expose account information; detailed public review rendering must avoid it.

## 4. Security

- **Authentication:** `require_customer` is a route dependency at `reviews.py:22`, chaining through `get_current_user`. Anonymous or non-customer requests cannot reach the body; failures originate as 401/403 in `auth.py`.
- **Authorization:** The route is customer-only, but also requires the approved order to link `user["id"]` and `restaurant_id` at `reviews.py:32-38`. Thus a customer cannot review an unrelated restaurant. It does not need restaurant-owner authorization because creation is intended for customers.
- **Injection:** Every SQL statement uses bound parameters: restaurant lookup at `reviews.py:27-29`, order eligibility at `reviews.py:32-36`, duplicate check at `reviews.py:39-42`, and insert at `reviews.py:44-49`. The comment is stored as text, not concatenated into SQL.
- **Input validation:** `ReviewInput` constrains `rating` to integer 1-5 (`reviews.py:7`, `sepet/api/panels.py:31-32`), and the schema has a matching `CHECK` (`sepet/api/schema.sql:38`). Comments are trimmed and required to be nonempty (`reviews.py:24`, `reviews.py:48`).
- **Data exposure:** The response exposes `user_email` from the authenticated row at `reviews.py:50`; since this goes to the authenticated customer, direct exposure is acceptable. Public endpoints elsewhere must select review fields explicitly and should not blindly serialize joined user data.
- **Abuse surface:** Duplicate prevention exists both in code (`reviews.py:39-43`) and schema (`sepet/api/schema.sql:41`). However, the code has no rate limiting, comment length limit, or content moderation, so spam and oversized payloads remain possible for an authenticated customer with an approved order.

## 5. Test checklist

1. **Anonymous review.** Setup: seed a restaurant and approved order, but send no `Authorization` header. Action: POST a valid rating and comment. Expected: HTTP 401.
2. **Restaurant user attempts review.** Setup: create a restaurant-role token. Action: POST to the review endpoint. Expected: HTTP 403 from the customer gate.
3. **Courier user attempts review.** Setup: create a courier-role token with courier profile. Action: POST. Expected: HTTP 403.
4. **Unknown restaurant.** Setup: authenticated customer with no eligible order. Action: POST to a nonexistent restaurant ID. Expected: HTTP 404.
5. **No approved order.** Setup: authenticated customer, existing restaurant, no order. Action: POST rating 5 and a nonempty comment. Expected: HTTP 403.
6. **Unapproved order only.** Setup: customer has an order for the restaurant with a status other than exactly `'onaylandı'`. Action: POST. Expected: HTTP 403.
7. **Approved order succeeds.** Setup: customer and restaurant with one approved order. Action: POST rating 4 and `"Harika hizmet"`. Expected: HTTP 201, body has nonzero ID, rating 4, trimmed comment, and correct email.
8. **First-time rating validation.** Setup: approved order. Action: POST rating 0, then rating 6. Expected: each receives FastAPI validation failure, not a 201 or database error.
9. **Boundary ratings.** Setup: approved order. Action: POST rating 1, then in a separate database/state test rating 5. Expected: both accepted where no duplicate exists.
10. **Empty comment.** Setup: approved order. Action: POST `"   "` as the comment. Expected: HTTP 400.
11. **Trimming.** Setup: approved order. Action: POST `"  iyi  "`. Expected: stored and returned comment is `"iyi"`.
12. **Duplicate review.** Setup: customer has an approved order and has already reviewed the restaurant. Action: POST again. Expected: HTTP 409; database still contains one review.
13. **Concurrent duplicate.** Setup: approved order, two identical requests submitted concurrently. Action: issue both POSTs. Expected: one succeeds and the other fails cleanly; if it becomes 500 due to a race, that is a defect to fix with constraint handling or serialization.
14. **Review affects displayed rating.** Setup: restaurant seed rating 5.0; add reviews with ratings 1 and 5. Action: call the rating helper with those rows. Expected: result 3.7 (`(5 + 1 + 5) / 3`, rounded).
15. **Empty review rows.** Setup: no review rows. Action: call `displayed_rating(4.2, [])`. Expected: 4.2 returned.
16. **Malformed review rows.** Setup: rows lacking `rating` or `None` as the iterable. Action: call `displayed_rating`. Expected: controlled error handling at the caller; current implementation raises.
17. **Comment payload size.** Setup: approved order. Action: POST a very large comment. Expected: enforce the product-defined maximum; if no maximum is intended, verify storage and frontend behavior remain safe.
18. **Unicode content.** Setup: approved order. Action: POST a comment containing Turkish and emoji characters. Expected: stored exactly after trimming and returned without mojibake.