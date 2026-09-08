from datetime import datetime, timezone
from typing import Any

from fastapi import Depends, FastAPI, HTTPException
from pydantic import BaseModel, Field

from accounts import router as accounts_router
from auth import get_optional_user, require_customer
from db import get_db
from panels import router as panels_router
from reviews import displayed_rating, router as reviews_router

app = FastAPI(title="Sepet API")

app.include_router(accounts_router)
app.include_router(panels_router)
app.include_router(reviews_router)


class OrderItemInput(BaseModel):
    menu_item_id: int
    quantity: int = Field(gt=0)


class OrderInput(BaseModel):
    restaurant_id: int
    address: str
    phone: str
    note: str = ""
    items: list[OrderItemInput]


@app.get("/api/restaurants")
def list_restaurants(q: str = "", cuisine: str = "", min_rating: float = 0.0, max_delivery_fee: float = 999.0, max_eta_minutes: int = 999, sort: str = "rating") -> list[dict[str, Any]]:
    conditions = []
    params: list[str] = []
    if q:
        conditions.append("(name LIKE ? OR cuisine LIKE ?)")
        search = f"%{q.lower()}%"
        params.extend([search, search])
    if cuisine:
        conditions.append("cuisine = ?")
        params.append(cuisine.lower())
    query = "SELECT * FROM restaurant"
    if conditions:
        query += " WHERE " + " AND ".join(conditions)
    with get_db() as conn:
        rows = conn.execute(query, params).fetchall()
        review_stats = {
            row["restaurant_id"]: row["avg_rating"]
            for row in conn.execute(
                "SELECT restaurant_id, AVG(rating) AS avg_rating FROM restaurant_review GROUP BY restaurant_id"
            ).fetchall()
        }
        result = []
        for row in rows:
            item = dict(row)
            if row["id"] in review_stats:
                item["rating"] = displayed_rating(
                    row["rating"], [{"rating": review_stats[row["id"]]}]
                )
            result.append(item)
    result = [item for item in result if item["rating"] >= min_rating and item["delivery_fee"] <= max_delivery_fee and item["eta_minutes"] <= max_eta_minutes]
    sorters = {
        "rating": lambda item: item["rating"],
        "eta": lambda item: item["eta_minutes"],
        "delivery_fee": lambda item: item["delivery_fee"],
        "min_order": lambda item: item["min_order"],
    }
    return sorted(result, key=sorters.get(sort, sorters["rating"]))


@app.get("/api/cuisines")
def list_cuisines() -> list[str]:
    with get_db() as conn:
        rows = conn.execute(
            "SELECT DISTINCT cuisine FROM restaurant ORDER BY cuisine"
        ).fetchall()
    return [row["cuisine"] for row in rows]


@app.get("/api/restaurants/{restaurant_id}")
def get_restaurant(restaurant_id: int, user=Depends(get_optional_user)) -> dict[str, Any]:
    with get_db() as conn:
        restaurant = conn.execute(
            "SELECT * FROM restaurant WHERE id = ?", (restaurant_id,)
        ).fetchone()
        if not restaurant:
            raise HTTPException(status_code=404, detail="Restoran bulunamadi")
        menu = conn.execute(
            "SELECT * FROM menu_item WHERE restaurant_id = ? ORDER BY category, name",
            (restaurant_id,),
        ).fetchall()
        reviews = conn.execute(
            """SELECT rr.id, rr.rating, rr.comment, rr.created_at, u.email AS user_email
               FROM restaurant_review rr JOIN "user" u ON u.id = rr.user_id
               WHERE rr.restaurant_id = ? ORDER BY rr.created_at DESC""",
            (restaurant_id,),
        ).fetchall()
    result = dict(restaurant)
    result["menu_items"] = [dict(item) for item in menu]
    result["reviews"] = [dict(review) for review in reviews]
    result["rating"] = displayed_rating(result["rating"], reviews)
    result["can_review"] = False
    result["has_review"] = False
    if user and user["role"] == "customer":
        result["can_review"] = bool(conn.execute(
            """SELECT 1 FROM "order"
               WHERE customer_id = ? AND restaurant_id = ? AND status = 'onaylandÄ±'""",
            (user["id"], restaurant_id),
        ).fetchone())
        result["has_review"] = bool(conn.execute(
            "SELECT 1 FROM restaurant_review WHERE restaurant_id = ? AND user_id = ?",
            (restaurant_id, user["id"]),
        ).fetchone())
    return result


@app.get("/api/orders/{order_id}")
def get_order(order_id: int) -> dict[str, Any]:
    with get_db() as conn:
        order = conn.execute(
            """SELECT o.*, r.name AS restaurant_name
               FROM "order" o
               JOIN restaurant r ON r.id = o.restaurant_id
               WHERE o.id = ?""",
            (order_id,),
        ).fetchone()
        if not order:
            raise HTTPException(status_code=404, detail="Siparis bulunamadi")
        items = conn.execute(
            """SELECT oi.id, oi.menu_item_id, oi.name, oi.unit_price, oi.quantity,
                      oi.unit_price * oi.quantity AS line_total
               FROM order_item oi WHERE oi.order_id = ? ORDER BY oi.id""",
            (order_id,),
        ).fetchall()
    result = dict(order)
    result["items"] = [dict(item) for item in items]
    return result


@app.post("/api/orders", status_code=201)
def create_order(payload: OrderInput, user=Depends(require_customer)) -> dict[str, Any]:
    if not payload.items:
        raise HTTPException(status_code=400, detail="Siparis icin en az bir urun gerekli")
    if user and user["role"] != "customer":
        raise HTTPException(status_code=403, detail="Siparis icin musteri hesabi kullanin")

    with get_db() as conn:
        restaurant = conn.execute(
            "SELECT * FROM restaurant WHERE id = ?", (payload.restaurant_id,)
        ).fetchone()
        if not restaurant:
            raise HTTPException(status_code=404, detail="Restoran bulunamadi")

        placeholders = ",".join("?" for _ in payload.items)
        menu_rows = conn.execute(
            f"""SELECT id, name, price FROM menu_item
                WHERE restaurant_id = ? AND id IN ({placeholders})""",
            [payload.restaurant_id, *[item.menu_item_id for item in payload.items]],
        ).fetchall()
        menu_by_id = {row["id"]: row for row in menu_rows}
        if len(menu_by_id) != len({item.menu_item_id for item in payload.items}):
            raise HTTPException(status_code=404, detail="Menu urunu bulunamadi")

        subtotal = sum(menu_by_id[item.menu_item_id]["price"] * item.quantity for item in payload.items)
        if subtotal < restaurant["min_order"]:
            missing = restaurant["min_order"] - subtotal
            raise HTTPException(
                status_code=400,
                detail=f"Minimum sepet tutarinin {missing:.2f} TL altindasiniz",
            )

        total = round(subtotal + restaurant["delivery_fee"], 2)
        missing_balance = round(max(0, total - user["balance"]), 2)
        if missing_balance > 0:
            raise HTTPException(
                status_code=400,
                detail=f"Bakiyeniz yetersiz. Siparis icin {missing_balance:.2f} TL daha yukleyin",
            )
        deducted = conn.execute(
            'UPDATE "user" SET balance = balance - ? WHERE id = ? AND balance >= ?',
            (total, user["id"], total),
        ).rowcount
        if not deducted:
            raise HTTPException(status_code=400, detail="Bakiye dusurulemedi")

        now = datetime.now(timezone.utc).isoformat()
        cursor = conn.execute(
            """INSERT INTO "order"
               (restaurant_id, address, phone, note, total, status, created_at, customer_id)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                payload.restaurant_id,
                payload.address,
                payload.phone,
                payload.note,
                total,
                "hazÄ±rlanÄ±yor",
                now,
                user["id"],
            ),
        )
        order_id = cursor.lastrowid
        for item in payload.items:
            menu_item = menu_by_id[item.menu_item_id]
            conn.execute(
                """INSERT INTO order_item
                   (order_id, menu_item_id, name, unit_price, quantity)
                   VALUES (?, ?, ?, ?, ?)""",
                (order_id, item.menu_item_id, menu_item["name"], menu_item["price"], item.quantity),
            )

    return {"id": order_id, "status": "hazÄ±rlanÄ±yor", "total": total}


