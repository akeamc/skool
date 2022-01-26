import { LayoutGroup, motion } from "framer-motion";
import { createContext, FunctionComponent, useContext, useState } from "react";
import { useCalendarList } from "../../lib/google/calendar";
import { useGoogleAuthorization } from "./auth";
import useSWRInfinite from "swr/infinite";
import { DateTime } from "luxon";
import { fetchLessons, useTimetables } from "../../lib/schedule";
import {
  SessionCredentials,
  useAuth,
  useSessionCredentials,
} from "../../lib/auth";

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

export const GoogleCalendarExport: FunctionComponent<{
  timetable: string;
  sessionToken: string;
  sessionCredentials: SessionCredentials;
}> = ({ timetable, sessionToken, sessionCredentials }) => {
  const { calendar } = useCalendarContext();
  const authorizationHeader = useGoogleAuthorization();
  const [start] = useState(DateTime.now);

  const { data, error, isValidating, mutate, size, setSize } = useSWRInfinite(
    (index) => start.plus({ weeks: index }).toISODate(),
    async (key) => {
      const { year, weekNumber } = DateTime.fromISO(key);
      console.log(key);
      return fetchLessons(
        { timetable: timetable!, year, week: weekNumber },
        sessionCredentials,
        sessionToken
      );
    },
    {
      initialSize: 10,
    }
  );

  if (!calendar) {
    return <>Välj en kalender för att fortsätta</>;
  }

  return (
    <div>
      <button>Exportera</button>
      <pre>{JSON.stringify(data, null, 2)}</pre>
    </div>
  );
};
