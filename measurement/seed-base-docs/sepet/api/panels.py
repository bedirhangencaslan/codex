from typing import Any, Literal

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel, Field

from auth import require_courier, require_customer, require_restaurant
from db import get_db, hash_password, utc_now


router = APIRouter(prefix="/api")


class CourierInput(BaseModel):
    email: str
    password: str
    name: str


class MenuItemInput(BaseModel):
    name: str
    description: str
    price: float = Field(gt=0)
    category: str


class AssignCourierInput(BaseModel):
    courier_id: int


class ReviewInput(BaseModel):
    rating: int = Field(ge=1, le=5)
    comment: str


def serialize_orders(conn, rows) -> list[dict[str, Any]]:
    result = []
    for row in rows:
        order = dict(row)
        order["items"] = [
            dict(item)
            for item in conn.execute(
                """SELECT id, menu_item_id, name, unit_price, quantity,
                          unit_price * quantity AS line_total
                   FROM order_item WHERE order_id = ? ORDER BY id""",
                (order["id"],),
            ).fetchall()
        ]
        result.append(order)
    return result


def order_with_items(conn, order_id: int):
    order = conn.execute(
        """SELECT o.*, r.name AS restaurant_name,
                  u.email AS customer_email, c.name AS courier_name
           FROM "order" o
           JOIN restaurant r ON r.id = o.restaurant_id
           LEFT JOIN "user" u ON u.id = o.customer_id
           LEFT JOIN courier c ON c.id = o.courier_id
           WHERE o.id = ?""",
        (order_id,),
    ).fetchone()
    if not order:
        return None
    return serialize_orders(conn, [order])[0]


@router.get("/orders/history")
def order_history(user=Depends(require_customer)) -> list[dict[str, Any]]:
    with get_db() as conn:
        rows = conn.execute(
            """SELECT o.*, r.name AS restaurant_name,
                      u.email AS customer_email, c.name AS courier_name
               FROM "order" o
               JOIN restaurant r ON r.id = o.restaurant_id
               LEFT JOIN "user" u ON u.id = o.customer_id
               LEFT JOIN courier c ON c.id = o.courier_id
               WHERE o.customer_id = ? ORDER BY o.created_at DESC""",
            (user["id"],),
        ).fetchall()
    return serialize_orders(conn, rows)


@router.get("/restaurant/orders")
def restaurant_orders(
    status: str = "",
    user_restaurant=Depends(require_restaurant),
) -> list[dict[str, Any]]:
    _, restaurant = user_restaurant
    query = """SELECT o.*, r.name AS restaurant_name,
                      u.email AS customer_email, c.name AS courier_name
               FROM "order" o
               JOIN restaurant r ON r.id = o.restaurant_id
               LEFT JOIN "user" u ON u.id = o.customer_id
               LEFT JOIN courier c ON c.id = o.courier_id
               WHERE o.restaurant_id = ?"""
    params: list[Any] = [restaurant["id"]]
    if status:
        query += " AND o.status = ?"
        params.append(status)
    with get_db() as conn:
        rows = conn.execute(query + " ORDER BY o.created_at DESC", params).fetchall()
    return serialize_orders(conn, rows)


@router.get("/restaurant/couriers")
def list_couriers(user_restaurant=Depends(require_restaurant)) -> list[dict[str, Any]]:
    _, restaurant = user_restaurant
    with get_db() as conn:
        rows = conn.execute(
            "SELECT id, name, user_id FROM courier WHERE restaurant_id = ? ORDER BY name",
            (restaurant["id"],),
        ).fetchall()
    return [dict(row) for row in rows]


@router.get("/restaurant/menu-items")
def list_menu_items(user_restaurant=Depends(require_restaurant)) -> list[dict[str, Any]]:
    _, restaurant = user_restaurant
    with get_db() as conn:
        rows = conn.execute(
            "SELECT * FROM menu_item WHERE restaurant_id = ? ORDER BY category, name",
            (restaurant["id"],),
        ).fetchall()
    return [dict(row) for row in rows]


@router.post("/restaurant/couriers", status_code=201)
def create_courier(payload: CourierInput, user_restaurant=Depends(require_restaurant)) -> dict:
    _, restaurant = user_restaurant
    if not payload.name.strip():
        raise HTTPException(status_code=400, detail="Kurye adi zorunludur")
    with get_db() as conn:
        if conn.execute('SELECT 1 FROM "user" WHERE email = ?', (payload.email,)).fetchone():
            raise HTTPException(status_code=409, detail="Bu e-posta zaten kayitli")
        cursor = conn.execute(
            'INSERT INTO "user" (email, password_hash, role) VALUES (?, ?, ?)',
            (payload.email, hash_password(payload.password), "courier"),
        )
        courier_id = cursor.lastrowid
        conn.execute(
            "INSERT INTO courier (user_id, restaurant_id, name) VALUES (?, ?, ?)",
            (courier_id, restaurant["id"], payload.name),
        )
    return {"id": courier_id, "name": payload.name, "restaurant_id": restaurant["id"]}


@router.post("/restaurant/menu-items", status_code=201)
def add_menu_item(payload: MenuItemInput, user_restaurant=Depends(require_restaurant)) -> dict:
    _, restaurant = user_restaurant
    with get_db() as conn:
        cursor = conn.execute(
            """INSERT INTO menu_item (restaurant_id, name, description, price, category)
               VALUES (?, ?, ?, ?, ?)""",
            (restaurant["id"], payload.name, payload.description, payload.price, payload.category),
        )
    return {"id": cursor.lastrowid, **payload.model_dump(), "restaurant_id": restaurant["id"]}


@router.post("/restaurant/orders/{order_id}/assign-courier")
def assign_courier(
    order_id: int,
    payload: AssignCourierInput,
    user_restaurant=Depends(require_restaurant),
) -> dict[str, str]:
    _, restaurant = user_restaurant
    now = utc_now()
    with get_db() as conn:
        order = conn.execute(
            'SELECT * FROM "order" WHERE id = ?', (order_id,)
        ).fetchone()
        if not order or order["restaurant_id"] != restaurant["id"]:
            raise HTTPException(status_code=404, detail="Siparis bulunamadi")
        if order["status"] != "hazırlanıyor":
            raise HTTPException(status_code=409, detail="Bu siparis icin kurye atanamaz")
        courier = conn.execute(
            "SELECT * FROM courier WHERE id = ? AND restaurant_id = ?",
            (payload.courier_id, restaurant["id"]),
        ).fetchone()
        if not courier:
            raise HTTPException(status_code=403, detail="Kurye bu restorana ait degil")
        conn.execute(
            """UPDATE "order" SET courier_id = ?, courier_assigned_at = ?, status = 'kuryede'
               WHERE id = ?""",
            (courier["id"], now, order_id),
        )
    return {"status": "kuryede", "courier_name": courier["name"]}


@router.get("/courier/orders")
def courier_orders(courier_data=Depends(require_courier)) -> list[dict[str, Any]]:
    _, courier = courier_data
    with get_db() as conn:
        rows = conn.execute(
            """SELECT o.*, r.name AS restaurant_name,
                      u.email AS customer_email, c.name AS courier_name
               FROM "order" o
               JOIN restaurant r ON r.id = o.restaurant_id
               LEFT JOIN "user" u ON u.id = o.customer_id
               JOIN courier c ON c.id = o.courier_id
               WHERE o.courier_id = ? ORDER BY o.created_at DESC""",
            (courier["id"],),
        ).fetchall()
    return serialize_orders(conn, rows)


@router.post("/courier/orders/{order_id}/deliver")
def deliver_order(order_id: int, courier_data=Depends(require_courier)) -> dict[str, str]:
    _, courier = courier_data
    now = utc_now()
    with get_db() as conn:
        order = conn.execute(
            'SELECT * FROM "order" WHERE id = ?', (order_id,)
        ).fetchone()
        if not order or order["courier_id"] != courier["id"]:
            raise HTTPException(status_code=404, detail="Siparis bulunamadi")
        if order["status"] != "kuryede":
            raise HTTPException(status_code=409, detail="Bu siparis teslim edilemez")
        conn.execute(
            """UPDATE "order" SET status = 'teslim_edildi', delivered_at = ? WHERE id = ?""",
            (now, order_id),
        )
    return {"status": "teslim_edildi"}


@router.post("/orders/{order_id}/confirm_delivery")
def confirm_delivery(order_id: int, user=Depends(require_customer)) -> dict[str, str]:
    now = utc_now()
    with get_db() as conn:
        order = conn.execute(
            'SELECT * FROM "order" WHERE id = ?', (order_id,)
        ).fetchone()
        if not order or order["customer_id"] != user["id"]:
            raise HTTPException(status_code=404, detail="Siparis bulunamadi")
        if order["status"] != "teslim_edildi":
            raise HTTPException(status_code=409, detail="Bu siparis onaylanamaz")
        conn.execute(
            """UPDATE "order" SET status = 'onaylandı', confirmed_at = ? WHERE id = ?""",
            (now, order_id),
        )
    return {"status": "onaylandı"}





@router.get("/restaurant/earnings")
def restaurant_earnings(user_restaurant=Depends(require_restaurant)) -> dict[str, Any]:
    _, restaurant = user_restaurant
    with get_db() as conn:
        overall = conn.execute(
            """SELECT COUNT(*) AS order_count,
                      COALESCE(SUM(total), 0) AS revenue
               FROM "order" WHERE restaurant_id = ?""",
            (restaurant["id"],),
        ).fetchone()
        confirmed = conn.execute(
            """SELECT COUNT(*) AS order_count,
                      COALESCE(SUM(total), 0) AS revenue
               FROM "order"
               WHERE restaurant_id = ? AND status = 'onayland\u0131'""",
            (restaurant["id"],),
        ).fetchone()
    average = round(overall["revenue"] / overall["order_count"], 2) if overall["order_count"] else 0
    return {
        "total_revenue": round(overall["revenue"], 2),
        "confirmed_revenue": round(confirmed["revenue"], 2),
        "pending_revenue": round(overall["revenue"] - confirmed["revenue"], 2),
        "order_count": overall["order_count"],
        "confirmed_order_count": confirmed["order_count"],
        "average_order_value": average,
    }