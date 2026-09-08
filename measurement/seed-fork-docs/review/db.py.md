# Code Review: `sepet/api/db.py`

## Purpose

This small utility module centralizes three cross-cutting backend concerns: opening the application's SQLite connection, generating canonical UTC timestamp strings, and password hashing/verification. `DB_PATH` is fixed relative to this file as `sepet.db` (`db.py:8`), so callers should receive a consistent database location regardless of the process working directory.

Nearly all route modules depend on `get_db()` for persistence, and authentication depends on `verify_password` indirectly during login and likely on `hash_password` during account creation or seeding. Timestamp generation through `utc_now()` standardizes ordering and audit fields. Because these primitives are shared, any weakness in connection setup, foreign-key enforcement, or password storage propagates broadly through the backend.

The module does not create tables or seed data. It opens whatever database currently exists at `DB_PATH`, leaving schema setup to `schema.sql` and `seed.py`. It also does not configure a connection timeout, isolation level, journal mode, or per-session `PRAGMA foreign_keys`; therefore database behavior under contention and referential-integrity enforcement are largely defaults.

## Walkthrough

### Module-level `DB_PATH` (`db.py:8`)

`DB_PATH` is a `Path` resolved with `Path(__file__).with_name("sepet.db")`. This pins the database beside `db.py`. It is not a function, but it is the module's only configuration point and all connections use it (`db.py:12`).

### `get_db() -> sqlite3.Connection` (`db.py:11`)

It takes no parameters and returns a SQLite connection.

Control flow:

1. Connect to `DB_PATH` (`db.py:12`).
2. Set the connection's row factory to `sqlite3.Row` so callers can access columns by name (`db.py:13`).
3. Return the connection (`db.py:14`).

It does not open a transaction immediately beyond SQLite's implicit behavior, does not enable foreign keys, and does not register cleanup. The expected pattern is `with get_db() as conn:` in callers, which commits on successful exit and rolls back on exception for Python's sqlite3 connection context manager. It does not close the connection unless the caller does so separately.

### `utc_now() -> str` (`db.py:17`)

It takes no parameters and returns a string.

Control flow is a single expression: get the current aware UTC datetime and return its ISO 8601 representation (`db.py:18`). The returned format includes timezone offset `+00:00` and precision typically to microseconds. This is useful for storage and lexicographic ordering, assuming all writers use the same helper.

### `hash_password(password: str) -> str` (`db.py:21`)

Its parameter is the plaintext password string. It returns a storage string in the form `<salt>$<hex digest>`.

Control flow:

1. Generate 16 cryptographically secure random bytes and hex-encode them as a 32-character salt (`db.py:22`).
2. Derive a SHA-256 PBKDF2 HMAC digest from the UTF-8-encoded password and salt using 120,000 iterations (`db.py:23`).
3. Return the salt and lowercase hex digest separated by `$` (`db.py:24`).

### `verify_password(password: str, stored: str) -> bool` (`db.py:27`)

Its parameters are a candidate plaintext password and the stored salt/digest string. It returns `True` when the candidate matches the stored value and `False` when it cannot match or the stored value is structurally invalid.

Control flow:

1. In a `try` block, split `stored` on the first `$` into salt and expected digest (`db.py:29`).
2. Recompute PBKDF2 with the same fixed algorithm, salt, and iteration count (`db.py:30`).
3. Compare the computed hex digest with `expected` in constant time (`db.py:31`).
4. Catch only `ValueError` and return `False` (`db.py:32-33`).

## Failure Modes

1. **Foreign-key enforcement is not enabled.** `get_db()` sets only `row_factory` (`db.py:11-14`). SQLite defaults to foreign keys off per connection, so route code can insert or update rows referencing missing restaurants, users, or orders unless every connection explicitly turns the pragma on. The AGENTS state file identifies this as a required backend improvement.
2. **Connection lifecycle depends on callers.** `get_db()` returns an open connection but does not close it (`db.py:11-14`). `with conn` manages transactions, not closure. A code path that forgets `conn.close()` can retain resources, especially after errors.
3. **No explicit busy timeout or retry policy.** `sqlite3.connect(DB_PATH)` uses defaults (`db.py:12`). Concurrent writes can raise `sqlite3.OperationalError: database is locked` rather than returning a controlled HTTP 503/429.
4. **No explicit journal mode or synchronous configuration.** The module does not choose WAL or other settings (`db.py:11-14`). Durability and reader/writer concurrency characteristics follow whatever the database file currently uses.
5. **Database path is fixed at import time.** `DB_PATH` is derived from the module location, not environment configuration (`db.py:8`). This simplifies deployment but prevents using a temporary database for tests without patching the module or replacing path construction.
6. **`utc_now()` precision and format can drift.** It returns `datetime.isoformat()` (`db.py:18`). If a caller writes another timestamp format, lexicographic ordering may break. The string includes `+00:00`, so mixed naive timestamps from external code may compare incorrectly.
7. **Fixed PBKDF2 iteration count.** Both hashing and verification hard-code 120,000 iterations (`db.py:23`, `db.py:30`). If the constant is later increased, old hashes can only be verified while the implementation retains versioning information; current stored strings have no algorithm/version marker.
8. **Password storage format has no algorithm metadata.** The format is only `salt$digest` (`db.py:24`). Migration from PBKDF2 to Argon2/bcrypt or changing iterations would require heuristics or a new column rather than reading a standard hash prefix.
9. **`verify_password` catches only `ValueError`.** A malformed stored value with the wrong number of components triggers `ValueError` (`db.py:29-32`), but a `None` stored value, a salt/digest with invalid Unicode, or a typing mismatch can raise another exception and bubble up. `password.encode()` can also fail for unsupported values if callers violate the type contract.
10. **Digest comparison is textual.** `compare_digest(digest.hex(), expected)` (`db.py:31`) is constant-time over the string. If `expected` has uppercase hex, extra whitespace, or a nonhex equivalent encoding, it fails even though cryptographically equivalent.
11. **No password strength validation.** The helper hashes any string (`db.py:21-24`); minimum-length and common-password rules must be enforced elsewhere.
12. **SQLite type affinity affects external schema, not this module.** `get_db()` itself is generic (`db.py:11-14`), so monetary columns, encoding quirks, and status literals must be handled by schema and route code.

## Security

- **Authentication support:** `hash_password` uses PBKDF2-HMAC-SHA256 with 120,000 iterations and a per-password 16-byte random salt (`db.py:21-24`), which is a materially better design than unsalted or fast hashes.
- **Verification:** Candidate comparison uses `secrets.compare_digest` (`db.py:31`), reducing timing side channels around digest equality.
- **No logging of secrets:** The module does not log passwords, hashes, tokens, or database contents.
- **Database access:** Connections are local to `DB_PATH` (`db.py:8`, `db.py:12`). SQL injection cannot arise from these helper functions because they execute no external query; callers must still parameterize.
- **Foreign keys:** Since `get_db()` does not enable `PRAGMA foreign_keys = ON` (`db.py:11-14`), referential integrity must not be assumed in authentication, orders, wallets, or reviews.
- **Data exposure:** The database path is predictable and colocated with code (`db.py:8`). Filesystem permissions determine exposure of password hashes and tokens.
- **Password API surface:** Passing an empty or trivially short password will be hashed (`db.py:21-24`). Login code must not treat an exception as a successful match; invalid stored formats return `False` for `ValueError` only (`db.py:27-33`).

## Test Checklist

1. **Connection row factory.** Setup: seeded database. Action: call `get_db()`, query a user, and access `row["email"]`. Expected: works without positional indexing.
2. **Database path stability.** Setup: run from project root and from another working directory. Action: import `db` and inspect/use `DB_PATH`. Expected: both resolve to the file beside `db.py`.
3. **Foreign keys currently disabled.** Setup: inspect a fresh connection. Action: `PRAGMA foreign_keys`. Expected: `0`, documenting the bug; after remediation expected `1`.
4. **Foreign-key violation behavior.** Setup: fresh connection. Action: attempt an insert violating a foreign key. Expected after fix: `IntegrityError`; currently may succeed depending on schema.
5. **Connection close.** Setup: open `get_db()` normally and call `close()`. Action: issue another query. Expected: `ProgrammingError` on the closed connection.
6. **Context manager rollback.** Setup: valid connection with a change attempt followed by an exception. Action: use `with get_db() as conn` and raise inside. Expected: no committed change.
7. **Context manager commit.** Setup: insert a row inside `with get_db() as conn`. Action: exit normally, reopen, query. Expected: row is visible.
8. **Concurrent writes.** Setup: two simultaneous connections. Action: perform dependent writes quickly. Expected: controlled completion or a documented timeout; no silent data loss.
9. **`utc_now` timezone.** Setup: freeze or compare with an independent UTC clock. Action: call `utc_now()`. Expected: string parses as UTC and includes `+00:00`.
10. **`utc_now` ordering.** Setup: generate timestamps across a short delay. Action: compare strings lexicographically. Expected: chronological order for outputs from this helper.
11. **Password hash uniqueness.** Setup: one fixed password. Action: call `hash_password` twice. Expected: different salts and therefore different stored strings.
12. **Correct password.** Setup: `stored = hash_password("correct horse")`. Action: `verify_password("correct horse", stored)`. Expected: `True`.
13. **Incorrect password.** Setup: hash a known password. Action: verify with another string. Expected: `False`.
14. **Malformed stored hash.** Setup: `stored` values `""`, `"abc"`, and `"a$b$c"`. Action: verify. Expected: `False` for strings producing `ValueError`; document behavior for other malformed types.
15. **Unicode password.** Setup: password containing Turkish characters. Action: hash then verify the exact string. Expected: `True`.
16. **Case-sensitive digest.** Setup: take a valid stored hash and uppercase the hex digest. Action: verify the correct password. Expected: `False`, revealing a normalization limitation.
17. **Hash migration marker.** Setup: replace the helper with a versioned format in the future. Action: verify old and new hashes. Expected: both succeed without ambiguous parsing.
