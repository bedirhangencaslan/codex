export type Restaurant = {
  id: number;
  name: string;
  cuisine: string;
  rating: number;
  delivery_fee: number;
  min_order: number;
  eta_minutes: number;
  image_emoji: string;
  menu_items?: MenuItem[];
  reviews?: Review[];
  can_review?: boolean;
  has_review?: boolean;
};

export type MenuItem = {
  id: number;
  restaurant_id: number;
  name: string;
  description: string;
  price: number;
  category: string;
};

export type OrderItem = {
  id: number;
  menu_item_id: number;
  name: string;
  unit_price: number;
  quantity: number;
  line_total?: number;
};

export type Order = {
  id: number;
  restaurant_name?: string;
  restaurant_id: number;
  address: string;
  phone: string;
  note: string;
  total: number;
  status: string;
  created_at: string;
  customer_email?: string | null;
  courier_name?: string | null;
  items: OrderItem[];
};

export type User = {
  id: number;
  email: string;
  role: "customer" | "restaurant" | "courier";
  balance?: number;
};

export type Earnings = {
  total_revenue: number;
  confirmed_revenue: number;
  pending_revenue: number;
  order_count: number;
  confirmed_order_count: number;
  average_order_value: number;
};
export type Review = {
  id: number;
  rating: number;
  comment: string;
  created_at: string;
  user_email: string;
};

export type Courier = {
  id: number;
  name: string;
  restaurant_id: number;
};

export type CartLine = {
  item: MenuItem;
  quantity: number;
  deliveryFee: number;
};
