import { useEffect, useState } from "react";
import { deliverOrder, fetchCourierOrders } from "../api";
import { Order } from "../types";

export default function CourierPanel() {
  const [orders, setOrders] = useState<Order[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const load = () => {
    setLoading(true);
    fetchCourierOrders()
      .then(setOrders)
      .catch(() => setError("Kurye siparişleri yüklenemedi"))
      .finally(() => setLoading(false));
  };

  useEffect(load, []);

  async function deliver(id: number) {
    await deliverOrder(id);
    load();
  }

  if (loading) return <div className="status">Siparişler yükleniyor...</div>;
  if (error) return <div className="status error">{error}</div>;
  if (!orders.length) return <div className="empty-state"><strong>Size atanmış sipariş yok</strong><p>Yeni atamalar burada listelenir.</p></div>;

  return (
    <div className="panel-list">
      {orders.map((order) => (
        <article key={order.id} className="panel-card">
          <div className="order-head">
            <strong>#{order.id} {order.restaurant_name}</strong>
            <span className="status-pill">{order.status}</span>
          </div>
          <p className="muted">{order.address}</p>
          <p className="muted">{order.phone} · {order.total.toFixed(2)} TL</p>
          <div className="order-actions">
            {order.status === "kuryede" && <button className="primary-button" onClick={() => deliver(order.id)}>Teslim ettim</button>}
          </div>
        </article>
      ))}
    </div>
  );
}