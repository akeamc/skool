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

interface Color {
  background: string;
  foreground: string;
}

export interface CalendarColors {
  calendar: Record<string, Color>;
  event: Record<string, Color>;
}

export function useCalendarColors() {
  const authorization = useGoogleAuthorization();

  return useSWR(
    authorization ? "https://www.googleapis.com/calendar/v3/colors" : null,
    async (url) => {
      const res = await fetch(url, {
        headers: {
          authorization: authorization!,
        },
      }).then((res) => res.json());

      return res as CalendarColors;
    }
  );
}

export interface CalendarEvent {
  id: string;
  summary: string;
  description: string;
  location: string;
  start: {
    dateTime: string;
  };
  end: {
    dateTime: string;
  };
  colorId: string;
  icalUID: string;
  // ...
}

export async function insertCalendarEvent(
  authorization: string,
  calendar: string,
  body: Partial<CalendarEvent>
) {
  return fetch(
    `https://www.googleapis.com/calendar/v3/calendars/${calendar}/events`,
    {
      method: "POST",
      headers: {
        authorization,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(body),
    }
  );
}
