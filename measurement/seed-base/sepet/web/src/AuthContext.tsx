import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { fetchMe, login as loginRequest, registerAccount } from "./api";
import type { User } from "./types";

type AuthState = {
  user: User | null;
  ready: boolean;
  login: (email: string, password: string) => Promise<void>;
  register: (payload: Record<string, unknown>) => Promise<void>;
  logout: () => void;
  refreshUser: () => Promise<void>;
};

const AuthContext = createContext<AuthState | null>(null);
const STORAGE_KEY = "sepet-auth";

function readStored(): { token?: string; user?: User } {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");
  } catch {
    return {};
  }
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(() => readStored().user ?? null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const stored = readStored();
    if (!stored.token) {
      setReady(true);
      return;
    }
    fetchMe()
      .then(setUser)
      .catch(() => setUser(null))
      .finally(() => setReady(true));
  }, []);

  const persist = (token: string, nextUser: User) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ token, user: nextUser }));
    setUser(nextUser);
  };

  const login = useCallback(async (email: string, password: string) => {
    const result = await loginRequest(email, password);
    persist(result.token, result.user);
  }, []);

  const register = useCallback(async (payload: Record<string, unknown>) => {
    const result = await registerAccount(payload);
    persist(result.token, result.user);
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem(STORAGE_KEY);
    setUser(null);
  }, []);

  const refreshUser = useCallback(async () => {
    const nextUser = await fetchMe();
    const stored = readStored();
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ ...stored, user: nextUser }));
    setUser(nextUser);
  }, []);

  const value = useMemo(
    () => ({ user, ready, login, register, logout, refreshUser }),
    [login, logout, ready, refreshUser, register, user],
  );
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) throw new Error("useAuth AuthProvider icinde kullanilmali");
  return context;
}