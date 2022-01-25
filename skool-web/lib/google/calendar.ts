import useSWR from "swr";
import { useGoogleAuthorization } from "../../components/google/auth";

interface CalendarListEntry {
  id: string;
  summary: string;
  description: string;
  backgroundColor: string;
  foregroundColor: string;
  accessRole: "reader" | "writer" | "owner";
  // ...
}

interface CalendarList {
  items: CalendarListEntry[];
}

export function useCalendarList() {
  const authorization = useGoogleAuthorization();

  return useSWR(
    authorization
      ? "https://www.googleapis.com/calendar/v3/users/me/calendarList"
      : null,
    async (url) => {
      const res = await fetch(url, {
        headers: {
          authorization: authorization!,
        },
      }).then((res) => res.json());

      return res as CalendarList;
    }
  );
}
