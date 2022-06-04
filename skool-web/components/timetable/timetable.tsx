import classNames from "classnames/bind";
import { DateTime } from "luxon";
import {
  createContext,
  FunctionComponent,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { Lesson, useLessons } from "../../lib/schedule";
import { Scale } from "./scale";
import styles from "./timetable.module.scss";
import { useTime } from "../../lib/time";
import { googleAuthUrl, GOOGLE_CALENDAR_SCOPES } from "../../lib/google/oauth";
import { FloatingLesson, FloatingLessonProps } from "./lesson";

const cx = classNames.bind(styles);

interface Context {
  cursor?: DateTime;
  setCursor: (cursor?: DateTime) => void;
  year?: number;
  week?: number;
  id?: string;
}

export const TimetableContext = createContext<Context>({ setCursor: () => {} });

export const useTimetableContext = () => useContext(TimetableContext);

interface Props {
  id?: string;
}

const Indicator: FunctionComponent = () => {
  const now = useTime();
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (typeof ref.current?.scrollIntoView === "function") {
      ref.current.scrollIntoView({
        behavior: "smooth",
        block: "center",
      });
    }
  }, []);

  return (
    <div
      ref={ref}
      className={cx("indicator")}
      style={{
        ["--secs" as any]: now.hour * 3600 + now.minute * 60 + now.second,
      }}
    />
  );
};

function positionEvents(lessons: Lesson[]): FloatingLessonProps[] {
  type Cell = Omit<FloatingLessonProps, "left" | "width">;

  const cols = lessons.reduce(
    (cols, l) => {
      const startSecs =
        l.start.hour * 3600 + l.start.minute * 60 + l.start.second;
      const durationSecs = l.end.diff(l.start).as("seconds");
      const props = {
        ...l,
        startSecs,
        durationSecs,
        left: 0,
        width: 1,
      };

      for (let i = 0; i < cols.length; i++) {
        const hasSpace = cols[i].every(
          (l) =>
            l.startSecs + l.durationSecs <= startSecs ||
            l.startSecs >= startSecs + durationSecs
        );

        if (hasSpace) {
          cols[i].push(props);
          return cols;
        }
      }

      cols.push([props]); // start a new column

      return cols;
    },
    [[]] as Cell[][]
  );

  return cols.flatMap((col, i) =>
    col.map((l) => {
      let spanCols = 1;

      // expand to the right
      for (let j = i + 1; j < cols.length; j++) {
        const hasSpace = cols[j].every(
          (other) =>
            other.startSecs + other.durationSecs <= l.startSecs ||
            other.startSecs >= l.startSecs + l.durationSecs
        );

        if (!hasSpace) {
          break;
        }

        spanCols++;
      }

      return { ...l, left: i / cols.length, width: spanCols / cols.length };
    })
  );
}

const DayColumn: FunctionComponent<{ day?: DateTime }> = ({ day }) => {
  const now = useTime(undefined, "day"); // if performance hurts, make sure this only updates when the day changes
  const { year, week, id } = useTimetableContext();
  const { data } = useLessons({ timetable: id, year, week });
  const isToday = day?.hasSame(now, "day") ?? false;
  const lessons: FloatingLessonProps[] =
    (day && data
      ? positionEvents(data.filter(({ start }) => start.hasSame(day, "day")))
      : undefined) ?? [];

  return (
    <div className={styles.col}>
      {isToday && <Indicator />}
      {lessons.map((lesson) => (
        <FloatingLesson {...lesson} key={lesson.id} />
      ))}
    </div>
  );
};

const Controls: FunctionComponent = () => {
  const { cursor, setCursor } = useTimetableContext();

  return (
    <div>
      <button onClick={() => setCursor(cursor?.minus({ weeks: 1 }))}>
        prev
      </button>
      Vecka {cursor?.weekNumber}
      <button onClick={() => setCursor(cursor?.plus({ weeks: 1 }))}>
        next
      </button>
    </div>
  );
};

export const Timetable: FunctionComponent<Props> = ({ id }) => {
  const [cursor, setCursor] = useState<DateTime | undefined>(DateTime.now);

  const days = Array.from({ length: 5 }).map((_, i) =>
    cursor?.set({ weekday: i + 1 })
  );

  return (
    <TimetableContext.Provider
      value={{
        cursor,
        setCursor,
        year: cursor?.year,
        week: cursor?.weekNumber,
        id,
      }}
    >
      <a href={googleAuthUrl(GOOGLE_CALENDAR_SCOPES)}>
        exportera till google calendar
      </a>
      <Controls />
      <div className={styles.table} style={{ ["--days" as any]: days.length }}>
        <header>
          <div /> {/* empty cell */}
          {days.map((d) => (
            <div key={d?.toISODate()}>
              {d?.toLocaleString({ weekday: "long" })}{" "}
              {d?.toLocaleString({ day: "numeric", month: "numeric" })}
            </div>
          ))}
        </header>
        <main>
          <Scale />
          {days.map((d) => (
            <DayColumn day={d} key={d?.toISODate()} />
          ))}
        </main>
      </div>
    </TimetableContext.Provider>
  );
};
