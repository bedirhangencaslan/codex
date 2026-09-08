# Code Review: `sepet/api/db.py`

## 1. Purpose

This is the backend's small data-access and cryptographic utility module. It centralizes the SQLite connection path and row factory, supplies a consistent UTC timestamp format, and provides password hashing and verification for account creation and login. Nearly all database-facing modules import `get_db`; seed and account code import the password helpers; route and panel code uses `utc_now` when creating orders, reviews, tokens, and other timestamped records.

The file deliberately separates connection construction from request handling. It does not define the schema, transactions, queries, or migrations itself. That makes it a foundational dependency: a change to `DB_PATH`, row wrapping, timestamp format, or password encoding propagates to login and every persisted timestamp.

## 2. Walkthrough

### Module-level constant — line 8

`DB_PATH = Path(__file__).with_name("sepet.db")` resolves the database location relative to `db.py`, not the current working directory. This makes CLI execution and server startup less sensitive to where Python is launched, provided the file lives beside the module.

### `get_db() -> sqlite3.Connection` — lines 11-14

Parameters: none.

Returns a newly opened `sqlite3.Connection` object.

Control flow:

1. Line 12 opens a SQLite connection to `DB_PATH`.
2. Line 13 sets `conn.row_factory = sqlite3.Row`, so queried rows support name-based access such as `row["email"]` as well as conversion with `dict(row)`.
3. Line 14 returns the connection. The caller owns closing it, typically through the `with get_db() as conn:` pattern used by routes.

Important behavioral details are implicit rather than explicit here. SQLite's default isolation behavior, foreign key enforcement, busy timeout, and journal mode are all left at library defaults unless changed elsewhere. In particular, this function does not issue `PRAGMA foreign_keys = ON`.

### `utc_now() -> str` — lines 17-18

Parameters: none.

Returns a string containing the current UTC time formatted by Python's ISO-8601 representation.

Control flow:

1. Line 18 calls `datetime.now(timezone.utc)`, guaranteeing an aware UTC timestamp.
2. The datetime is converted with `.isoformat()`, yielding a string that may include fractional seconds and a `+00:00` offset.

Because SQLite stores this as text, sorting is lexicographic. Within the same Python-produced format and UTC zone, lexicographic ordering is usually chronological, but alternate producers can make mixed timestamps harder to compare.

### `hash_password(password: str) -> str` — lines 21-24

Parameter:

- `password`: the plaintext password chosen at registration or account creation.

Returns a string of the form `<salt>$<hex digest>`.

Control flow:

1. Line 22 generates a random 16-byte salt and encodes it as 32 hexadecimal characters.
2. Line 23 computes PBKDF2-HMAC-SHA256 for 120,000 iterations using the UTF-8 password bytes and UTF-8-encoded salt.
3. Line 24 returns the salt and hexadecimal digest joined by `$`.

The salt is unique per password, so identical passwords do not normally produce identical hashes.

### `verify_password(password: str, stored: str) -> bool` — lines 27-33

Parameters:

- `password`: plaintext candidate presented for login.
- `stored`: previously produced hash string from the database.

Returns `True` when the candidate matches the stored hash; otherwise `False`.

Control flow:

1. Line 29 splits `stored` into a salt and expected digest at the first `$`.
2. Line 30 recomputes PBKDF2-HMAC-SHA256 using the same algorithm and iteration count as `hash_password`.
3. Line 31 compares the recomputed hexadecimal digest to the expected value with `secrets.compare_digest`, a constant-time comparison helper.
4. Lines 32-33 catch `ValueError` and return `False`.

The exception is intended for malformed stored values. A `None` stored value, however, raises `TypeError`, not `ValueError`, because it does not support `.split`.

## 3. Failure modes

1. **Foreign keys are not enabled.** `get_db` only connects and sets `row_factory` (`db.py:12-13`). SQLite leaves foreign key enforcement off by default for each connection, so routes can insert or update child rows in ways that reference missing parent rows unless the schema or environment does something else.
2. **Connection lifetime is entirely caller-dependent.** `get_db` returns an open connection (`db.py:14`) without context-managed cleanup of its own. A caller that forgets `with` or `close()` leaks file handles until garbage collection.
3. **Write errors and busy contention are unhandled.** Opening SQLite with defaults (`db.py:12`) provides no explicit busy timeout or retry policy. Under simultaneous writes, callers may receive uncontrolled `sqlite3.OperationalError`.
4. **No schema-version guard.** The module opens whatever file exists at `DB_PATH` (`db.py:8`, `db.py:12`). If the file is missing, SQLite may create an empty database; if it is stale, queries farther up the stack fail.
5. **Timestamp consumers must agree on format.** `utc_now` uses Python's variable-length ISO output (`db.py:18`). If another writer stores timestamps in a different precision or format, lexicographic SQLite ordering can disagree with chronological ordering.
6. **Password input cannot be empty-proofed here.** `hash_password` encodes whatever string it receives (`db.py:23`). The empty string is hashed successfully; whether it is valid is a caller concern. If callers do not validate, trivial passwords are accepted.
7. **Corrupt or malicious stored hash handling is partial.** A string without `$` raises `ValueError` and is treated as false (`db.py:29`, `db.py:32-33`), but malformed hex after `$` does not raise; it simply fails comparison. A `None` stored hash raises `TypeError` because line 29 cannot split it.
8. **Algorithm metadata is absent.** The stored value contains salt and digest (`db.py:24`) but no algorithm identifier, iteration count, or version. Future strengthening requires a migration strategy that can distinguish old and new hashes.
9. **Relative side effects of DB_PATH resolution.** Using `__file__` is robust for the bundled app, but tests that want a temporary database must patch or replace `DB_PATH` before opening connections (`db.py:8`, `db.py:12`).

## 4. Security

- **Authentication support:** This module does not authenticate requests itself, but account login depends on `verify_password` (`db.py:27-33`), and token/account creation depends on `hash_password` and `utc_now` elsewhere.
- **Password storage:** PBKDF2-HMAC-SHA256 with a random per-password salt and 120,000 iterations is used at `db.py:22-23`. Plaintext passwords are not intentionally persisted by these helpers.
- **Timing safety:** Digest comparison uses `secrets.compare_digest` rather than `==` (`db.py:31`), reducing timing leakage for matching digests.
- **Malformed hash:** Invalid split structure returns false rather than throwing authentication-flow errors (`db.py:29`, `db.py:32-33`), limiting error-based disruption.
- **Injection:** There are no SQL statements in this file. Connection construction and cryptographic operations use no string interpolation (`db.py:12`, `db.py:22-24`, `db.py:29-31`).
- **Data exposure:** The hash format necessarily exposes salt and digest to anyone who can read the database (`db.py:24`). It does not return password material to API callers. The lack of expiration or token logic here means broader authentication exposure is controlled by `auth.py` and account endpoints.
- **Configuration:** The database path is not externally configurable (`db.py:8`). This is predictable for deployment but makes secure test isolation harder unless patched deliberately.

## 5. Test checklist

1. **Row factory.** Setup: initialize schema and create a known user. Action: call `get_db`, query the user, access `row["email"]`. Expected: value is returned and `dict(row)` contains expected keys.
2. **Database location.** Setup: run backend from a different working directory. Action: open `get_db` and inspect the path or perform a query. Expected: it still resolves the database beside `db.py`.
3. **Connection closure.** Setup: open a connection with `with get_db() as conn:`. Action: exit the block and call `sqlite3.connect(DB_PATH)` for a write. Expected: no unexpected lock remains under normal SQLite semantics.
4. **UTC timestamp shape.** Setup: call `utc_now()`. Action: parse it with `datetime.fromisoformat` and compare `utcoffset()` to zero. Expected: parses successfully and offset is UTC.
5. **Timestamp ordering.** Setup: create records with successive `utc_now()` values. Action: sort strings lexicographically in SQLite. Expected: order matches creation order for this Python-produced format.
6. **Password hash uniqueness.** Setup: hash the same password twice. Action: compare both outputs. Expected: salts and full hashes differ.
7. **Correct password verification.** Setup: call `hash_password("correct horse")`. Action: call `verify_password("correct horse", stored)`. Expected: `True`.
8. **Wrong password verification.** Setup: hash a known password. Action: verify a different password. Expected: `False`.
9. **Malformed stored hash.** Setup: use `"no-dollar-sign"` as `stored`. Action: verify any password. Expected: `False`, not an exception.
10. **Malformed digest part.** Setup: use `"salt$not-hex"` as `stored`. Action: verify any password. Expected: `False`; comparison does not raise.
11. **Missing stored value variant.** Setup: pass `None` as `stored`. Action: call `verify_password`. Expected: document current behavior; implementation currently raises `TypeError`, so decide whether login should normalize it to `False`.
12. **Unicode password.** Setup: hash `"şifre✓"`. Action: verify the same Unicode string. Expected: `True`.
13. **Foreign key enforcement.** Setup: create a database with schema and foreign keys, call `get_db`. Action: insert a child row with a nonexistent parent. Expected: with current code it succeeds; after enabling `PRAGMA foreign_keys = ON` it should fail.
14. **Concurrent writes.** Setup: open two connections. Action: perform simultaneous transactions that contend. Expected: one completes; failure is handled by caller or an explicit timeout/retry strategy.
15. **Corrupt or missing database.** Setup: point `DB_PATH` to a corrupt file. Action: call `get_db` and query. Expected: controlled failure at the application boundary, with no silent empty-database behavior in production.