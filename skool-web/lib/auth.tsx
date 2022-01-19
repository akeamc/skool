import {
  createContext,
  FunctionComponent,
  useCallback,
  useContext,
  useEffect,
  useState,
} from "react";
import useSWR from "swr";
import { retryRequest } from "./retry";

async function login(
  username?: string,
  password?: string,
): Promise<void> {
  const req = { username, password };
  const hasBody = Object.keys(req).length > 0;

  const res = await retryRequest(fetch("http://localhost:8000/auth/login", {
    method: "POST",
    body: hasBody ? JSON.stringify(req) : undefined,
    credentials: "include",
    headers: hasBody
      ? {
          "Content-Type": "application/json",
        }
      : undefined,
  }));

  if (!res.ok) {
    throw new Error(await res.text());
  }
}

export interface AuthData {
  authenticated: boolean;
  loading: boolean;
  loggedOut: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthData>({
  authenticated: false,
  loading: false,
  loggedOut: true,
  login: async () => {},
  logout: async () => {},
});

export const AuthProvider: FunctionComponent = ({ children }) => {
  const [authenticated, setAuthenticated] = useState(false);
  const [loggedOut, setLoggedOut] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!authenticated && !loggedOut) {
      login()
        .then(() => setAuthenticated(true))
        .catch(console.error)
        .finally(() => setLoading(false));
    }
  }, [authenticated, loggedOut]);

  const bruhLogin = useCallback(async (username: string, password: string) => {
    setLoading(true);

    await login(username, password);

    setAuthenticated(true);
    setLoading(false);
  }, []);

  const bruhLogout = useCallback(async () => {
    await fetch("http://localhost:8000/auth/logout", {
      credentials: "include",
    });

    setAuthenticated(false);
    setLoggedOut(true);
  }, []);

  return (
    <AuthContext.Provider
      value={{ authenticated, loading, loggedOut, login: bruhLogin, logout: bruhLogout }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => useContext(AuthContext);

export const useToken = () => {
  const {authenticated} = useAuth();

  return useSWR(authenticated ? `/token` : null, async () => {
    const res = await fetch("http://localhost:8000/auth/token", {
      credentials: "include",
    });

    const text = await res.text();

    if (!res.ok) {
      throw new Error(text);
    }

    return text;
  });
}
