from fastapi import Depends, Header, HTTPException

from db import get_db


def _user_from_token(conn, token: str | None):
    if not token or not token.startswith("Bearer "):
        return None
    value = token.removeprefix("Bearer ")
    return conn.execute(
        """SELECT u.id, u.email, u.role, u.balance
           FROM auth_token t JOIN "user" u ON u.id = t.user_id
           WHERE t.token = ?""",
        (value,),
    ).fetchone()


def resolve_user(authorization: str | None = Header(default=None)):
    with get_db() as conn:
        return _user_from_token(conn, authorization)


def get_current_user(user=Depends(resolve_user)):
    if not user:
        raise HTTPException(status_code=401, detail="Giris yapmalisiniz")
    return user


def get_optional_user(user=Depends(resolve_user)):
    return user


def require_customer(user=Depends(get_current_user)):
    if user["role"] != "customer":
        raise HTTPException(status_code=403, detail="Bu islem icin musteri girisi gerekli")
    return user


def require_restaurant(user=Depends(get_current_user)):
    if user["role"] != "restaurant":
        raise HTTPException(status_code=403, detail="Bu islem icin restoran girisi gerekli")
    with get_db() as conn:
        restaurant = conn.execute(
            "SELECT * FROM restaurant WHERE user_id = ?", (user["id"],)
        ).fetchone()
    if not restaurant:
        raise HTTPException(status_code=403, detail="Restoran bulunamadi")
    return user, restaurant


def require_courier(user=Depends(get_current_user)):
    if user["role"] != "courier":
        raise HTTPException(status_code=403, detail="Bu islem icin kurye girisi gerekli")
    with get_db() as conn:
        courier = conn.execute(
            "SELECT * FROM courier WHERE user_id = ?", (user["id"],)
        ).fetchone()
    if not courier:
        raise HTTPException(status_code=403, detail="Kurye profili bulunamadi")
    return user, courier
