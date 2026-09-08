import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { fetchCuisines, fetchRestaurants, type RestaurantFilters } from "../api";
import type { Restaurant } from "../types";

const formatTL = (value: number) => `${value.toFixed(2)} TL`;

const emptyFilters: RestaurantFilters = {
  query: "",
  cuisine: "",
  minRating: 0,
  maxDeliveryFee: 999,
  maxEtaMinutes: 999,
  sort: "rating",
};

export default function Restaurants() {
  const [restaurants, setRestaurants] = useState<Restaurant[]>([]);
  const [cuisines, setCuisines] = useState<string[]>([]);
  const [filters, setFilters] = useState<RestaurantFilters>(emptyFilters);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    fetchCuisines()
      .then(setCuisines)
      .catch(() => setError("Mutfaklar yüklenemedi"));
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => {
      setLoading(true);
      fetchRestaurants(filters)
        .then((data) => {
          setRestaurants(data);
          setError("");
        })
        .catch(() => setError("Restoranlar yüklenemedi"))
        .finally(() => setLoading(false));
    }, 180);
    return () => clearTimeout(timer);
  }, [filters]);

  const update = (values: Partial<RestaurantFilters>) =>
    setFilters((current) => ({ ...current, ...values }));

  const cards = useMemo(
    () =>
      restaurants.map((restaurant) => (
        <Link to={`/r/${restaurant.id}`} key={restaurant.id} className="restaurant-card">
          <div className="restaurant-avatar" aria-hidden="true">
            {restaurant.image_emoji}
          </div>
          <div className="restaurant-body">
            <div className="restaurant-title">
              <h3>{restaurant.name}</h3>
              <span className="rating">{restaurant.rating.toFixed(1)}</span>
            </div>
            <p className="cuisine-label">{restaurant.cuisine}</p>
            <div className="restaurant-facts">
              <span>{restaurant.eta_minutes} dk teslim</span>
              <span>Teslimat {formatTL(restaurant.delivery_fee)}</span>
              <span>Min. {formatTL(restaurant.min_order)}</span>
            </div>
            <span className="restaurant-cta">Menüyü gör</span>
          </div>
        </Link>
      )),
    [restaurants],
  );

  const filtersActive =
    filters.query !== "" || filters.cuisine !== "" || filters.minRating !== 0 ||
    filters.maxDeliveryFee !== 999 || filters.maxEtaMinutes !== 999;

  return (
    <section className="home-page">
      <div className="page-hero">
        <div>
          <span className="eyebrow">Sepet</span>
          <h1>Ne yemek istersin?</h1>
          <p>Aramanı yaz, mutfak seç ve teslim süresi ile bütçene göre restoranları daralt.</p>
        </div>
        <div className="hero-stats">
          <strong>{restaurants.length}</strong>
          <span>restoran bulundu</span>
        </div>
      </div>

      <form className="filter-bar" onSubmit={(event) => event.preventDefault()}>
        <label className="search-field">
          <span>Arama</span>
          <input
            value={filters.query}
            onChange={(event) => update({ query: event.target.value })}
            placeholder="Restoran, mutfak veya yemek ara"
            aria-label="Restoran ara"
          />
        </label>
        <label className="filter-field">
          <span>Puan</span>
          <select value={filters.minRating} onChange={(event) => update({ minRating: Number(event.target.value) })}>
            <option value={0}>Tümü</option>
            <option value={4.5}>4.5+</option>
            <option value={4.2}>4.2+</option>
            <option value={4}>4.0+</option>
          </select>
        </label>
        <label className="filter-field">
          <span>Süre</span>
          <select value={filters.maxEtaMinutes} onChange={(event) => update({ maxEtaMinutes: Number(event.target.value) })}>
            <option value={999}>Tümü</option>
            <option value={25}>25 dk</option>
            <option value={30}>30 dk</option>
            <option value={40}>40 dk</option>
          </select>
        </label>
        <label className="filter-field">
          <span>Teslimat</span>
          <select value={filters.maxDeliveryFee} onChange={(event) => update({ maxDeliveryFee: Number(event.target.value) })}>
            <option value={999}>Tümü</option>
            <option value={10}>10 TL altı</option>
            <option value={15}>15 TL altı</option>
            <option value={20}>20 TL altı</option>
          </select>
        </label>
        <label className="filter-field">
          <span>Sıralama</span>
          <select value={filters.sort} onChange={(event) => update({ sort: event.target.value as RestaurantFilters["sort"] })}>
            <option value="rating">Puana göre</option>
            <option value="eta">Süreye göre</option>
            <option value="delivery_fee">Teslimat ücreti</option>
            <option value="min_order">Minimum sepet</option>
          </select>
        </label>
      </form>

      <div className="chips" aria-label="Mutfak filtresi">
        <button className={filters.cuisine === "" ? "chip active" : "chip"} onClick={() => update({ cuisine: "" })}>
          Tüm mutfaklar
        </button>
        {cuisines.map((item) => (
          <button
            key={item}
            className={filters.cuisine === item ? "chip active" : "chip"}
            onClick={() => update({ cuisine: item })}
          >
            {item}
          </button>
        ))}
      </div>

      {filtersActive && (
        <div className="filter-summary">
          <span>Filtreli sonuç</span>
          <button className="link-button" onClick={() => setFilters(emptyFilters)}>Filtreleri temizle</button>
        </div>
      )}

      {loading && <div className="status">Restoranlar yükleniyor...</div>}
      {!loading && error && <div className="status error">{error}</div>}
      {!loading && !error && restaurants.length === 0 && (
        <div className="empty-state">
          <strong>Aramanla eşleşen restoran yok</strong>
          <p>Süreyi, teslimat ücretini veya mutfak filtresini değiştirerek tekrar dene.</p>
          <button onClick={() => setFilters(emptyFilters)}>Tüm restoranları göster</button>
        </div>
      )}
      <div className="restaurant-grid">{cards}</div>
    </section>
  );
}