import { LayoutGroup, motion } from "framer-motion";
import { createContext, FunctionComponent, useContext, useState } from "react";
import { useCalendarList } from "../../lib/google/calendar";

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
      <p>Välj inte en kalender som du är rädd om – skapa en ny.</p>
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
                    &gt;
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
            padding: 0 0 0 2ch;
            margin: 0;
            position: relative;
            font-family: monospace;
          }
        `}</style>
      </LayoutGroup>
    </div>
  );
};
