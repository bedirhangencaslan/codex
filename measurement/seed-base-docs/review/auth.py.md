# Code Review: `sepet/api/auth.py`

## 1. Purpose

This module is the backend authentication and role-gating layer. It converts an HTTP `Authorization` header into a database-backed user identity, refuses anonymous access where required, and enforces coarse-grained role checks. Every protected route in the application depends on one of its dependency functions. The account endpoints use `get_current_user` for profile and wallet operations, customer-only order and review routes use `require_customer`, restaurant panel operations use `require_restaurant`, and courier panel operations use `require_courier`. Public or flexible endpoints may use `resolve_user` or `get_optional_user` when identity should enrich the response but should not be mandatory.

The module also exposes a shared convention for role-shaped dependency results: ordinary user identity is returned as a SQLite row, while restaurant and courier gates return a pair containing the user row and the matching profile row. That convention is used by route handlers to access user IDs, balance, role, restaurant ownership, and courier ownership.

## 2. Walkthrough

### `_user_from_token(conn, token: str | None)` — lines 6-15

Parameters:

- `conn`: an open SQLite connection, expected to use the application's row factory.
- `token`: the raw HTTP authorization header value, or `None`.

Returns a SQLite row containing `u.id`, `u.email`, `u.role`, and `u.balance` when the header names the Bearer scheme and the credential matches `auth_token.token`; otherwise returns `None`.

Control flow:

1. At line 7 it returns `None` for a missing, empty, or non-Bearer token.
2. At line 9 it removes the literal `Bearer ` prefix.
3. At lines 10-15 it joins `auth_token` to `"user"` on the token's `user_id` and selects the user fields using a parameterized query. SQLite returns no row if the credential is unknown.

### `resolve_user(authorization: str | None = Header(default=None))` — lines 18-20

Parameter:

- `authorization`: FastAPI supplies the `Authorization` request header; the default is `None`.

Returns the result of `_user_from_token`: a user row or `None`.

Control flow:

1. It opens a database connection with a context manager at line 19.
2. It delegates lookup to `_user_from_token` at line 20.
3. It does not itself raise an authentication error; callers decide whether missing identity is acceptable.

### `get_current_user(user=Depends(resolve_user))` — lines 23-26

Parameter:

- `user`: the result of `resolve_user` injected by FastAPI.

Returns the authenticated user row.

Control flow:

1. At line 24, if the user is absent or falsey, it raises HTTP `401` with a Turkish message.
2. At line 26, it returns the user row for successful authentication.

### `get_optional_user(user=Depends(resolve_user))` — lines 29-30

Parameter:

- `user`: the result of `resolve_user` injected by FastAPI.

Returns the same value unchanged: a user row or `None`. Its control flow is a single return, making optional identity explicit for endpoints that must accept anonymous visitors.

### `require_customer(user=Depends(get_current_user))` — lines 33-36

Parameter:

- `user`: an already authenticated user row supplied through `get_current_user`.

Returns the user row.

Control flow:

1. `get_current_user` guarantees that `user` is present.
2. At lines 34-35, if `user["role"]` is not exactly `"customer"`, it raises HTTP `403`.
3. At line 36, it returns the customer user row.

### `require_restaurant(user=Depends(get_current_user))` — lines 39-48

Parameter:

- `user`: an already authenticated user row.

Returns `(user, restaurant)`, where `restaurant` is the row associated with that user.

Control flow:

1. At lines 40-41, any role other than `"restaurant"` produces HTTP `403`.
2. At lines 42-45, it opens a new connection and queries `restaurant` by `user_id`.
3. At lines 46-47, if no restaurant profile exists, it raises HTTP `403` with a "restaurant not found" message.
4. At line 48, it returns both the user and restaurant profile.

### `require_courier(user=Depends(get_current_user))` — lines 51-60

Parameter:

- `user`: an already authenticated user row.

Returns `(user, courier)`, where `courier` is the row associated with that user.

Control flow:

1. At lines 52-53, any role other than `"courier"` produces HTTP `403`.
2. At lines 54-57, it queries `courier` by `user_id`.
3. At lines 58-59, a missing courier profile raises HTTP `403`.
4. At line 60, it returns the user and courier profile.

## 3. Failure modes

1. **Expired, revoked, or rotated tokens remain valid.** The lookup at `auth.py:10-15` checks only token existence, not `created_at`, expiration, revocation, or logout state. A database row for an old token continues to authenticate indefinitely.
2. **Case-sensitive Bearer handling rejects valid RFC-style variants.** `_user_from_token` requires the exact prefix at `auth.py:7-9`. Headers such as `bearer <token>` are treated as absent.
3. **Whitespace after the token is not accepted.** The prefix is removed exactly at `auth.py:9`; a client sending `"Bearer abc "` will query a token string containing a trailing space and fail.
4. **Unexpected `False` behavior from dependency chaining is limited but tight coupling remains.** `get_current_user` relies on `resolve_user`; `require_customer`, `require_restaurant`, and `require_courier` all rely on `get_current_user` (`auth.py:23`, `auth.py:33`, `auth.py:39`, `auth.py:51`). A change to the identity shape silently affects every gate.
5. **Role/profile consistency is enforced only at request time.** A restaurant whose user record is later changed from `"restaurant"` to another role will lose access, even if its profile remains (`auth.py:40-48`). Conversely, profile rows are looked up only for restaurant and courier, not customer.
6. **Database exceptions surface as server errors.** Every database call (`auth.py:10-15`, `auth.py:42-45`, `auth.py:54-57`) can raise if the database is unavailable or schema is out of sync; the module does not translate these into controlled HTTP responses.
7. **Disconnected identity and profile query timing.** `require_restaurant` and `require_courier` open a second connection after the identity lookup (`auth.py:42`, `auth.py:54`). If rows are changed concurrently, the identity used in the response may no longer match current profile state.
8. **No authentication error differentiation.** Missing header, malformed scheme, and unknown credential all lead to the same 401 through `get_current_user` (`auth.py:7-8`, `auth.py:24-25`). This is often intentional, but makes client diagnosis harder.

## 4. Security

- **Authentication:** Credentials are checked against `auth_token` through `_user_from_token` at `auth.py:10-15`. The absence of any validity window means authentication does not expire.
- **Authorization:** `get_current_user` enforces presence at `auth.py:24-25`; `require_customer` at `auth.py:34-35`, `require_restaurant` at `auth.py:40-47`, and `require_courier` at `auth.py:52-59` enforce role/profile requirements. Dependencies that return the user row directly do not add authorization.
- **Injection:** All credential, user-ID, and role comparisons use SQL placeholders (`auth.py:10-15`, `auth.py:43-45`, `auth.py:55-57`); no user input is interpolated into SQL. There is no HTML/CLI output in this module, so output-escaping concerns are downstream.
- **Data exposure:** `_user_from_token` returns only `id`, `email`, `role`, and `balance` (`auth.py:11-14`), avoiding password hashes. However, `balance` is exposed to every dependency result, including restaurant and courier gates, and may be forwarded to route responses if callers serialize the row without filtering. `resolve_user` returns `None` for malformed headers rather than leaking lookup details (`auth.py:7-8`).

## 5. Test checklist

1. **No authorization header.** Setup: launch app with a seeded database. Action: call a `get_current_user`-protected endpoint without `Authorization`. Expected: HTTP 401.
2. **Malformed scheme.** Setup: create a valid token. Action: send `Authorization: Token <token>`. Expected: HTTP 401.
3. **Lowercase scheme.** Setup: create a valid token. Action: send `Authorization: bearer <token>` to a protected route. Expected: with current code, HTTP 401; document whether the API intends to accept it.
4. **Unknown token.** Setup: seeded user/token database. Action: send `Bearer not-a-token`. Expected: HTTP 401.
5. **Valid customer token.** Setup: create a customer and token. Action: call an endpoint using `get_current_user`. Expected: HTTP 200 and identity fields match the customer.
6. **Customer gate blocks restaurant.** Setup: create a restaurant user and token. Action: call a `require_customer` route. Expected: HTTP 403.
7. **Customer gate allows customer.** Setup: create a customer and token. Action: call a `require_customer` route. Expected: HTTP 200 or the route's normal success response.
8. **Restaurant gate requires profile.** Setup: create a user with role `restaurant` but no restaurant row. Action: call a `require_restaurant` route. Expected: HTTP 403.
9. **Restaurant gate succeeds.** Setup: create a restaurant user plus matching restaurant row. Action: call a restaurant panel route. Expected: handler receives both user and restaurant; response is not 401/403.
10. **Courier gate requires profile.** Setup: create a courier-role user without a courier profile. Action: call a `require_courier` route. Expected: HTTP 403.
11. **Courier gate succeeds.** Setup: create courier-role user plus courier profile. Action: call a courier route. Expected: HTTP 200 or normal success.
12. **Optional identity anonymous.** Setup: no token. Action: call a `get_optional_user` route. Expected: endpoint executes and treats user as anonymous.
13. **Optional identity known.** Setup: valid token. Action: call same optional route. Expected: endpoint receives the user row.
14. **Token revocation.** Setup: create a token, delete or revoke it directly in the database. Action: call a protected route. Expected: if the API intends revocation, HTTP 401; with current behavior, the token remains accepted until the row is deleted.
15. **Database unavailable.** Setup: point DB_PATH at an unavailable/corrupt database. Action: call protected route. Expected: controlled 500 or appropriate recovery behavior, not a process crash.