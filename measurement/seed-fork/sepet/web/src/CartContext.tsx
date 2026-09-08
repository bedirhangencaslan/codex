import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import type { CartLine, MenuItem } from "./types";

type CartState = {
  lines: CartLine[];
  restaurantId: number | null;
  subtotal: number;
  add: (item: MenuItem, deliveryFee: number) => void;
  change: (itemId: number, delta: number) => void;
  clear: () => void;
};

const CartContext = createContext<CartState | null>(null);
const STORAGE_KEY = "sepet-cart-v1";

type StoredCart = {
  restaurantId: number | null;
  lines: CartLine[];
};

export function CartProvider({ children }: { children: ReactNode }) {
  const [cart, setCart] = useState<StoredCart>(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      return stored ? JSON.parse(stored) as StoredCart : { restaurantId: null, lines: [] };
    } catch {
      return { restaurantId: null, lines: [] };
    }
  });

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(cart));
  }, [cart]);

  const add = useCallback((item: MenuItem, deliveryFee: number) => {
    setCart((current) => {
      if (!current.restaurantId || current.restaurantId !== item.restaurant_id) {
        return { restaurantId: item.restaurant_id, lines: [{ item, quantity: 1, deliveryFee }] };
      }
      const lines = current.lines.map((line) =>
        line.item.id === item.id ? { ...line, quantity: line.quantity + 1 } : line,
      );
      if (!lines.some((line) => line.item.id === item.id)) {
      lines.push({ item, quantity: 1, deliveryFee });
      }
      return { ...current, lines };
    });
  }, []);

  const change = useCallback((itemId: number, delta: number) => {
    setCart((current) => {
      const lines = current.lines
        .map((line) => line.item.id === itemId ? { ...line, quantity: line.quantity + delta } : line)
        .filter((line) => line.quantity > 0);
      return lines.length ? { ...current, lines } : { restaurantId: null, lines: [] };
    });
  }, []);

  const clear = useCallback(() => setCart({ restaurantId: null, lines: [] }), []);

  const value = useMemo<CartState>(() => {
    const subtotal = cart.lines.reduce((sum, line) => sum + line.item.price * line.quantity, 0);
    return {
      lines: cart.lines,
      restaurantId: cart.restaurantId,
      subtotal,
      add,
      change,
      clear,
    };
  }, [add, cart, change, clear]);

  return <CartContext.Provider value={value}>{children}</CartContext.Provider>;
}

export function useCart() {
  const context = useContext(CartContext);
  if (!context) throw new Error("useCart CartProvider icinde kullanilmali");
  return context;
}
