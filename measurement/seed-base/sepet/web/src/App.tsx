import { Link, Route, Routes, useNavigate } from "react-router-dom";
import { useAuth } from "./AuthContext";
import Checkout from "./pages/Checkout";
import Account from "./pages/Account";
import OrderPage from "./pages/OrderPage";
import Panel from "./pages/Panel";
import RestaurantDetail from "./pages/RestaurantDetail";
import Restaurants from "./pages/Restaurants";

export default function App() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  return (
    <div className="shell">
      <header className="site-header">
        <div className="header-inner">
          <Link to="/" className="brand" aria-label="Sepet ana sayfa">
            <span aria-hidden="true">S</span>
            Sepet
          </Link>
          <nav className="site-nav" aria-label="Ana menü">
            <Link className="nav-link" to="/">Restoranlar</Link>
            {user ? (
              <>
                {user.role === "customer" && typeof user.balance === "number" && (
                  <span className="balance-pill">{user.balance.toFixed(2)} TL</span>
                )}
                <Link className="nav-link" to="/panel">Panelim</Link>
                <button
                  className="ghost-button"
                  onClick={() => {
                    logout();
                    navigate("/");
                  }}
                >
                  Çıkış
                </button>
              </>
            ) : (
              <Link className="primary-link" to="/giris">Giriş / Kayıt</Link>
            )}
          </nav>
        </div>
      </header>
      <main className="page">
        <Routes>
          <Route path="/" element={<Restaurants />} />
          <Route path="/r/:id" element={<RestaurantDetail />} />
          <Route path="/checkout" element={<Checkout />} />
          <Route path="/order/:id" element={<OrderPage />} />
          <Route path="/giris" element={<Account />} />
          <Route path="/panel" element={<Panel />} />
        </Routes>
      </main>
    </div>
  );
}