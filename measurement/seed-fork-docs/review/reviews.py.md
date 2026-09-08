# Code Review: `sepet/api/reviews.py`

## Purpose

This module implements restaurant reviews in the Sepet FastAPI backend. It owns the API route that creates a review and provides one helper used to combine a restaurant's seeded rating with actual review ratings. The router is mounted with the `/api` prefix (`reviews.py:10`), so its operational route is `POST /api/restaurants/{restaurant_id}/reviews`.

The route depends on `require_customer` from `auth.py` for authenticated customer-only access (`reviews.py:5`, `reviews.py:22`), on `get_db` for SQLite access and `utc_now` for timestamps (`reviews.py:6`), and on `ReviewInput` from `panels.py` for request validation (`reviews.py:7`, `reviews.py:21`). It also depends on database consistency in the `"order"` table: only a status exactly equal to `'onaylandı'` establishes eligibility (`reviews.py:32-34`). Downstream functionality, such as restaurant rating display or customer profile history, may consume rows written into `restaurant_review`, but that consumption is not in this file.

The module's intended behavior is deliberately narrow: an authenticated customer may leave one review for a restaurant after having at least one approved order there. It verifies comment content, restaurant existence, order eligibility, and one-review-per-customer uniqueness before inserting.

## Walkthrough

### `displayed_rating(seed_rating: float, review_rows) -> float` (`reviews.py:13`)

Parameters are a numeric `seed_rating` and an iterable of rows or mappings, each expected to have a `"rating"` key. It returns a float rounded to one decimal place.

Control flow:

1. Build a list beginning with `seed_rating`, followed by each `row["rating"]` (`reviews.py:14`).
2. Return the arithmetic mean rounded to one decimal (`reviews.py:15`).

There is no validation that values are numeric or non-null, and no protection against an empty collection. Its local name suggests it is intended for rows with a `rating` column fetched via `sqlite3.Row`.

### `create_review(restaurant_id: int, payload: ReviewInput, user=Depends(require_customer)) -> dict[str, Any]` (`reviews.py:18-50`)

FastAPI supplies the integer path parameter `restaurant_id`, the validated body `payload`, and the customer row from `require_customer`. On success it returns a dictionary containing the inserted review `id`, `rating`, cleaned `comment`, and `user_email` (`reviews.py:50`). It returns errors as HTTP exceptions.

Control flow:

1. Strip the comment and reject it as HTTP 400 when the result is empty (`reviews.py:24-25`).
2. Open a SQLite transaction context with `get_db()` (`reviews.py:26`).
3. Query the restaurant by parameterized exact `id`; return HTTP 404 when absent (`reviews.py:27-31`).
4. Query `"order"` for one row belonging to this customer and restaurant with status `'onaylandı'`; return HTTP 403 when none exists (`reviews.py:32-38`).
5. Query `restaurant_review` for an existing pair of this restaurant and user; return HTTP 409 when one already exists (`reviews.py:39-43`).
6. Insert the review with restaurant id, customer id, payload rating, stripped comment, and current UTC timestamp (`reviews.py:44-48`).
7. Leave the `with get_db()` context, which commits on successful exit.
8. Return the inserted row id plus fields derived from the request and user (`reviews.py:50`).

The check-then-insert pattern is not guarded by an explicit unique constraint in this module. If one exists in schema definitions outside this file, concurrent requests may still be safe at the database level; without it, a race can create two reviews.

## Failure Modes

1. **Eligibility depends on a non-ASCII status literal.** The order query requires exactly `'onaylandı'` (`reviews.py:32-34`). If database seeding, status normalization, or another writer stores a different Unicode encoding or a misspelled variant, the customer is denied even though the order is approved.
2. **Any approved order qualifies, regardless of order item or current ownership.** The query filters only `customer_id`, `restaurant_id`, and status (`reviews.py:33-35`). This is likely intended, but if order records can be reassigned or if deleted/cancelled orders later become approved, eligibility may not match product expectations.
3. **Duplicate-review race.** The route first selects for an existing review, then inserts (`reviews.py:39-48`). Two simultaneous requests by the same customer can both observe no existing row and both insert unless a database unique constraint blocks them. Without such a constraint, the customer can end up with two reviews.
4. **No explicit handling of database integrity errors.** If `restaurant_review` has a uniqueness constraint, the second concurrent request may raise `sqlite3.IntegrityError` and surface as an HTTP 500 rather than the intended HTTP 409 (`reviews.py:44-48`). If no constraint exists, it silently creates a duplicate.
5. **Rating bounds come only from `ReviewInput`.** This module inserts `payload.rating` directly (`reviews.py:48`). If `ReviewInput` does not constrain the range and type robustly, invalid ratings can enter the table and distort averages.
6. **`displayed_rating` can produce invalid or surprising results.** It accepts arbitrary `review_rows`; null ratings make the result null, non-numeric values can raise, and a call with no rows produces `ZeroDivisionError` (`reviews.py:13-15`). Seed ratings outside the user-facing range also flow straight through.
7. **Response uses `cursor.lastrowid` after the connection context ends.** This normally works for SQLite, but it exposes an implementation detail and assumes the insert is on the last statement's cursor (`reviews.py:44`, `reviews.py:50`). If the function changes to execute another statement before returning, the wrong id could be returned.
8. **Comment length is not checked here.** Only non-whitespace is enforced (`reviews.py:24-25`). Any maximum length must be enforced by `ReviewInput`, database validation, or UI, otherwise very large comments may be stored.
9. **Deleting a restaurant or user may leave review history inconsistent.** The insert stores `restaurant_id` and `user_id` (`reviews.py:45-48`). Referential behavior and cascading rules live outside this module; without enforced foreign keys, related deletions can produce display anomalies or orphan rows.
10. **Errors between insert and response are not explicitly rolled back.** The `with get_db()` context is expected to manage transaction boundaries (`reviews.py:26`), so this depends on `get_db()` and FastAPI exception semantics. If the context does not commit/rollback as assumed, data loss or partial writes become possible.

## Security

- **Authentication:** `user=Depends(require_customer)` requires a valid database-backed token and a customer role before the route body runs (`reviews.py:22`; implementation in `auth.py:23-36`).
- **Authorization:** The route checks that the customer has an approved order for the target restaurant (`reviews.py:32-38`). It prevents reviewing an arbitrary restaurant without purchase history.
- **Identity binding:** The review is inserted with `user["id"]`, not with a client-supplied user id (`reviews.py:48`), preventing direct user spoofing in the body.
- **SQL injection:** All SQL queries use parameterized placeholders: restaurant lookup (`reviews.py:27-29`), order eligibility (`reviews.py:32-35`), duplicate check (`reviews.py:39-41`), and insert (`reviews.py:44-48`). Restaurant id comes from a typed path parameter.
- **Input exposure:** The response includes the caller's own `user_email` (`reviews.py:50`). If the frontend later displays this or a query exposes it publicly, review anonymity requirements should be revisited; this module currently exposes it only to the creator.
- **Data integrity:** Eligibility and uniqueness are application-level checks, not necessarily database constraints (`reviews.py:32-48`). SQL injection is mitigated, but concurrency and consistency are not fully controlled here.
- **Data cleanup:** There is no route to update or delete reviews. A customer who creates one cannot modify it, and abusive content removal must occur outside this API.

## Test Checklist

1. **Anonymous create.** Setup: seeded restaurant. Action: `POST /api/restaurants/{id}/reviews` without Authorization. Expected: HTTP 401 and no row inserted.
2. **Restaurant user.** Setup: valid restaurant token. Action: call the review endpoint. Expected: HTTP 403 and no row inserted.
3. **Courier user.** Setup: valid courier token. Action: call the review endpoint. Expected: HTTP 403.
4. **Unknown restaurant.** Setup: valid customer token and nonexistent id. Action: post a valid review. Expected: HTTP 404 and no row inserted.
5. **No approved order.** Setup: customer token and restaurant with no matching approved order. Action: post a valid review. Expected: HTTP 403 and no row inserted.
6. **Approved order.** Setup: customer has an order for the restaurant with status exactly `'onaylandı'`. Action: post rating and nonempty comment. Expected: HTTP 201; response id, rating, comment, and email are correct; database row has matching values and UTC timestamp.
7. **Whitespace-only comment.** Setup: eligible customer. Action: send `"   \n\t  "`. Expected: HTTP 400 and no row inserted.
8. **Nonempty comment is trimmed.** Setup: eligible customer. Action: send `"  cok iyi  "`. Expected: HTTP 201 and database stores `"cok iyi"`.
9. **Second review same restaurant.** Setup: eligible customer who already reviewed that restaurant. Action: post another valid review. Expected: HTTP 409 and still one row for the pair.
10. **Duplicate review under concurrency.** Setup: fresh eligible customer; send two simultaneous identical requests. Action: await both responses. Expected: one HTTP 201 and one HTTP 409 (or a safe mapped conflict), exactly one row.
11. **Reviews for different restaurants.** Setup: customer has approved orders at two restaurants. Action: create a review for each. Expected: both HTTP 201.
12. **Rating boundaries.** Setup: eligible customer. Action: send minimum and maximum ratings allowed by `ReviewInput`. Expected: accepted within bounds; database values match.
13. **Invalid rating.** Setup: eligible customer. Action: send a rating outside `ReviewInput` constraints or a non-numeric value. Expected: FastAPI validation error, not an inserted row.
14. **Approved-order status encoding.** Setup: order status visually similar but stored differently from `'onaylandı'`. Action: attempt review. Expected: HTTP 403; test Unicode code points to verify the exact backend issue.
15. **`displayed_rating` normal case.** Setup: seed rating 4.0 and review rows 3.0, 4.0. Action: call helper. Expected: `3.7`.
16. **`displayed_rating` empty rows.** Setup: no review rows. Action: call helper with any seed. Expected: either clearly documented seed return or a handled error; currently expect `ZeroDivisionError`, so a test should capture the intended remediation.
