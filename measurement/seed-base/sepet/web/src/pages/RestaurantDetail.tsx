import { useEffect, useMemo, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { fetchRestaurant, submitReview } from "../api";
import { useCart } from "../CartContext";
import type { MenuItem, Restaurant } from "../types";

const formatTL = (value: number) => `${value.toFixed(2)} TL`;

export default function RestaurantDetail() {
  const { id } = useParams();
  const [restaurant, setRestaurant] = useState<Restaurant | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [reviewComment, setReviewComment] = useState("");
  const [reviewRating, setReviewRating] = useState(5);
  const cart = useCart();

  useEffect(() => {
    setLoading(true);
    fetchRestaurant(Number(id))
      .then((data) => {
        setRestaurant(data);
        setError("");
      })
      .catch(() => setError("Restoran yüklenemedi"))
      .finally(() => setLoading(false));
  }, [id]);

  const groupedMenu = useMemo(() => {
    const groups = new Map<string, MenuItem[]>();
    restaurant?.menu_items?.forEach((item) => {
      const existing = groups.get(item.category) ?? [];
      existing.push(item);
      groups.set(item.category, existing);
    });
    return [...groups.entries()];
  }, [restaurant]);

  function addToCart(item: MenuItem) {
    if (cart.restaurantId && cart.restaurantId !== item.restaurant_id) {
      const approved = window.confirm("Farklı restorandan ürün eklediniz. Sepet boşaltılacak.");
      if (!approved) return;
      cart.clear();
    }
    cart.add(item, restaurant?.delivery_fee ?? 0);
  }

  if (loading) return <div className="status">Menü yükleniyor...</div>;
  if (error) return <div className="status error">{error}</div>;
  if (!restaurant) return null;

  const ownCart = cart.restaurantId === restaurant.id ? cart : null;
  const cartEmpty = !ownCart || ownCart.lines.length === 0;

  return (
    <section className="restaurant-detail">
      <div className="restaurant-hero">
        <span className="hero-emoji" aria-hidden="true">{restaurant.image_emoji}</span>
        <div className="restaurant-hero-body">
          <span className="eyebrow">{restaurant.cuisine}</span>
          <h1>{restaurant.name}</h1>
          <div className="hero-facts">
            <span>{restaurant.rating.toFixed(1)} puan</span>
            <span>{restaurant.eta_minutes} dk</span>
            <span>Teslimat {formatTL(restaurant.delivery_fee)}</span>
            <span>Min. {formatTL(restaurant.min_order)}</span>
          </div>
        </div>
      </div>

      <div className="detail-layout">
        <div className="menu-list">
          {groupedMenu.map(([category, items]) => (
            <div key={category} className="menu-group">
              <h2>{category}</h2>
              {items.map((item) => (
                <article key={item.id} className="menu-card">
                  <div className="menu-copy">
                    <h3>{item.name}</h3>
                    <p>{item.description}</p>
                    <strong>{formatTL(item.price)}</strong>
                  </div>
                  <button onClick={() => addToCart(item)}>Ekle</button>
                </article>
              ))}
            </div>
          ))}

          <section className="review-section">
            <h2>Yorumlar</h2>
            {!restaurant.reviews?.length && <div className="status">Henüz yorum yok.</div>}
            {restaurant.reviews?.map((review) => (
              <article key={review.id} className="review-card">
                <div className="review-head">
                  <strong>{review.user_email}</strong>
                  <span className="rating">{review.rating}.0</span>
                </div>
                <p>{review.comment}</p>
              </article>
            ))}
            {restaurant.can_review && !restaurant.has_review && (
              <form
                className="review-form"
                onSubmit={async (event) => {
                  event.preventDefault();
                  await submitReview(restaurant.id, reviewRating, reviewComment);
                  const updated = await fetchRestaurant(restaurant.id);
                  setRestaurant(updated);
                  setReviewComment("");
                }}
              >
                <h3>Değerlendirme yap</h3>
                <label>
                  Puan
                  <select value={reviewRating} onChange={(event) => setReviewRating(Number(event.target.value))}>
                    {[5, 4, 3, 2, 1].map((rating) => <option key={rating} value={rating}>{rating} yıldız</option>)}
                  </select>
                </label>
                <label>
                  Yorum
                  <textarea
                    value={reviewComment}
                    onChange={(event) => setReviewComment(event.target.value)}
                    placeholder="Yemek ve teslimat deneyimini kısaca anlat."
                    required
                  />
                </label>
                <button className="primary-button">Yorumu gönder</button>
              </form>
            )}
          </section>
        </div>

        <aside className={cartEmpty ? "cart-panel is-empty" : "cart-panel"}>
          <h2>Sepetin</h2>
          {cartEmpty ? (
            <p>Sepet şu an boş. Menüden ekleyeceğin ürünler burada görünecek.</p>
          ) : (
            <>
              {ownCart?.lines.map((line) => (
                <div key={line.item.id} className="cart-line">
                  <span>{line.item.name}</span>
                  <div className="quantity">
                    <button onClick={() => cart.change(line.item.id, -1)} aria-label="Azalt">-</button>
                    <span>{line.quantity}</span>
                    <button onClick={() => cart.change(line.item.id, 1)} aria-label="Arttır">+</button>
                  </div>
                  <strong>{formatTL(line.item.price * line.quantity)}</strong>
                </div>
              ))}
              <div className="totals"><span>Ara toplam</span><span>{formatTL(cart.subtotal)}</span></div>
              <div className="totals"><span>Teslimat</span><span>{formatTL(restaurant.delivery_fee)}</span></div>
              <div className="totals strong"><span>Toplam</span><span>{formatTL(cart.subtotal + restaurant.delivery_fee)}</span></div>
              {cart.subtotal < restaurant.min_order && (
                <p className="warning">
                  Minimum sepet için {formatTL(restaurant.min_order - cart.subtotal)} daha ekleyin.
                </p>
              )}
              <Link className="primary-button" to="/checkout" state={{ deliveryFee: restaurant.delivery_fee }}>
                Sepeti onayla
              </Link>
            </>
          )}
        </aside>
      </div>
    </section>
  );
}