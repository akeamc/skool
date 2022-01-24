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
import { retryRequest } from "./retry";

interface SessionFromUsernamePassword {
  username: string;
  password: string;
}

interface SessionFromRefreshToken {
  refresh_token: string;
}

type CreateSessionRequest = Either<
  SessionFromUsernamePassword,
  SessionFromRefreshToken
>;

interface CreateSessionResponse {
  session_token: string;
  refresh_token: string | null;
}

async function createSession(
  data: CreateSessionRequest
): Promise<CreateSessionResponse> {
  const res = await retryRequest(
    fetch(`${API_ENDPOINT}/auth/session`, {
      method: "POST",
      body: JSON.stringify(data),
      headers: {
        "Content-Type": "application/json",
      },
    })
  );

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

const REFRESH_TOKEN_KEY = "refresh_token";
const SESSION_TOKEN_KEY = "session_token";

const useRefreshTokenState = createPersistedState(REFRESH_TOKEN_KEY);
const useSessionTokenState =
  typeof window === "undefined"
    ? () => []
    : createPersistedState(SESSION_TOKEN_KEY, window.sessionStorage);

export const AuthProvider: FunctionComponent = ({ children }) => {
  const [loggedOut, setLoggedOut] = useState(false);
  const [loading, setLoading] = useState(true);
  const [refreshToken, setRefreshToken] = useRefreshTokenState<string>();
  const [sessionToken, setSessionToken] = useSessionTokenState<string>();

  useEffect(() => {
    if (refreshToken && !sessionToken) {
      createSession({ refresh_token: refreshToken }).then(({ session_token }) =>
        setSessionToken(session_token)
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const bruhLogin = async (username: string, password: string) => {
    setLoading(true);

    await createSession({ username, password })
      .then(({ refresh_token, session_token }) => {
        setSessionToken(session_token);

        if (typeof refresh_token === "string") {
          setRefreshToken(refresh_token);
        }
      })
      .finally(() => setLoading(false));
  };

  const bruhLogout = useCallback(async () => {
    setRefreshToken(undefined);
    setSessionToken(undefined);
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
        login: bruhLogin,
        logout: bruhLogout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => useContext(AuthContext);

interface SessionCredentials {
  scope: string;
}

export const useSessionCredentials = (): SWRResponse<
SessionCredentials
> => {
  const { sessionToken } = useAuth();

  return useSWR(
    sessionToken ? `/schedule/credentials` : null,
    async () => {
      const res = await fetch(`${API_ENDPOINT}/schedule/credentials`, {
        headers: {
          Authorization: `Bearer ${sessionToken}`,
        },
      });

      return res.json();
    }
  );
};
