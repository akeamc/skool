import {
  createContext,
  FunctionComponent,
  useCallback,
  useContext,
  useEffect,
  useState,
} from "react";

async function login(
  username?: string,
  password?: string,
  retries = 3
): Promise<void> {
  const req = { username, password };
  const hasBody = Object.keys(req).length > 0;

  const res = await fetch("http://localhost:8000/login", {
    method: "POST",
    body: hasBody ? JSON.stringify(req) : undefined,
    credentials: "include",
    headers: hasBody
      ? {
          "Content-Type": "application/json",
        }
      : undefined,
  });

  if (res.status === 400) {
    throw new Error(await res.text());
  }

  if (res.status % 100 === 5 && retries > 0) {
    return login(username, password, retries - 1);
  }
}

export interface AuthData {
  authenticated: boolean;
  loading: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthData>({
  authenticated: false,
  loading: false,
  login: async () => {},
  logout: async () => {},
});

export const AuthProvider: FunctionComponent = ({ children }) => {
  const [authenticated, setAuthenticated] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!authenticated) {
      login()
        .then(() => setAuthenticated(true))
        .catch(console.error)
        .finally(() => setLoading(false));
    }
  }, [authenticated]);

  const bruhLogin = useCallback(async (username: string, password: string) => {
    setLoading(true);

    await login(username, password);

    setAuthenticated(true);
    setLoading(false);
  }, []);

  const bruhLogout = useCallback(async () => {
    await fetch("http://localhost:8000/logout", { credentials: "include" });

    setAuthenticated(false);
  }, []);

  return (
    <AuthContext.Provider
      value={{ authenticated, loading, login: bruhLogin, logout: bruhLogout }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => useContext(AuthContext);
