import { LayoutGroup, motion } from "framer-motion";
import { createContext, FunctionComponent, useContext, useState } from "react";
import {
  CalendarColors,
  CalendarEvent,
  insertCalendarEvent,
  useCalendarColors,
  useCalendarList,
} from "../../lib/google/calendar";
import { useGoogleAuthorization } from "./auth";
import useSWRInfinite from "swr/infinite";
import { DateTime } from "luxon";
import { fetchLessons, useTimetables } from "../../lib/schedule";
import {
  SessionCredentials,
  useAuth,
  useSessionCredentials,
} from "../../lib/auth";
import chroma from "chroma-js";

export const GoogleCalendarContext = createContext<{
  calendar: string | null;
  setCalendar: (calendar: string | null) => void;
}>({ calendar: null, setCalendar: () => {} });

export const GoogleCalendarProvider: FunctionComponent = ({ children }) => {
  const [calendar, setCalendar] = useState<string | null>(null);

  return (
    <GoogleCalendarContext.Provider value={{ calendar, setCalendar }}>
      {children}
    </GoogleCalendarContext.Provider>
  );
};

export const useCalendarContext = () => useContext(GoogleCalendarContext);

export const GoogleCalendarSelector: FunctionComponent = () => {
  const { data } = useCalendarList();
  const { calendar, setCalendar } = useCalendarContext();

  const writable = data?.items.filter(
    (i) => i.accessRole === "writer" || i.accessRole === "owner"
  );

  return (
    <div>
      <p>Välj inte en kalender som du är rädd om – skapa en ny istället.</p>
      <LayoutGroup>
        <ul>
          {writable?.map(
            ({ id, backgroundColor, foregroundColor, summary }) => (
              <li key={id}>
                {id === calendar && (
                  <motion.div
                    layoutId="indicator"
                    style={{ position: "absolute", left: 0 }}
                  >
                    vald &gt;
                  </motion.div>
                )}
                <button
                  style={{
                    backgroundColor,
                    color: foregroundColor,
                    opacity: id === calendar ? 1 : 0.5,
                  }}
                  onClick={() => setCalendar(id)}
                >
                  {summary}
                </button>
              </li>
            )
          )}
        </ul>
        <style jsx>{`
          ul {
            list-style: none;
            padding: 0 0 0 7ch;
            margin: 0;
            position: relative;
            font-family: monospace;
          }
        `}</style>
      </LayoutGroup>
    </div>
  );
};

function toHex(str: string) {
  let result = "";
  for (var i = 0; i < str.length; i++) {
    result += str.charCodeAt(i).toString(16);
  }
  return result;
}

export const GoogleCalendarExport: FunctionComponent<{
  timetable: string;
  sessionToken: string;
  sessionCredentials: SessionCredentials;
  colors: CalendarColors;
}> = ({ timetable, sessionToken, sessionCredentials, colors }) => {
  const authorization = useGoogleAuthorization();
  const { calendar } = useCalendarContext();
  const [start] = useState(DateTime.now);

  const { data, isValidating } = useSWRInfinite(
    (index) => start.plus({ weeks: index }).toISODate(),
    async (key) => {
      const { year, weekNumber } = DateTime.fromISO(key);
      return fetchLessons(
        { timetable: timetable!, year, week: weekNumber },
        sessionCredentials,
        sessionToken
      );
    },
    {
      initialSize: 2,
    }
  );

  const colorIds = Object.fromEntries(
    Object.entries(colors.event).map(([id, color]) => [color.background, id])
  );

  const events: Partial<CalendarEvent>[] | undefined = data
    ?.flat()
    .map((lesson) => {
      const closestColor = lesson.color
        ? Object.entries(colorIds).reduce(
            (closest, [color, id]) => {
              const distance = chroma.distance(color, lesson.color!);
              return distance < closest.distance ? { distance, id } : closest;
            },
            { distance: Infinity, id: "" }
          )
        : undefined;

      return {
        summary: lesson.course ?? "(Namnlös)",
        description: lesson.teacher ?? undefined,
        location: lesson.location ?? undefined,
        start: {
          dateTime: lesson.start,
        },
        end: {
          dateTime: lesson.end,
        },
        colorId: closestColor?.id,
        id: toHex(lesson.id),
      };
    });

  if (!calendar) {
    return <>Välj en kalender för att fortsätta</>;
  }

  if (!authorization) {
    return <>Vänta ...</>;
  }

  if (!events) {
    return <>Hämtar lektioner ...</>;
  }

  return (
    <div>
      <button
        onClick={() =>
          Promise.all(
            events.map((e) => insertCalendarEvent(authorization, calendar, e))
          )
        }
      >
        Exportera {events.length} händelser
      </button>
    </div>
  );
};
