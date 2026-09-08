CREATE TABLE IF NOT EXISTS restaurant (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    cuisine TEXT NOT NULL,
    rating REAL NOT NULL,
    delivery_fee REAL NOT NULL,
    min_order REAL NOT NULL,
    eta_minutes INTEGER NOT NULL,
    image_emoji TEXT NOT NULL,
    user_id INTEGER REFERENCES "user"(id)
);

CREATE TABLE IF NOT EXISTS "user" (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('customer', 'restaurant', 'courier')),
    balance REAL NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS auth_token (
    token TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES "user"(id),
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS courier (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL UNIQUE REFERENCES "user"(id),
    restaurant_id INTEGER NOT NULL REFERENCES restaurant(id),
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS restaurant_review (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    restaurant_id INTEGER NOT NULL REFERENCES restaurant(id),
    user_id INTEGER NOT NULL REFERENCES "user"(id),
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comment TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (restaurant_id, user_id)
);

CREATE TABLE IF NOT EXISTS menu_item (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    restaurant_id INTEGER NOT NULL REFERENCES restaurant(id),
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    price REAL NOT NULL,
    category TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "order" (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    restaurant_id INTEGER NOT NULL REFERENCES restaurant(id),
    address TEXT NOT NULL,
    phone TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    total REAL NOT NULL,
    status TEXT NOT NULL DEFAULT 'hazÄ±rlanÄ±yor',
    created_at TEXT NOT NULL,
    customer_id INTEGER REFERENCES "user"(id),
    courier_id INTEGER REFERENCES courier(id),
    courier_assigned_at TEXT,
    delivered_at TEXT,
    confirmed_at TEXT
);

CREATE TABLE IF NOT EXISTS order_item (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL REFERENCES "order"(id),
    menu_item_id INTEGER NOT NULL REFERENCES menu_item(id),
    name TEXT NOT NULL,
    unit_price REAL NOT NULL,
    quantity INTEGER NOT NULL
);

