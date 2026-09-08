import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { fetchOrder } from "../api";
import type { Order } from "../types";

const formatTL = (value: number) => `${value.toFixed(2)} TL`;

export default function OrderPage() {
  const { id } = useParams();
  const [order, setOrder] = useState<Order | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    fetchOrder(Number(id))
      .then(setOrder)
      .catch(() => setError("Sipariş bulunamadı"))
      .finally(() => setLoading(false));
  }, [id]);

  if (loading) return <div className="status">Sipariş yükleniyor...</div>;
  if (error) return <div className="status error">{error}</div>;
  if (!order) return null;

  return (
    <section className="order-page">
      <div className="success-box">
        <div>
          <span className="eyebrow">Sipariş #{order.id}</span>
          <h1>Siparişin alındı</h1>
          <p>Durum: {order.status} · Toplam: {formatTL(order.total)}</p>
        </div>
      </div>
      <div className="order-layout">
        <article className="cart-panel">
          <h2>{order.restaurant_name}</h2>
          <p className="muted">{order.address} · {order.phone}</p>
          {order.note && <p className="muted">Not: {order.note}</p>}
          <div className="order-items">
            {order.items.map((item) => (
              <div key={item.id} className="cart-line compact">
                <span>{item.quantity} x {item.name}</span>
                <strong>{formatTL(item.line_total ?? item.unit_price * item.quantity)}</strong>
              </div>
            ))}
          </div>
          <div className="totals strong"><span>Toplam</span><span>{formatTL(order.total)}</span></div>
        </article>
        <aside className="cart-panel">
          <h2>Sonraki adım</h2>
          <p>Restoran siparişi hazırlıyor. Kurye atandığında durum burada güncellenir.</p>
          <Link className="primary-button" to="/">Başka restoranlara bak</Link>
        </aside>
      </div>
    </section>
  );
}