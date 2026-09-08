import { FormEvent, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../AuthContext";

export default function Account() {
  const { login, register } = useAuth();
  const navigate = useNavigate();
  const [mode, setMode] = useState<"login" | "register">("login");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [sending, setSending] = useState(false);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSending(true);
    setError("");
    try {
      if (mode === "login") await login(email, password);
      else await register({ email, password, role: "customer" });
      navigate("/panel");
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : "İşlem tamamlanamadı");
    } finally {
      setSending(false);
    }
  }

  return (
    <section className="page-section">
      <div className="section-head">
        <h1>Hesabın</h1>
        <p>Müşteri girişi yapabilir veya yeni hesap oluşturabilirsin.</p>
      </div>
      <div className="account-layout">
        <form className="form-card" onSubmit={submit}>
          <h2>{mode === "login" ? "Giriş yap" : "Müşteri kaydı"}</h2>
          <label>E-posta<input type="email" value={email} onChange={(event) => setEmail(event.target.value)} required /></label>
          <label>Şifre<input type="password" minLength={6} value={password} onChange={(event) => setPassword(event.target.value)} required /></label>
          {error && <p className="status error">{error}</p>}
          <button className="primary-button" disabled={sending}>
            {sending ? "Gönderiliyor..." : mode === "login" ? "Giriş yap" : "Kayıt ol"}
          </button>
          <button type="button" className="link-button" onClick={() => setMode(mode === "login" ? "register" : "login")}>
            {mode === "login" ? "Hesabın yok mu? Kayıt ol" : "Hesabın var mı? Giriş yap"}
          </button>
        </form>

        <aside className="form-card">
          <h2>Restoran hesabı</h2>
          <p className="muted">İşletmen için restoran ve kurye yönetimi hesabı oluşturun.</p>
          <form
            className="nested-form"
            onSubmit={async (event) => {
              event.preventDefault();
              const form = event.currentTarget;
              const data = new FormData(form);
              setSending(true);
              setError("");
              try {
                await register(Object.fromEntries(data.entries()));
                navigate("/panel");
              } catch (requestError) {
                setError(requestError instanceof Error ? requestError.message : "Kayıt tamamlanamadı");
              } finally {
                setSending(false);
              }
            }}
          >
            <input name="email" type="email" placeholder="İşletme e-postası" required />
            <input name="password" type="password" placeholder="Şifre" minLength={6} required />
            <input name="restaurant_name" placeholder="Restoran adı" required />
            <input name="cuisine" placeholder="Mutfak (pizza, kebap...)" required />
            <input type="hidden" name="role" value="restaurant" />
            <button className="primary-button" disabled={sending}>Restoran hesabı oluştur</button>
          </form>
          <div className="demo-accounts">
            <strong>Demo hesaplar</strong>
            <span>musteri@sepet.test / sepet123</span>
            <span>sahip.anadolu-atesi@sepet.test / sepet123</span>
            <span>kurye1.anadolu-atesi@sepet.test / sepet123</span>
          </div>
        </aside>
      </div>
    </section>
  );
}