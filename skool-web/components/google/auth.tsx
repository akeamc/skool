import { useRouter } from "next/router";

export function useGoogleAuthorization(): string | undefined {
  const router = useRouter();
  const accessToken = router.query.google_access_token?.toString();

  if (typeof accessToken === "string") {
    return `Bearer ${accessToken}`;
  }

  return undefined;
}
