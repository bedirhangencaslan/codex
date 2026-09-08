import re
import secrets

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel, Field

from auth import get_current_user
from db import get_db, hash_password, utc_now, verify_password


router = APIRouter(prefix="/api/auth")


class RegisterInput(BaseModel):
    email: str
    password: str
    role: str
    restaurant_name: str = ""
    cuisine: str = ""
    delivery_fee: float = 10.0
    min_order: float = 50.0
    eta_minutes: int = 30
    image_emoji: str = "Ã„Å¸Ã…Â¸Ã‚ÂÃ‚Âª"


class LoginInput(BaseModel):
    email: str
    password: str


def user_payload(user) -> dict:
    return {"id": user["id"], "email": user["email"], "role": user["role"], "balance": round(user["balance"], 2)}


def validate_email(email: str) -> None:
    if not re.fullmatch(r"[^@\s]+@[^@\s]+\.[^@\s]+", email):
        raise HTTPException(status_code=400, detail="Gecerli bir e-posta girin")


def validate_registration(payload: RegisterInput) -> None:
    validate_email(payload.email)
    if len(payload.password) < 6:
        raise HTTPException(status_code=400, detail="Sifre en az 6 karakter olmalidir")
    if payload.role not in {"customer", "restaurant"}:
        raise HTTPException(status_code=400, detail="Gecersiz hesap turu")
    if payload.role == "restaurant" and (not payload.restaurant_name or not payload.cuisine):
        raise HTTPException(status_code=400, detail="Restoran adi ve mutfak zorunludur")


def issue_token(conn, user_id: int) -> str:
    token = secrets.token_urlsafe(32)
    conn.execute(
        "INSERT INTO auth_token (token, user_id, created_at) VALUES (?, ?, ?)",
        (token, user_id, utc_now()),
    )
    return token


@router.post("/register", status_code=201)
def register(payload: RegisterInput) -> dict:
    validate_registration(payload)
    with get_db() as conn:
        if conn.execute('SELECT 1 FROM "user" WHERE email = ?', (payload.email,)).fetchone():
            raise HTTPException(status_code=409, detail="Bu e-posta zaten kayitli")
        cursor = conn.execute(
            'INSERT INTO "user" (email, password_hash, role, balance) VALUES (?, ?, ?, ?)',
            (payload.email, hash_password(payload.password), payload.role, 100.0),
        )
        user_id = cursor.lastrowid
        if payload.role == "restaurant":
            conn.execute(
                """INSERT INTO restaurant
                   (name, cuisine, rating, delivery_fee, min_order, eta_minutes, image_emoji, user_id)
                   VALUES (?, ?, 4.0, ?, ?, ?, ?, ?)""",
                (
                    payload.restaurant_name,
                    payload.cuisine.lower(),
                    payload.delivery_fee,
                    payload.min_order,
                    payload.eta_minutes,
                    payload.image_emoji,
                    user_id,
                ),
            )
        token = issue_token(conn, user_id)
    return {"token": token, "user": {"id": user_id, "email": payload.email, "role": payload.role, "balance": 100.0}}


@router.post("/login")
def login(payload: LoginInput) -> dict:
    with get_db() as conn:
        user = conn.execute(
            'SELECT * FROM "user" WHERE email = ?', (payload.email,)
        ).fetchone()
        if not user or not verify_password(payload.password, user["password_hash"]):
            raise HTTPException(status_code=401, detail="E-posta veya sifre hatali")
        token = issue_token(conn, user["id"])
    return {"token": token, "user": user_payload(user)}


@router.get("/me")
def me(user=Depends(get_current_user)) -> dict:
    return user_payload(user)


class TopUpInput(BaseModel):
    amount: float = Field(gt=0, le=1000)


@router.post("/wallet/topup")
def top_up_wallet(payload: TopUpInput, user=Depends(get_current_user)) -> dict:
    if user["role"] != "customer":
        raise HTTPException(status_code=403, detail="Sadece musteriler bakiye yukleyebilir")
    amount = round(payload.amount, 2)
    with get_db() as conn:
        updated = conn.execute(
            'UPDATE "user" SET balance = balance + ? WHERE id = ?',
            (amount, user["id"]),
        ).rowcount
        if not updated:
            raise HTTPException(status_code=404, detail="Kullanici bulunamadi")
        row = conn.execute('SELECT balance FROM "user" WHERE id = ?', (user["id"],)).fetchone()
    return {"balance": round(row["balance"], 2), "added": amount}
