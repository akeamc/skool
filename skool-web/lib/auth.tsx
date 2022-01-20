import {
  createContext,
  FunctionComponent,
  useCallback,
  useContext,
  useEffect,
  useState,
} from "react";
import useSWR from "swr";
import createPersistedState from "use-persisted-state";
import { API_ENDPOINT } from "./api";
import { retryRequest } from "./retry";

async function login(username: string, password: string): Promise<void> {
  const req = { username, password };

  const res = await retryRequest(
    fetch(`${API_ENDPOINT}/auth/login`, {
      method: "POST",
      body: JSON.stringify(req),
      headers: {
            "Content-Type": "application/json",
          }
    })
  );

  if (!res.ok) {
    throw new Error(await res.text());
  }
}

async function getSessionToken(refreshToken: string): Promise<string> {
  const res = await retryRequest(
    fetch(`${API_ENDPOINT}/auth/session`, {
      method: "POST",
      body: JSON.stringify({ refresh_token: refreshToken }),
      headers: {
        "Content-Type": "application/json",
      },
    })
  );

  if (!res.ok) {
    throw new Error(await res.text());
  }

  const { session_token } = await res.json();
  return session_token;
}

export interface AuthData {
  authenticated: boolean;
  loading: boolean;
  loggedOut: boolean;
  sessionToken?: string;
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

const useRefreshTokenState = createPersistedState("refresh_token");

export const AuthProvider: FunctionComponent = ({ children }) => {
  const [authenticated, setAuthenticated] = useState(false);
  const [loggedOut, setLoggedOut] = useState(false);
  const [loading, setLoading] = useState(true);
  const [refreshToken, setRefreshToken] = useRefreshTokenState<string>();
  const [sessionToken, setSessionToken] = useState<string>();

  useEffect(() => {
    if (refreshToken) {
      getSessionToken(refreshToken).then(setSessionToken);
    }
  }, [refreshToken]);

  const bruhLogin = async (username: string, password: string) => {
    setLoading(true);

    await login(username, password)
      .then(() => {
        setAuthenticated(true);
      })
      .finally(() => setLoading(false));
  };

  const bruhLogout = useCallback(async () => {
    await fetch(`${API_ENDPOINT}/auth/logout`, {
      credentials: "include",
    });

    setAuthenticated(false);
    setLoggedOut(true);
  }, []);

  return (
    <AuthContext.Provider
      value={{
        authenticated,
        loading,
        loggedOut,
        sessionToken,
        login: bruhLogin,
        logout: bruhLogout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => useContext(AuthContext);

export const useToken = () => {
  const { authenticated } = useAuth();

  return useSWR(authenticated ? `/token` : null, async () => {
    const res = await fetch(`${API_ENDPOINT}/auth/token`, {
      credentials: "include",
    });

    const text = await res.text();

    if (!res.ok) {
      throw new Error(text);
    }

    return text;
  });
};
