import { useRouter } from "next/router";
import {
  Component,
  ComponentType,
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

interface UsernamePasswordPayload {
  username: string;
  password: string;
}

interface LoginTokenPayload {
  login_token: string;
}

type CreateSessionRequest = Either<UsernamePasswordPayload, LoginTokenPayload>;

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
  loggingIn: boolean;
  loggedOut: boolean;
  sessionToken?: string;
  login: (username: string, password: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthData>({
  authenticated: false,
  loggingIn: false,
  loggedOut: true,
  login: async () => {},
  logout: () => {},
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
  const [loggingIn, setLoggingIn] = useState(false);
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
    setLoggingIn(true);

    try {
      const { login_token, session_token } = await createSession({
        username,
        password,
      });

      setSessionToken(session_token);

      if (typeof login_token === "string") {
        setLoginToken(login_token);
      }
    } finally {
      setLoggingIn(false);
    }
  };

  const logout = useCallback(() => {
    setLoginToken(undefined);
    localStorage.removeItem(LOGIN_TOKEN_KEY);
    setSessionToken(undefined);
    sessionStorage.removeItem(SESSION_TOKEN_KEY);
    setLoggedOut(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <AuthContext.Provider
      value={{
        authenticated: !!sessionToken,
        loggingIn: loggingIn,
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

export function withAuth<P extends object>(Component: ComponentType<P>): FunctionComponent<P> {
  // eslint-disable-next-line react/display-name
  return (props) => {
    const auth = useAuth();
    const router = useRouter();

    const redirect = !auth.authenticated;

    useEffect(() => {
      if (redirect) {
        router.push("/login", {query: {redirect: router.asPath}});
      }
    }, [redirect, router]);

    if (redirect) {
      return <>omdirigerar...</>;
    } else {
      return <Component {...props} />;
    }
  }
}
