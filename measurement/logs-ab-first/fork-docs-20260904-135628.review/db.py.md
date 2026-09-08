# Review: `sepet/api/db.py`

## 1. Purpose

This small utility module owns three cross-cutting backend primitives: opening the SQLite database, generating timestamp strings, and hashing or verifying user passwords. It does not define API behavior itself, but nearly every persistence-bearing backend path depends on it.

`DB_PATH` points to `sepet.db` beside this file (`db.py:8`). `get_db()` is the connection factory (`db.py:11-14`), used by authentication helpers and route handlers for reads and writes. `utc_now()` supplies ISO-8601 timestamp strings (`db.py:17-18`), typically persisted into `created_at`-style columns. `hash_password()` and `verify_password()` implement the PBKDF2 password storage contract (`db.py:21-33`), used by registration/login code and therefore critical to account security.

The module is intentionally compact. The important consequence is that any SQLite configuration omitted here applies globally: every caller gets whatever defaults `sqlite3.connect` supplies rather than a configured transaction, foreign-key, timeout, or retry policy.

## 2. Walkthrough

### `DB_PATH` module constant - `db.py:8`

- Value: `Path(__file__).with_name("sepet.db")`.
- Behavior: computes the database location from the source file location, not the process working directory. This makes the database predictable when the server is started from another directory, provided the environment has write access to the module directory.

### `get_db() -> sqlite3.Connection` - `db.py:11-14`

- Parameters: none.
- Returns: an open `sqlite3.Connection`.
- Control flow: `db.py:12` calls `sqlite3.connect(DB_PATH)`; `db.py:13` sets `row_factory = sqlite3.Row` so row values can be accessed by column name; `db.py:14` returns the connection.
- The function does not enable foreign keys, set busy timeout, choose a transaction isolation mode, execute schema checks, or close the connection. Callers are expected to use `with get_db() as conn:` for transactional commit/rollback semantics, but Python's connection context manager does not close it. No caller behavior is implemented here, so leaks depend on all call sites.

### `utc_now() -> str` - `db.py:17-18`

- Parameters: none.
- Returns: the current UTC time as an ISO-8601 string, including timezone offset and fractional seconds.
- Control flow: `datetime.now(timezone.utc).isoformat()` is evaluated and returned. Because it is timezone-aware, stored strings are self-describing if read carefully; lexicographic comparison across fractional seconds is generally practical for same-precision values but is not a formal database timestamp type.

### `hash_password(password: str) -> str` - `db.py:21-24`

- Parameters: a plaintext password string.
- Returns: a stored credential string in the form `<32-hex-character salt>$<64-hex-character PBKDF2 digest>`.
- Control flow: generates a random 16-byte salt as hexadecimal with `secrets.token_hex(16)` (`db.py:22`). It computes SHA-256 PBKDF2-HMAC over the UTF-8 password bytes and the salt bytes using 120,000 iterations (`db.py:23`). It returns the hexadecimal salt and digest joined by `$` (`db.py:24`).
- The plaintext password is not retained by this function. The digest size is fixed by SHA-256.

### `verify_password(password: str, stored: str) -> bool` - `db.py:27-33`

- Parameters: a candidate plaintext password and the previously stored salt-plus-digest string.
- Returns: `True` for a valid candidate password, `False` for a malformed stored string or a failed verification.
- Control flow: `db.py:29` splits `stored` into a salt and expected digest at the first `$`. `db.py:30` recomputes the same PBKDF2-SHA256 construction with 120,000 iterations. `db.py:31` compares the newly computed hex digest to the expected value using `secrets.compare_digest`, which is a constant-time comparison for the compared values. `db.py:32-33` catches only `ValueError` and returns `False` for a stored string without a `$` separator.
- The function does not validate salt length, expected digest length, encoding, or the PBKDF2 iteration/version. A malformed-but-splittable stored string can therefore produce an ordinary cryptographic mismatch rather than a validation error.

## 3. Failure modes

- Foreign-key constraints are inactive by default: `get_db()` only opens the connection and sets `row_factory` (`db.py:11-14`). SQLite requires `PRAGMA foreign_keys = ON` per connection, so insertions or updates can create orphan records even when the schema declares foreign keys.
- Connection lifecycle is delegated: `with sqlite3.Connection` commits or rolls back but does not close the object. Since `get_db()` does not close it (`db.py:11-14`), a call site using `conn = get_db()` without explicit close leaks a file handle until garbage collection.
- Database location can fail in read-only deployments: because `DB_PATH` is fixed next to the source file (`db.py:8`), a production container with a read-only application directory or a mounted data volume outside it cannot be configured through an environment variable.
- Concurrent writes may surface `database is locked`: `sqlite3.connect` defaults to a five-second busy timeout, but no timeout is explicitly tuned here (`db.py:12`). Several simultaneous order, wallet, or review writes can produce an HTTP 500 rather than a controlled retry.
- Missing migrations or schema initialization: `get_db()` assumes `sepet.db` exists and has the expected schema (`db.py:12`). A missing file causes SQLite to create a new empty database on first connection, then later queries can fail in confusing ways.
- Timestamp precision and parsing: `utc_now()` includes fractional seconds (`db.py:17-18`). External systems or tests that format to whole seconds can create comparisons that do not match expectations.
- UTC string awareness: timezone-aware ISO strings are returned (`db.py:18`). Code that assumes timestamps are local time or strips the offset can order events incorrectly.
- Password encoding assumptions: `password.encode()` defaults to UTF-8 (`db.py:23`, `db.py:30`). That is normal, but alternate normalized password forms, Unicode normalization, or trailing whitespace are not handled.
- Stored format evolution is unsupported: verification only understands one unversioned `salt$digest` shape (`db.py:29-31`). Increasing iterations or switching algorithms later requires compatibility logic.
- Malformed stored values: only the missing separator case is explicitly caught (`db.py:32-33`). A wrong-length expected digest returns false; a non-UTF-8 or non-string stored value may raise another exception and turn login into a server error.
- Salt duplication is theoretically possible: random salts are used without uniqueness enforcement (`db.py:22`). This is acceptable probabilistically, but not a database-enforced invariant.

## 4. Security

- Password storage uses PBKDF2-HMAC-SHA256 with 120,000 iterations and random salts (`db.py:22-23`). This is a defensible baseline, though current platform capabilities may justify a higher iteration count or a modern memory-hard function.
- Verification uses `secrets.compare_digest` (`db.py:31`), reducing timing-oracle risk around digest comparison.
- Salt and digest are stored together in the password column (`db.py:24`). This is standard, but requires the schema to keep that column access-controlled.
- No plaintext password is logged or persisted by these helpers (`db.py:21-33`).
- Token security is not addressed here. This module has no token generation, expiry, or revocation primitive, so authentication behavior depends entirely on separate code and schema.
- SQL injection: the module itself executes no user-parameterized SQL. Every query remains the caller's responsibility.
- Authorization: `get_db()` performs no user identity checks (`db.py:11-14`). Any code with the returned connection can access every table; row-level authorization must happen in route logic.
- Data exposure: returning a live connection with `row_factory` (`db.py:13`) makes it easy for callers to `SELECT *` and accidentally expose new columns. Responses must explicitly select fields.
- Failure information: `verify_password` returns a boolean and does not distinguish malformed storage from a wrong password (`db.py:31-33`), avoiding a useful attacker signal at this layer.
- DoS consideration: password hashing is deliberately expensive (`db.py:23`, `db.py:30`). Unauthenticated login or registration endpoints should apply rate limiting because this module cannot.

## 5. Test checklist

1. Setup: initialize the expected schema at `sepet.db`. Action: call `get_db()`. Expected: an open `sqlite3.Connection` whose `row_factory` is `sqlite3.Row` and whose rows support named access.
2. Setup: insert a parent row and child row with a valid foreign key. Action: enable foreign keys on that connection, attempt to delete the parent. Expected: SQLite raises an `IntegrityError`; then repeat with `PRAGMA foreign_keys=ON` established in `get_db()` after a fix and verify the same protection applies automatically.
3. Setup: schema declares foreign keys. Action: insert an orphan child while foreign keys remain off. Expected: current implementation permits it, documenting the bug; after the fix it must fail.
4. Setup: start the API from a different working directory. Action: perform a database read/write. Expected: the same `sepet/api/sepet.db` file is used because `DB_PATH` is source-relative.
5. Setup: two open connections. Action: begin a write transaction in one and immediately write from the other. Expected: behavior is documented; after tuning, busy timeout or retries should yield a controlled outcome rather than an immediate lock error.
6. Setup: use `with get_db() as conn:`. Action: insert a row and allow normal context exit. Expected: transaction commits and data is visible afterward.
7. Setup: use `with get_db() as conn:`. Action: raise an exception after an insert. Expected: the context manager rolls back the insert; a follow-up close check should also confirm no connection leak.
8. Setup: call `utc_now()` twice. Action: compare returned strings and their parsed timezone. Expected: both parse as timezone-aware UTC values and the second is not earlier than the first.
9. Setup: two users with the same plaintext password. Action: call `hash_password` twice. Expected: salts differ and stored hashes differ.
10. Setup: hash a known password. Action: call `verify_password` with the correct password. Expected: returns `True`.
11. Setup: hash a known password. Action: call `verify_password` with incorrect passwords differing by case, Unicode form, and trailing whitespace. Expected: each returns `False`.
12. Setup: stored values `plaintext`, `"salt"` with no `$`, and `"a$short"` with a wrong-length digest. Action: call `verify_password`. Expected: no separator returns `False`; the team decides whether malformed splittable values also fail safely without a 500.
13. Setup: performance test. Action: measure registration/login hashing. Expected: work factor remains at 120,000 iterations unless a documented migration changes it; malformed or absent rate limits are noted separately.