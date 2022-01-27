import {
  createContext,
  FunctionComponent,
  useCallback,
  useContext,
  useEffect,
  useState,
} from "react";
import useSWR, { SWRResponse } from "swr";
import createPersistedState from "use-persisted-state";
import { API_ENDPOINT, Either } from "./api";

interface SessionFromUsernamePassword {
  username: string;
  password: string;
}

interface SessionFromLoginToken {
  login_token: string;
}

type CreateSessionRequest = Either<
  SessionFromUsernamePassword,
  SessionFromLoginToken
>;

interface CreateSessionResponse {
  session_token: string;
  login_token: string | null;
}

async function createSession(
  data: CreateSessionRequest
): Promise<CreateSessionResponse> {
  const res = await fetch(`${API_ENDPOINT}/auth/session`, {
    method: "POST",
    body: JSON.stringify(data),
    headers: {
      "Content-Type": "application/json",
    },
  });

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return res.json();
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

const LOGIN_TOKEN_KEY = "login_token";
const SESSION_TOKEN_KEY = "session_token";

const useLoginTokenState = createPersistedState(LOGIN_TOKEN_KEY);
const useSessionTokenState = createPersistedState(
  SESSION_TOKEN_KEY,
  typeof window !== "undefined" ? window.sessionStorage : undefined
);

export const AuthProvider: FunctionComponent = ({ children }) => {
  const [loggedOut, setLoggedOut] = useState(false);
  const [loading, setLoading] = useState(false);
  const [loginToken, setLoginToken] = useLoginTokenState<string>();
  const [sessionToken, setSessionToken] = useSessionTokenState<string>();

  useEffect(() => {
    if (loginToken && !sessionToken) {
      createSession({ login_token: loginToken })
        .then(({ session_token }) => setSessionToken(session_token))
        .catch(() => {
          localStorage.removeItem(LOGIN_TOKEN_KEY);
        });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const login = async (username: string, password: string) => {
    setLoading(true);

    await createSession({ username, password })
      .then(({ login_token, session_token }) => {
        setSessionToken(session_token);

        if (typeof login_token === "string") {
          setLoginToken(login_token);
        }
      })
      .finally(() => setLoading(false));
  };

  const logout = useCallback(async () => {
    localStorage.removeItem(LOGIN_TOKEN_KEY);
    sessionStorage.removeItem(SESSION_TOKEN_KEY);
    setLoggedOut(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <AuthContext.Provider
      value={{
        authenticated: !!sessionToken,
        loading,
        loggedOut,
        sessionToken,
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => useContext(AuthContext);

export interface SessionCredentials {
  scope: string;
}

export const useSessionCredentials = (): SWRResponse<SessionCredentials> => {
  const { sessionToken } = useAuth();

  return useSWR(
    sessionToken ? `${API_ENDPOINT}/schedule/credentials` : null,
    async (url) => {
      const res = await fetch(url, {
        headers: {
          Authorization: `Bearer ${sessionToken}`,
        },
      });

      return res.json();
    }
  );
};
