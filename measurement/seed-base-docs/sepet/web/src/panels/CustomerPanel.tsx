import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { confirmDelivery, fetchHistory, topUpWallet } from "../api";
import { useAuth } from "../AuthContext";
import { Order } from "../types";

const statusLabel: Record<string, string> = {
  "hazırlanıyor": "Hazırlanıyor",
  kuryede: "Kuryede",
  teslim_edildi: "Teslim Edildi",
  "onaylandı": "Onaylandı",
};

export default function CustomerPanel() {
  const { user, refreshUser } = useAuth();
  const [orders, setOrders] = useState<Order[]>([]);
  const [amount, setAmount] = useState("100");
  const [walletMessage, setWalletMessage] = useState("");
  const [walletError, setWalletError] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const load = () => {
    setLoading(true);
    fetchHistory()
      .then(setOrders)
      .catch(() => setError("Sipariş geçmişi yüklenemedi"))
      .finally(() => setLoading(false));
  };

  useEffect(load, []);

  async function confirm(id: number) {
    await confirmDelivery(id);
    load();
  }

  async function addBalance(event: React.FormEvent) {
    event.preventDefault();
    setWalletMessage("");
    setWalletError("");
    try {
      const result = await topUpWallet(Number(amount));
      await refreshUser();
      setWalletMessage(`${result.added.toFixed(2)} TL bakiye eklendi.`);
    } catch (requestError) {
      setWalletError(requestError instanceof Error ? requestError.message : "Bakiye yüklenemedi");
    }
  }

  if (loading) return <div className="status">Siparişler yükleniyor...</div>;
  if (error) return <div className="status error">{error}</div>;

  return (
    <div className="panel-body">
      <section className="form-card wallet-card">
        <div className="wallet-head">
          <div>
            <span className="eyebrow">Cüzdan</span>
            <h2>{(user?.balance ?? 0).toFixed(2)} TL</h2>
            <p className="muted">Sipariş toplamı bakiyeden düşülür.</p>
          </div>
          <span aria-hidden="true" className="wallet-icon">₺</span>
        </div>
        <form className="wallet-form" onSubmit={addBalance}>
          <label>
            Yüklenecek tutar
            <input
              value={amount}
              onChange={(event) => setAmount(event.target.value)}
              type="number"
              min="1"
              max="1000"
              step="0.01"
              required
            />
          </label>
          <button className="primary-button">Bakiye yükle</button>
        </form>
        {walletMessage && <div className="status success">{walletMessage}</div>}
        {walletError && <div className="status error">{walletError}</div>}
      </section>

      {!orders.length ? (
        <div className="empty-state">
          <strong>Henüz siparişiniz yok</strong>
          <p>Restoran seçip ilk siparişinizi verebilirsiniz.</p>
          <Link className="secondary-button" to="/">Sipariş ver</Link>
        </div>
      ) : (
        <div className="panel-list">
          {orders.map((order) => (
            <article key={order.id} className="panel-card">
              <div className="order-head">
                <strong>#{order.id} {order.restaurant_name}</strong>
                <span className="status-pill">{statusLabel[order.status] ?? order.status}</span>
              </div>
              <div className="order-meta">
                <span>{new Date(order.created_at).toLocaleString("tr-TR")}</span>
                <span>{order.total.toFixed(2)} TL</span>
              </div>
              <div className="order-items">
                {order.items.map((item) => <span key={item.id}>{item.quantity} x {item.name}</span>)}
              </div>
              <div className="order-actions">
                <Link className="link-button" to={`/order/${order.id}`}>Detay</Link>
                {order.status === "teslim_edildi" && (
                  <button className="secondary-button" onClick={() => confirm(order.id)}>Teslimi onayla</button>
                )}
              </div>
            </article>
          ))}
        </div>
      )}
    </div>
  );
}