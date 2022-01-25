export const GOOGLE_REDIRECT_URL =
  "http://localhost:3000/api/auth/redirect/google";

export const googleAuthUrl = (scopes: string[]) =>
  `https://accounts.google.com/o/oauth2/v2/auth?client_id=${
    process.env.NEXT_PUBLIC_GOOGLE_CLIENT_ID
  }&redirect_uri=${GOOGLE_REDIRECT_URL}&response_type=code&scope=${scopes.join(
    " "
  )}`;

export const GOOGLE_CALENDAR_SCOPES = [
  "https://www.googleapis.com/auth/calendar",
];
