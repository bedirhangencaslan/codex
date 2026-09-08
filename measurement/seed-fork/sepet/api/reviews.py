from typing import Any

from fastapi import APIRouter, Depends, HTTPException

from auth import get_optional_user, require_customer
from db import get_db, utc_now
from panels import ReviewInput


router = APIRouter(prefix="/api")


def displayed_rating(seed_rating: float, review_rows) -> float:
    ratings = [seed_rating, *[row["rating"] for row in review_rows]]
    return round(sum(ratings) / len(ratings), 1)


@router.post("/restaurants/{restaurant_id}/reviews", status_code=201)
def create_review(
    restaurant_id: int,
    payload: ReviewInput,
    user=Depends(require_customer),
) -> dict[str, Any]:
    if not payload.comment.strip():
        raise HTTPException(status_code=400, detail="Yorum metni zorunludur")
    with get_db() as conn:
        restaurant = conn.execute(
            "SELECT id FROM restaurant WHERE id = ?", (restaurant_id,)
        ).fetchone()
        if not restaurant:
            raise HTTPException(status_code=404, detail="Restoran bulunamadi")
        approved = conn.execute(
            """SELECT 1 FROM "order"
               WHERE customer_id = ? AND restaurant_id = ? AND status = 'onaylandı'""",
            (user["id"], restaurant_id),
        ).fetchone()
        if not approved:
            raise HTTPException(status_code=403, detail="Yorum icin onayli siparis gerekli")
        if conn.execute(
            "SELECT 1 FROM restaurant_review WHERE restaurant_id = ? AND user_id = ?",
            (restaurant_id, user["id"]),
        ).fetchone():
            raise HTTPException(status_code=409, detail="Bu restorana yorum yaptiniz")
        cursor = conn.execute(
            """INSERT INTO restaurant_review
               (restaurant_id, user_id, rating, comment, created_at)
               VALUES (?, ?, ?, ?, ?)""",
            (restaurant_id, user["id"], payload.rating, payload.comment.strip(), utc_now()),
        )
    return {"id": cursor.lastrowid, "rating": payload.rating, "comment": payload.comment.strip(), "user_email": user["email"]}

