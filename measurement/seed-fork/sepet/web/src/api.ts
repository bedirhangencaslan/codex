import type { CartLine, Courier, Earnings, MenuItem, Order, OrderItem, Restaurant, Review, User } from "./types";

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const stored = localStorage.getItem("sepet-auth");
  const token = stored ? (JSON.parse(stored) as { token?: string }).token : undefined;
  const response = await fetch(path, {
    headers: {
      ...(options?.body ? { "Content-Type": "application/json" } : {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    ...options,
  });
  if (!response.ok) {
    const message = await response
      .json()
      .then((data) => data?.detail)
      .catch(() => undefined);
    throw new Error(typeof message === "string" ? message : "Ä°ÅŸlem tamamlanamadÄ±");
  }
  return response.json() as Promise<T>;
}

export type RestaurantFilters = {
  query: string;
  cuisine: string;
  minRating: number;
  maxDeliveryFee: number;
  maxEtaMinutes: number;
  sort: "rating" | "eta" | "delivery_fee" | "min_order";
};

export const fetchRestaurants = (filters: RestaurantFilters) => {
  const params = new URLSearchParams();
  if (filters.query) params.set("q", filters.query);
  if (filters.cuisine) params.set("cuisine", filters.cuisine);
  if (filters.minRating) params.set("min_rating", String(filters.minRating));
  if (filters.maxDeliveryFee !== 999) params.set("max_delivery_fee", String(filters.maxDeliveryFee));
  if (filters.maxEtaMinutes !== 999) params.set("max_eta_minutes", String(filters.maxEtaMinutes));
  params.set("sort", filters.sort);
  const suffix = params.size ? `?${params.toString()}` : "";
  return request<Restaurant[]>(`/api/restaurants${suffix}`);
};

export const fetchRestaurant = (id: number) =>
  request<Restaurant>(`/api/restaurants/${id}`);

export const fetchCuisines = () => request<string[]>("/api/cuisines");

export const fetchOrder = (id: number) => request<Order>(`/api/orders/${id}`);

export const createOrder = (restaurantId: number, address: string, phone: string, note: string, items: CartLine[]) =>
  request<{ id: number }>("/api/orders", {
    method: "POST",
    body: JSON.stringify({
      restaurant_id: restaurantId,
      address,
      phone,
      note,
      items: items.map((line) => {
        const input: Pick<OrderItem, "menu_item_id" | "quantity"> = {
          menu_item_id: line.item.id,
          quantity: line.quantity,
        };
        return input;
      }),
    }),
  });

export const login = (email: string, password: string) =>
  request<{ token: string; user: User }>("/api/auth/login", {
    method: "POST",
    body: JSON.stringify({ email, password }),
  });

export const registerAccount = (payload: Record<string, unknown>) =>
  request<{ token: string; user: User }>("/api/auth/register", {
    method: "POST",
    body: JSON.stringify(payload),
  });

export const fetchMe = () => request<User>("/api/auth/me");

export const fetchHistory = () => request<Order[]>("/api/orders/history");
export const confirmDelivery = (id: number) =>
  request<{ status: string }>(`/api/orders/${id}/confirm_delivery`, { method: "POST" });

export const fetchRestaurantOrders = () => request<Order[]>("/api/restaurant/orders");
export const fetchRestaurantMenu = () => request<MenuItem[]>("/api/restaurant/menu-items");
export const fetchCouriers = () => request<Courier[]>("/api/restaurant/couriers");
export const addCourier = (email: string, password: string, name: string) =>
  request<Courier>("/api/restaurant/couriers", {
    method: "POST",
    body: JSON.stringify({ email, password, name }),
  });
export const addMenuItem = (payload: Omit<MenuItem, "id" | "restaurant_id">) =>
  request<MenuItem>("/api/restaurant/menu-items", {
    method: "POST",
    body: JSON.stringify(payload),
  });
export const assignCourier = (orderId: number, courierId: number) =>
  request<{ status: string }>(`/api/restaurant/orders/${orderId}/assign-courier`, {
    method: "POST",
    body: JSON.stringify({ courier_id: courierId }),
  });

export const fetchCourierOrders = () => request<Order[]>("/api/courier/orders");
export const deliverOrder = (id: number) =>
  request<{ status: string }>(`/api/courier/orders/${id}/deliver`, { method: "POST" });

export const submitReview = (restaurantId: number, rating: number, comment: string) =>
  request<Review>(`/api/restaurants/${restaurantId}/reviews`, {
    method: "POST",
    body: JSON.stringify({ rating, comment }),
  });


export const topUpWallet = (amount: number) =>
  request<{ balance: number; added: number }>("/api/auth/wallet/topup", {
    method: "POST",
    body: JSON.stringify({ amount }),
  });

export const fetchRestaurantEarnings = () => request<Earnings>("/api/restaurant/earnings");