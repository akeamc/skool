import useSWR, { SWRResponse } from "swr";
import { useAuth } from "./auth";

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
  const { authenticated } = useAuth();

  return useSWR(authenticated ? "/schedule/timetables" : null, async () => {
    return fetch(`http://localhost:8000/schedule/timetables`, {
      credentials: "include",
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

export function useLessons({timetable, year, week}: UseLessons): SWRResponse<Lesson[]> {
  const { authenticated } = useAuth();

  return useSWR(
    timetable && authenticated && year && week ? `/schedule/timetables/${timetable}/lessons?year=${year}&week=${week}` : null,
    async (path) => {
      return fetch(
        `http://localhost:8000${path}`,
        {
          credentials: "include",
        }
      ).then((res) => res.json());
    }
  );
}
