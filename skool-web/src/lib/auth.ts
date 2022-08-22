import { browser } from "$app/env";
import { derived, writable } from "svelte/store";
import { API_ENDPOINT } from "./api";

export const loginToken = writable<string | null>(
	browser ? localStorage.getItem("login_token") : null
);
export const sessionToken = writable<string | null>(
	browser ? sessionStorage.getItem("session_token") : null
);
export const authenticating = writable<boolean>(false);
export const authenticated = derived(sessionToken, (t) => !!t);

loginToken.subscribe((v) => {
	if (browser) {
		if (v) {
			localStorage.setItem("login_token", v);
		} else {
			localStorage.removeItem("login_token");
		}
	}
});

sessionToken.subscribe((v) => {
	if (browser) {
		if (v) {
			sessionStorage.setItem("session_token", v);
		} else {
			sessionStorage.removeItem("session_token");
		}
	}
});

derived([loginToken, sessionToken], (a) => a).subscribe(async ([loginToken, sessionToken]) => {
	console.log(loginToken, sessionToken);
	if (loginToken && !sessionToken) {
		createSession({ login_token: loginToken });
	}
});

interface UsernamePasswordPayload {
	username: string;
	password: string;
}

interface LoginTokenPayload {
	login_token: string;
}

type Only<T, U> = {
	[P in keyof T]: T[P];
} & {
	[P in keyof U]?: never;
};

type Either<T, U> = Only<T, U> | Only<U, T>;

type CreateSessionRequest = Either<UsernamePasswordPayload, LoginTokenPayload>;

interface CreateSessionResponse {
	session_token: string;
	login_token: string | null;
}

export async function createSession(data: CreateSessionRequest): Promise<void> {
	authenticating.set(true);
	const res = await fetch(`${API_ENDPOINT}/auth/session`, {
		method: "POST",
		headers: {
			"content-type": "application/json"
		},
		body: JSON.stringify(data)
	});

	if (!res.ok) {
		authenticating.set(false);
		throw new Error(await res.text());
	}

	const { session_token, login_token }: CreateSessionResponse = await res.json();

	sessionToken.set(session_token);

	if (login_token) {
		loginToken.set(login_token);
	}

	authenticating.set(false);
}

export function logout() {
	// this order is important, otherwise the sessionToken will be renewed
	loginToken.set(null);
	sessionToken.set(null);
}
