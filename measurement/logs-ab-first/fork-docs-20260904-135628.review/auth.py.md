# Review: `sepet/api/auth.py`

## 1. Purpose

This module is the shared authentication and role-authorization layer for the FastAPI backend. It converts an HTTP `Authorization` header into a database-backed user record, then exposes reusable FastAPI dependencies for routes that require a user, optionally accept a user, or require the user to have a specific role.

The token lookup joins `auth_token` to the `user` table and returns the user's id, email, role, and balance (`auth.py:10-15`). That result is the identity object used elsewhere in the API. Routes depending on `get_current_user` receive an authenticated user; routes depending on `require_customer` are limited to customers; routes depending on `require_restaurant` or `require_courier` additionally receive the corresponding business profile. The module therefore has a central blast radius: a defect in token parsing or role checks can affect all protected endpoints.

## 2. Walkthrough

### `_user_from_token(conn, token: str | None)` - `auth.py:6-15`

- Parameters: an open SQLite connection and the raw authorization header value or `None`.
- Returns: a `sqlite3.Row` containing user `id`, `email`, `role`, and `balance`, or `None` when no token is supplied, the header is malformed, or the token does not match a database row.
- Control flow: `auth.py:7-8` rejects a missing header or one not beginning with `"Bearer "`. `auth.py:9` strips that prefix. `auth.py:10-15` performs a parameterized query joining `auth_token` and `user`, filters by the token value, and returns the first row with `fetchone()`.
- There is no expiry comparison in this function. Any token string in the database remains valid, as far as this module is concerned.

### `resolve_user(authorization: str | None = Header(default=None))` - `auth.py:18-20`

- Parameters: FastAPI injects the optional `Authorization` header.
- Returns: the same user row or `None` result produced by `_user_from_token`.
- Control flow: opens a new database connection with `get_db()` (`auth.py:19`), resolves the header, and closes the connection when the context manager exits. It deliberately does not raise for missing or invalid credentials; it returns `None`.

### `get_current_user(user=Depends(resolve_user))` - `auth.py:23-26`

- Parameters: the resolved user row injected by `resolve_user`.
- Returns: the user row when authentication succeeded.
- Control flow: if the injected value is falsy (`auth.py:24`), it raises HTTP 401 with the message `Giris yapmalisiniz` (`auth.py:25`). Otherwise it returns the row.

### `get_optional_user(user=Depends(resolve_user))` - `auth.py:29-30`

- Parameters: the resolved user row injected by `resolve_user`.
- Returns: either a user row or `None`.
- Control flow: no branching; it simply passes the resolved value to the route. Routes using this dependency must themselves handle both cases.

### `require_customer(user=Depends(get_current_user))` - `auth.py:33-36`

- Parameters: an already-authenticated user produced by `get_current_user`.
- Returns: the authenticated user row if its role is exactly `"customer"`.
- Control flow: `auth.py:34` compares the role. A non-customer receives HTTP 403 (`auth.py:35`); a customer is returned unchanged.

### `require_restaurant(user=Depends(get_current_user))` - `auth.py:39-48`

- Parameters: an already-authenticated user produced by `get_current_user`.
- Returns: a tuple `(user, restaurant)`, where the second element is the complete restaurant profile row for that user.
- Control flow: `auth.py:40-41` rejects a user whose role is not `"restaurant"` with HTTP 403. It then opens a connection and looks up `restaurant` by `user_id` (`auth.py:42-45`). If there is no profile, it raises HTTP 403 (`auth.py:46-47`). Otherwise it returns both objects. This means a valid restaurant login without a corresponding profile cannot operate as a restaurant.

### `require_courier(user=Depends(get_current_user))` - `auth.py:51-60`

- Parameters: an already-authenticated user produced by `get_current_user`.
- Returns: a tuple `(user, courier)`, where the second element is the complete courier profile row for that user.
- Control flow: `auth.py:52-53` rejects non-courier users with HTTP 403. `auth.py:54-57` looks up `courier` by `user_id`. A role-valid user without a profile is rejected with HTTP 403 (`auth.py:58-59`). Otherwise both objects are returned (`auth.py:60`).

## 3. Failure modes

- Missing or malformed credentials: `_user_from_token` returns `None` for `None`, an empty header, or any header not starting with `"Bearer "` (`auth.py:7-8`). This is intentional for optional users but becomes 401 through `get_current_user`.
- Token typo or deleted token: the exact-match query returns no row (`auth.py:10-15`). Public APIs see an anonymous user; protected APIs see 401.
- Expired tokens accepted: the query checks only token equality, never an expiry column or timestamp (`auth.py:10-15`). A stale token remains usable forever unless it is deleted.
- Logout does not reliably invalidate: because tokens are long-lived, deletion behavior belongs to the logout handler; this module will continue to accept any row still present in `auth_token` (`auth.py:10-15`).
- Case sensitivity: only exactly `Bearer ` is accepted (`auth.py:7`). A client sending `bearer` is treated as unauthenticated.
- Multiple token rows: if schema or seed data permits the same token string to occur more than once, `fetchone()` picks one row (`auth.py:15`), potentially associating the request with an unintended user.
- Role string drift: role checks use exact equality against `customer`, `restaurant`, and `courier` (`auth.py:34`, `auth.py:40`, `auth.py:52`). A schema or seed change that stores a different case or Unicode variant silently receives 403.
- Missing profiles: restaurant and courier users without matching profile rows get HTTP 403 (`auth.py:46-47`, `auth.py:58-59`). New role-bearing users cannot act until profile creation is complete.
- Balance snapshot: the resolver selects `balance` (`auth.py:11-15`). Any mutation after resolution within a request is not reflected in this row.
- Database context: `resolve_user` opens and closes its own connection (`auth.py:19`). This is isolated from route-level transactions, so token deletion and subsequent identity resolution cannot be made atomic through this helper alone.
- UTF-8 detail: the Turkish role data is not manipulated here, but messages and role literals are plain source strings; a database collation mismatch is not guarded by normalization.

## 4. Security

- Authentication is header-based: `_user_from_token` accepts only a `Bearer`-prefixed value (`auth.py:7-9`) and maps it to the user through `auth_token`.
- Token lookup is parameterized: the token value is passed as a bound parameter (`auth.py:13-14`), so this module does not expose direct header-to-SQL injection.
- Invalid authentication is denied: `get_current_user` raises 401 for any falsy resolution result (`auth.py:24-25`).
- Authorization is role-based: customer, restaurant, and courier operations require exact role matches (`auth.py:33-36`, `auth.py:39-48`, `auth.py:51-60`).
- Profile authorization exists: role alone is insufficient for restaurant and courier actions; a matching profile row must exist (`auth.py:42-47`, `auth.py:54-59`).
- Data exposure risk: the resolver returns the full user row fields it selected, including `balance` and `email` (`auth.py:11-15`). Any route that blindly serializes the identity object may expose data not intended for public responses.
- Sensitive token handling: the module does not log or echo the token, but an excessively long token is still transmitted to SQLite before failing to match (`auth.py:10-15`); no input size limit is enforced here.
- No transport concern is enforceable locally: the module cannot guarantee that the Bearer token arrived over HTTPS.
- Database exposure: `SELECT * FROM restaurant` and `SELECT * FROM courier` (`auth.py:43-44`, `auth.py:55-56`) give route handlers every profile column. If either table later gains sensitive fields, route responses can leak them by accident.

## 5. Test checklist

1. Setup: database has one valid user and token. Action: call a protected route with `Authorization: Bearer <valid-token>`. Expected: HTTP 200 and the endpoint sees the correct user id, email, role, and balance.
2. Setup: protected route only. Action: omit `Authorization`. Expected: HTTP 401 with the configured Turkish detail.
3. Setup: valid token. Action: send `Authorization: <token>` without `Bearer `. Expected: HTTP 401 on protected routes; optional-user route sees no user.
4. Setup: valid token with lowercase `bearer `. Action: call protected route. Expected: HTTP 401 because the prefix comparison is case-sensitive.
5. Setup: no token row matching the supplied value. Action: send `Authorization: Bearer unknown`. Expected: HTTP 401; no user identity is materialized.
6. Setup: delete or alter the token row before a request. Action: retry with the old header. Expected: HTTP 401, proving lookup depends on current database state.
7. Setup: optional endpoint requiring `get_optional_user`. Action: call without credentials. Expected: HTTP success and endpoint receives `None`.
8. Setup: customer, restaurant, and courier users. Action: call each of the three role dependencies with the wrong role. Expected: HTTP 403 for each mismatch.
9. Setup: restaurant-role user without a `restaurant` profile row. Action: call a `require_restaurant` endpoint. Expected: HTTP 403 and no route logic executes.
10. Setup: courier-role user without a `courier` profile row. Action: call a `require_courier` endpoint. Expected: HTTP 403 and no route logic executes.
11. Setup: customer-role user. Action: call a customer dependency. Expected: HTTP 200 and route receives the same user row.
12. Setup: two API calls with the same valid token. Action: use it immediately in sequence. Expected: both calls resolve the same identity; if token revocation is implemented, the removed token call becomes 401.
13. Setup: valid restaurant and courier profiles. Action: call role endpoints. Expected: route receives both user and profile objects without leaking profile fields unless explicitly selected.