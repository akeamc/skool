import useSWR, { SWRResponse } from "swr";
import { API_ENDPOINT } from "./api";
import { useAuth, useSessionCredentials } from "./auth";

export interface Timetable {
  school_guid: string;
  unit_guid: string;
  school_id: string;
  timetable_id: string;
  person_guid: string;
  first_name: string;
  last_name: string;
}

export function useTimetables(): SWRResponse<Timetable[]> {
  const { sessionToken } = useAuth();
  const { data: credentials } = useSessionCredentials();

  return useSWR(sessionToken && credentials ? `/schedule/timetables?scope=${credentials.scope}` : null, async (path) => {
    return fetch(`${API_ENDPOINT}${path}`, {
      headers: {
        Authorization: `Bearer ${sessionToken}`,
      }
    }).then((res) => res.json());
  });
}

export interface Lesson {
  teacher: string | null;
  location: string | null;
  start: string;
  end: string;
  course: string | null;
  id: string;
  color: string | null;
}

interface UseLessons {
  timetable?: string;
  year?: number;
  week?: number;
}

export function useLessons({ timetable, year, week }: UseLessons): SWRResponse<Lesson[]> {
  const { sessionToken } = useAuth();
  const { data: credentials } = useSessionCredentials();

  return useSWR(
    timetable && sessionToken && year && week && credentials ? `/schedule/timetables/${timetable}/lessons?year=${year}&week=${week}&scope=${credentials.scope}` : null,
    async (path) => {
      return fetch(
        `${API_ENDPOINT}${path}`,
        {
          headers: {
            Authorization: `Bearer ${sessionToken}`,
          }
        }
      ).then((res) => res.json());
    }
  );
}
