import { Link } from "react-router-dom";
import { useAuth } from "../AuthContext";
import CourierPanel from "../panels/CourierPanel";
import CustomerPanel from "../panels/CustomerPanel";
import RestaurantPanel from "../panels/RestaurantPanel";

export default function Panel() {
  const { user, ready, logout } = useAuth();

  if (!ready) return <div className="status">Yükleniyor...</div>;
  if (!user) {
    return (
      <div className="empty-state">
        <strong>Panele girmek için giriş yapın</strong>
        <p>Hesabınıza girerek siparişlerinizi veya işletme panelinizi görebilirsiniz.</p>
        <Link className="secondary-button" to="/giris">Giriş sayfasına git</Link>
      </div>
    );
  }

  return (
    <section className="page-section">
      <div className="section-head">
        <div>
          <h1>Panelim</h1>
          <p className="muted">{user.email} · {user.role}</p>
        </div>
        <button className="ghost-button" onClick={logout}>Çıkış yap</button>
      </div>
      {user.role === "customer" && <CustomerPanel />}
      {user.role === "restaurant" && <RestaurantPanel />}
      {user.role === "courier" && <CourierPanel />}
    </section>
  );
}