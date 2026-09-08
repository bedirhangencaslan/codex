# Code Review: `sepet/api/auth.py`

## Purpose

This module is the backend authentication and role-gating layer for the Sepet FastAPI application. It converts the HTTP `Authorization` header into a database-backed user record, refuses requests when authentication is mandatory, and returns reusable FastAPI dependencies for customer, restaurant, and courier endpoints. In source order, it defines token lookup, optional and mandatory user resolution, and role-specific dependencies (`auth.py:6` through `auth.py:60`).

Other route modules depend on it directly: `reviews.py` imports `get_optional_user` and `require_customer` (`reviews.py:5`). The broader API is expected to use `get_current_user` for any authenticated endpoint, `require_customer` for customer-only operations, and `require_restaurant` or `require_courier` where the caller must also have a matching business profile. Because these dependencies return database rows, callers receive the user's id, email, role, and balance, and for business roles they receive the complete corresponding restaurant or courier row.

The module does not create, renew, revoke, hash, or list tokens. It only interprets a token that already exists in the `auth_token` table and joins that token to a user. Consequently, lifecycle guarantees such as expiry and logout invalidation must be evaluated in the token-issuing code, schema, and any logout route, not here. Within this file, the important security surface is token matching, role equality checks, and profile existence checks.

## Walkthrough

### `_user_from_token(conn, token: str | None) -> sqlite3.Row | None` (`auth.py:6`)

Parameters are an open SQLite connection and an optional header string. The function returns either a row representing a user or `None`.

Control flow:

1. If `token` is falsy or does not begin with `"Bearer "`, return `None` (`auth.py:7-8`).
2. Remove the exact `"Bearer "` prefix (`auth.py:9`).
3. Execute a parameterized query joining `auth_token` to `"user"` and selecting user `id`, `email`, `role`, and `balance`, filtered by exact token equality (`auth.py:10-15`).
4. Return the first matching row or `None` (`auth.py:15`).

### `resolve_user(authorization: str | None = Header(default=None)) -> sqlite3.Row | None` (`auth.py:18`)

The only parameter is FastAPI's `Authorization` header, defaulting to `None`. It returns the same user row or `None` as `_user_from_token`.

Control flow opens a database context with `get_db()` (`auth.py:19`) and delegates to `_user_from_token` (`auth.py:20`). There is no exception handling and no distinction between a malformed header, an unknown token, and a deleted user; all yield `None`.

### `get_current_user(user=Depends(resolve_user)) -> sqlite3.Row` (`auth.py:23`)

Its dependency-injected parameter is the result of `resolve_user`. It returns a nonempty user row, or raises.

Control flow is a single guard: if the resolved user is falsy, it raises HTTP 401 with the message `"Giris yapmalisiniz"` (`auth.py:24-25`); otherwise it returns the row (`auth.py:26`).

### `get_optional_user(user=Depends(resolve_user)) -> sqlite3.Row | None` (`auth.py:29`)

Its parameter is the dependency-injected optional user. It returns whatever `resolve_user` produced. Its body is only `return user` (`auth.py:30`), making anonymous requests valid for endpoints that expose different behavior when logged in.

### `require_customer(user=Depends(get_current_user)) -> sqlite3.Row` (`auth.py:33`)

Its parameter is a mandatory authenticated user. It returns that user only when the role is exactly `"customer"`.

Control flow compares `user["role"]` with `"customer"` (`auth.py:34`); on mismatch it raises HTTP 403 (`auth.py:35`), otherwise returns the user (`auth.py:36`). It performs no customer-profile lookup because the role is assumed sufficient.

### `require_restaurant(user=Depends(get_current_user)) -> tuple[sqlite3.Row, sqlite3.Row]` (`auth.py:39`)

Its parameter is a mandatory authenticated user. It returns `(user, restaurant)`.

Control flow:

1. Require `role == "restaurant"` and otherwise raise HTTP 403 (`auth.py:40-41`).
2. Open a second database connection and query the full `restaurant` row by `user_id` (`auth.py:42-45`).
3. If no restaurant exists, raise HTTP 403 (`auth.py:46-47`).
4. Return both rows (`auth.py:48`).

### `require_courier(user=Depends(get_current_user)) -> tuple[sqlite3.Row, sqlite3.Row]` (`auth.py:51`)

Its parameter is a mandatory authenticated user. It returns `(user, courier)`.

Control flow mirrors the restaurant dependency: enforce `role == "courier"` (`auth.py:52-53`), query the full `courier` row by `user_id` (`auth.py:54-57`), raise HTTP 403 when absent (`auth.py:58-59`), and return both rows (`auth.py:60`).

## Failure Modes

1. **Non-expiring and non-revocable acceptance.** `_user_from_token` accepts any token with an exact database match, with no expiry predicate or revocation status (`auth.py:10-15`). A leaked token remains usable until the row is manually removed or the issuing flow changes.
2. **Header parsing is narrower than common transports.** Only the exact prefix `"Bearer "` is accepted (`auth.py:7`, `auth.py:9`). `"bearer x"` fails; `token x` fails; a missing header and a bad scheme are both merely `None`, which is acceptable for optional auth but can make 401/403 behavior harder to distinguish.
3. **Token lookup does not ensure the joined user still exists independently.** The inner join means deletion of the user naturally makes the token unusable, but the module cannot distinguish an orphaned token from an unknown token (`auth.py:12-14`). If foreign keys are off, orphan token rows remain silently unusable rather than being cleaned up.
4. **Role checks assume the database role strings are consistent.** `require_customer`, `require_restaurant`, and `require_courier` use exact equality (`auth.py:34`, `auth.py:40`, `auth.py:52`). Any encoding or literal drift between schema seed data and this code turns valid users into 403 responses or, if a role is accidentally changed, grants the wrong class of access.
5. **Business profile lookup opens a second connection after user authentication.** `require_restaurant` reads the user on one connection through dependencies, then opens `get_db()` again for the profile (`auth.py:42-45`; similarly `auth.py:54-57`). This is not intrinsically wrong for read-only checks, but a transactional or recently mutated profile may be observed at a different point in time than the user row.
6. **Password and token type assumptions live elsewhere.** This file accepts whatever opaque string is stored in `auth_token.token` (`auth.py:10-15`). If token issuance does not store high-entropy random values, this module will still accept them.
7. **Header injection or duplicated Authorization semantics depend on FastAPI/framework normalization.** Since the module processes the framework-provided header as one string (`auth.py:18`), behavior with duplicate headers or unusual whitespace is outside this file's control and should be tested rather than assumed.
8. **Mandatory auth returns 401, while role and profile failures return 403.** The status split is logical, but callers that blanket-redirect on 403 may send authenticated customers to an invalid destination (`auth.py:25`, `auth.py:35`, `auth.py:41`, `auth.py:47`).

## Security

- **Authentication:** The module authenticates with an exact token lookup against the database (`auth.py:10-15`). It does not authenticate by email, id, or role claimed in the request body, which is a good boundary.
- **Authorization:** Customer access is restricted by exact role equality (`auth.py:34-35`). Restaurant and courier access additionally require a matching profile row (`auth.py:43-48`, `auth.py:55-60`), so merely changing a role value is not enough if no profile exists.
- **Token handling:** Bearer credentials are removed from the header and used only as a query parameter (`auth.py:9-14`). The query is parameterized, so SQL injection through the `Authorization` value is not expected here.
- **Data exposure:** Dependencies expose selected user fields (`id`, `email`, `role`, and `balance`) to all downstream handlers (`auth.py:11-12`). Business dependencies expose all columns of the restaurant or courier rows (`auth.py:44`, `auth.py:56`). This is internal dependency data, not directly a response, but any handler that blindly serializes it can leak profile or balance fields.
- **No hashing or issuance:** This file neither hashes secrets nor issues tokens; it trusts the `auth_token` contents produced elsewhere (`auth.py:12-14`). Its claims about credential strength are therefore incomplete without reviewing issuance.
- **No ownership authorization:** Role checks are class-level. Ownership of a specific restaurant, order, or courier assignment must be enforced in routes; `require_restaurant` only proves the caller has some restaurant profile (`auth.py:39-48`).

## Test Checklist

1. **No Authorization header, protected endpoint.** Setup: start API with a seeded database. Action: call an endpoint using `get_current_user` without the header. Expected: HTTP 401, no user row returned.
2. **Unknown bearer token, protected endpoint.** Setup: database has no matching `auth_token`. Action: send `Authorization: Bearer nonexistent`. Expected: HTTP 401.
3. **Known token, protected endpoint.** Setup: insert a token for a valid customer. Action: send `Authorization: Bearer <token>`. Expected: HTTP 2xx where the endpoint permits it, and downstream sees the correct `id`, `email`, `role`, and `balance`.
4. **Lowercase scheme.** Setup: valid token exists. Action: send `Authorization: bearer <token>`. Expected: treated as unauthenticated and HTTP 401 on mandatory auth.
5. **No space after Bearer.** Setup: valid token exists. Action: send `Authorization: Bearer<token>`. Expected: HTTP 401 on mandatory auth.
6. **Token for deleted user.** Setup: token row exists, user row is removed. Action: call mandatory endpoint. Expected: HTTP 401 because the inner join returns no row.
7. **Customer on customer-only endpoint.** Setup: valid customer token. Action: call an endpoint guarded by `require_customer`. Expected: HTTP 2xx and customer identity available.
8. **Restaurant on customer-only endpoint.** Setup: valid restaurant token. Action: call the same endpoint. Expected: HTTP 403, not HTTP 401.
9. **Restaurant role without restaurant profile.** Setup: user has `role = "restaurant"` but no `restaurant` row. Action: call an endpoint guarded by `require_restaurant`. Expected: HTTP 403 with `"Restoran bulunamadi"`.
10. **Valid restaurant profile.** Setup: role and matching `restaurant.user_id` exist. Action: call guarded endpoint. Expected: handler receives the correct `(user, restaurant)` pair.
11. **Courier role without courier profile.** Setup: role exists but no `courier` row. Action: call `require_courier` endpoint. Expected: HTTP 403 with `"Kurye profili bulunamadi"`.
12. **Optional endpoint while anonymous.** Setup: endpoint uses `get_optional_user`. Action: omit Authorization. Expected: route executes and observes `user is None`.
13. **Optional endpoint with valid token.** Setup: valid token. Action: call route. Expected: route receives the user row.
14. **Token reused after direct database removal.** Setup: valid token, then delete its `auth_token` row. Action: call protected endpoint. Expected: HTTP 401.
