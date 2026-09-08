import { FormEvent, useState } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { createOrder } from "../api";
import { useAuth } from "../AuthContext";
import { useCart } from "../CartContext";

const formatTL = (value: number) => `${value.toFixed(2)} TL`;

export default function Checkout() {
  const cart = useCart();
  const { user } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const deliveryFee = (location.state?.deliveryFee as number | undefined) ?? cart.lines[0]?.deliveryFee ?? 0;
  const [address, setAddress] = useState("");
  const [phone, setPhone] = useState("");
  const [note, setNote] = useState("");
  const [error, setError] = useState("");
  const [sending, setSending] = useState(false);
  const total = cart.subtotal + deliveryFee;
  const missingBalance = Math.max(0, total - (user?.balance ?? 0));

  if (!cart.lines.length) {
    return (
      <div className="empty-state">
        <strong>Sepetiniz boş</strong>
        <p>Sipariş vermek için bir restoran seçip menüden ürün ekleyin.</p>
        <Link className="secondary-button" to="/">Restoranlara dön</Link>
      </div>
    );
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    const digits = phone.replace(/\D/g, "");
    if (address.trim().length < 10) {
      setError("Adres en az 10 karakter olmalıdır.");
      return;
    }
    if (digits.length < 10 || digits.length > 11) {
      setError("Telefon 10-11 hane olmalıdır.");
      return;
    }
    setSending(true);
    setError("");
    try {
      const result = await createOrder(cart.restaurantId!, address, digits, note, cart.lines);
      cart.clear();
      navigate(`/order/${result.id}`);
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : "Sipariş gönderilemedi");
    } finally {
      setSending(false);
    }
  }

  return (
    <section className="page-section">
      <div className="section-head">
        <h1>Teslimat</h1>
        <p>Siparişin doğru adrese gitsin.</p>
      </div>
      <div className="checkout-layout">
        <form onSubmit={submit} className="form-card">
          {!user && (
            <div className="status warning">
              Sipariş için giriş yapmalısınız. <Link className="link-button" to="/giris">Giriş yap</Link>
            </div>
          )}
          <label>Adres<textarea value={address} onChange={(event) => setAddress(event.target.value)} placeholder="Mahalle, sokak, bina ve daire" required /></label>
          <label>Telefon<input value={phone} onChange={(event) => setPhone(event.target.value)} inputMode="tel" placeholder="5xx xxx xx xx" required /></label>
          <label>Sipariş notu<textarea value={note} onChange={(event) => setNote(event.target.value)} placeholder="Kapı kodu, kat bilgisi veya özel istek" /></label>
          {error && <p className="status error">{error}</p>}
          <button className="primary-button" disabled={sending || !user || missingBalance > 0}>
            {sending ? "Gönderiliyor..." : "Siparişi onayla"}
          </button>
        </form>
        <aside className="cart-panel">
          <h2>Sipariş özeti</h2>
          {cart.lines.map((line) => (
            <div key={line.item.id} className="cart-line compact">
              <span>{line.item.name}</span>
              <strong>{formatTL(line.item.price * line.quantity)}</strong>
            </div>
          ))}
          <div className="totals"><span>Ara toplam</span><span>{formatTL(cart.subtotal)}</span></div>
          <div className="totals"><span>Teslimat</span><span>{formatTL(deliveryFee)}</span></div>
          <div className="totals strong"><span>Toplam</span><span>{formatTL(total)}</span></div>
          {user && typeof user.balance === "number" && (
            <div className="totals balance-line"><span>Bakiyeniz</span><span>{formatTL(user.balance)}</span></div>
          )}
          {missingBalance > 0 && (
            <div className="status warning">
              Bakiyeniz {formatTL(missingBalance)} eksik. <Link className="link-button" to="/panel">Bakiye yükle</Link>
            </div>
          )}
        </aside>
      </div>
    </section>
  );
}