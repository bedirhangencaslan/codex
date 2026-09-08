import { useEffect, useState } from "react";
import {
  addCourier, addMenuItem, assignCourier, fetchCouriers,
  fetchRestaurantEarnings, fetchRestaurantMenu, fetchRestaurantOrders,
} from "../api";
import { Courier, Earnings, MenuItem, Order } from "../types";

type Tab = "orders" | "menu" | "couriers";

const statusLabel: Record<string, string> = {
  "hazırlanıyor": "Hazırlanıyor",
  kuryede: "Kuryede",
  teslim_edildi: "Teslim Edildi",
  "onaylandı": "Onaylandı",
};

export default function RestaurantPanel() {
  const [tab, setTab] = useState<Tab>("orders");
  const [orders, setOrders] = useState<Order[]>([]);
  const [couriers, setCouriers] = useState<Courier[]>([]);
  const [menu, setMenu] = useState<MenuItem[]>([]);
  const [earnings, setEarnings] = useState<Earnings | null>(null);
  const [error, setError] = useState("");

  const load = async () => {
    try {
      const [orderData, courierData, menuData, earningsData] = await Promise.all([
        fetchRestaurantOrders(), fetchCouriers(), fetchRestaurantMenu(), fetchRestaurantEarnings(),
      ]);
      setOrders(orderData);
      setCouriers(courierData);
      setMenu(menuData);
      setEarnings(earningsData);
      setError("");
    } catch {
      setError("Panel verileri yüklenemedi");
    }
  };

  useEffect(() => { void load(); }, []);

  async function assign(orderId: number, courierId: string) {
    if (!courierId) return;
    await assignCourier(orderId, Number(courierId));
    await load();
  }

  return (
    <div className="panel-body">
      {earnings && (
        <section className="stats-grid" aria-label="Kazanç özeti">
          <article className="stat-card">
            <span>Toplam gelir</span>
            <strong>{earnings.total_revenue.toFixed(2)} TL</strong>
          </article>
          <article className="stat-card">
            <span>Onaylanmış gelir</span>
            <strong>{earnings.confirmed_revenue.toFixed(2)} TL</strong>
          </article>
          <article className="stat-card">
            <span>Bekleyen gelir</span>
            <strong>{earnings.pending_revenue.toFixed(2)} TL</strong>
          </article>
          <article className="stat-card">
            <span>Ortalama sipariş</span>
            <strong>{earnings.average_order_value.toFixed(2)} TL</strong>
          </article>
          <article className="stat-card">
            <span>Sipariş sayısı</span>
            <strong>{earnings.order_count}</strong>
          </article>
          <article className="stat-card">
            <span>Onaylanan sipariş</span>
            <strong>{earnings.confirmed_order_count}</strong>
          </article>
        </section>
      )}

      <div className="tabs">
        {(["orders", "menu", "couriers"] as Tab[]).map((value) => (
          <button key={value} className={tab === value ? "active" : ""} onClick={() => setTab(value)}>
            {{ orders: "Siparişler", menu: "Menü", couriers: "Kuryeler" }[value]}
          </button>
        ))}
      </div>
      {error && <div className="status error">{error}</div>}

      {tab === "orders" && (
        !orders.length ? <div className="empty-state"><strong>Gelen sipariş yok</strong><p>Yeni siparişler burada görünür.</p></div> : (
          <div className="panel-list">
            {orders.map((order) => (
              <article key={order.id} className="panel-card">
                <div className="order-head">
                  <strong>#{order.id} {order.customer_email ?? "Misafir"}</strong>
                  <span className="status-pill">{statusLabel[order.status] ?? order.status}</span>
                </div>
                <div className="order-meta">
                  <span>{new Date(order.created_at).toLocaleString("tr-TR")}</span>
                  <span>{order.total.toFixed(2)} TL</span>
                </div>
                <p className="muted">{order.address}</p>
                <div className="order-items">
                  {order.items.map((item) => <span key={item.id}>{item.quantity} x {item.name}</span>)}
                </div>
                {order.status === "hazırlanıyor" && (
                  <label className="assign-field">
                    Kurye ata
                    <select defaultValue="" onChange={(event) => assign(order.id, event.target.value)}>
                      <option value="">Kurye seçin</option>
                      {couriers.map((courier) => <option key={courier.id} value={courier.id}>{courier.name}</option>)}
                    </select>
                  </label>
                )}
                {order.courier_name && <p className="muted">Kurye: {order.courier_name}</p>}
              </article>
            ))}
          </div>
        )
      )}

      {tab === "menu" && (
        <>
          <div className="panel-list">
            {menu.map((item) => (
              <article key={item.id} className="panel-card">
                <strong>{item.name}</strong>
                <p className="muted">{item.category} · {item.price.toFixed(2)} TL</p>
                <p className="muted">{item.description}</p>
              </article>
            ))}
          </div>
          <form
            className="form-card"
            onSubmit={async (event) => {
              event.preventDefault();
              const form = event.currentTarget;
              const data = new FormData(form);
              const created = await addMenuItem({
                name: String(data.get("name")), description: String(data.get("description")),
                price: Number(data.get("price")), category: String(data.get("category")),
              });
              setMenu((current) => [...current, created]);
              form.reset();
            }}
          >
            <h2>Menüye ürün ekle</h2>
            <input name="name" placeholder="Ürün adı" required />
            <textarea name="description" placeholder="Açıklama" required />
            <input name="price" type="number" min="1" step="0.01" placeholder="Fiyat" required />
            <input name="category" placeholder="Kategori" required />
            <button className="primary-button">Ürünü ekle</button>
          </form>
        </>
      )}

      {tab === "couriers" && (
        <form
          className="form-card"
          onSubmit={async (event) => {
            event.preventDefault();
            const form = event.currentTarget;
            const data = new FormData(form);
            const created = await addCourier(String(data.get("email")), String(data.get("password")), String(data.get("name")));
            setCouriers((current) => [...current, created]);
            form.reset();
          }}
        >
          <h2>Kurye hesabı ekle</h2>
          <input name="name" placeholder="Kurye adı" required />
          <input name="email" type="email" placeholder="E-posta" required />
          <input name="password" type="password" placeholder="Şifre" minLength={6} required />
          <button className="primary-button">Kuryeyi ekle</button>
        </form>
      )}
    </div>
  );
}