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

export function useScope() {
  const { authenticated } = useAuth();

  return useSWR(authenticated ? "/scope" : null, async () => {
    return fetch("http://localhost:8000/scope", {
      credentials: "include",
    }).then((res) => res.text());
  });
}

export function useTimetables(): SWRResponse<Timetable[]> {
  const { authenticated } = useAuth();
  const { data: scope } = useScope();

  return useSWR(authenticated && scope ? "/timetables" : null, async () => {
    return fetch(`http://localhost:8000/timetables?scope=${scope}`, {
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
}

export function useLessons(timetable: string): SWRResponse<Lesson[]> {
  const { authenticated } = useAuth();
  const { data: scope } = useScope();

  return useSWR(
    authenticated && scope ? `/timetables/${timetable}/lessons` : null,
    async () => {
      return fetch(
        `http://localhost:8000/timetables/${timetable}/lessons?scope=${scope}`,
        {
          credentials: "include",
        }
      ).then((res) => res.json());
    }
  );
}
