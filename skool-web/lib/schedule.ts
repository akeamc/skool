import { DateTime } from "luxon";
import useSWR, { SWRResponse } from "swr";
import { API_ENDPOINT } from "./api";
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
  const { sessionToken } = useAuth();

  return useSWR(sessionToken ? "/schedule/timetables" : null, async (path) => {
    return fetch(`${API_ENDPOINT}${path}`, {
      headers: {
        Authorization: `Bearer ${sessionToken}`,
      },
    }).then((res) => res.json());
  });
}

interface LessonJson {
  teacher: string | null;
  location: string | null;
  start: string;
  end: string;
  course: string | null;
  id: string;
  color: string | null;
}

export interface Lesson extends Omit<LessonJson, "start" | "end"> {
  start: DateTime;
  end: DateTime;
}

interface FetchLessons {
  timetable: string;
  year: number;
  week: number;
}

const fetchLessonsUrl = ({ timetable, year, week }: FetchLessons) =>
  `${API_ENDPOINT}/schedule/timetables/${timetable}/lessons?year=${year}&week=${week}`;

export async function fetchLessons(
  { timetable, year, week }: FetchLessons,
  sessionToken: string
): Promise<Lesson[]> {
  const res = await fetch(fetchLessonsUrl({ timetable, year, week }), {
    headers: {
      Authorization: `Bearer ${sessionToken}`,
    },
  });

  const json: LessonJson[] = await res.json();
  const lessons = json.map((lesson) => ({
    ...lesson,
    start: DateTime.fromISO(lesson.start).toLocal(),
    end: DateTime.fromISO(lesson.end).toLocal(),
  }));

  return lessons.sort(
    (a, b) => +a.start - +b.start
  );
}

export function useLessons({
  timetable,
  year,
  week,
}: Partial<FetchLessons>): SWRResponse<Lesson[]> {
  const { sessionToken } = useAuth();

  return useSWR(
    timetable && sessionToken && year && week
      ? fetchLessonsUrl({ timetable, year, week })
      : null,
    () =>
      fetchLessons(
        { timetable: timetable!, year: year!, week: week! },
        sessionToken!
      )
  );
}
