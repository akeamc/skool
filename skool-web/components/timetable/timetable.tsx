import { DateTime } from "luxon";
import { createContext, FunctionComponent, memo, useContext, useState } from "react";
import { Lesson, useLessons } from "../../lib/schedule";
import { Scale } from "./scale";
import styles from "./timetable.module.scss";

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

const FloatingLesson: FunctionComponent<{ lesson: Lesson }> = ({ lesson }) => {
  const start = DateTime.fromISO(lesson.start);
  const end = DateTime.fromISO(lesson.end);

  return (
    <div
      key={lesson.start}
      className={styles.event}
      style={{
        ["--start-secs" as any]:
          start.hour * 3600 + start.minute * 60 + start.second,
        ["--duration-secs" as any]: end.diff(start).as("seconds"),
      }}
    >
      <time>{DateTime.fromISO(lesson.start).toLocaleString(DateTime.TIME_SIMPLE)}</time>
      –
      <time>{DateTime.fromISO(lesson.end).toLocaleString(DateTime.TIME_SIMPLE)}</time>
      <h3>
      {lesson.course}
      </h3>
      <div className={styles.details}>
        {[lesson.location, lesson.teacher].filter((v) => typeof v == "string").join(" · ")}
      </div>
    </div>
  );
};

const DayColumn: FunctionComponent<{ weekday: number }> = ({ weekday }) => {
  const { year, week, id, cursor } = useTimetableContext();
  const { data } = useLessons({ timetable: id, year, week });
  const timestamp = cursor?.set({ weekday });
  const lessons =
    (timestamp
      ? data?.filter((d) =>
          DateTime.fromISO(d.start)
            .setZone(timestamp.zone)
            .hasSame(timestamp, "day")
        )
      : undefined) ?? [];

  return (
    <div>
      <div className={styles.board}>
        {lessons.map((lesson) => (
          <FloatingLesson lesson={lesson} key={lesson.start} />
        ))}
      </div>
    </div>
  );
};

const Controls: FunctionComponent = () => {
  const {cursor, setCursor} = useTimetableContext();
  
  return (
    <div>
      <button onClick={() => setCursor(cursor?.minus({weeks: 1}))}>prev</button>
      {cursor?.toLocaleString(DateTime.DATE_FULL)}
      <button onClick={() => setCursor(cursor?.plus({weeks: 1}))}>next</button>
    </div>
  )
}

export const Timetable: FunctionComponent<Props> = ({ id }) => {
  const [cursor, setCursor] = useState<DateTime | undefined>(DateTime.now);
  // const { data: lessons } = useLessons({ timetable: id, year: 2022, week: 2 });

  const days = 5;

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
      <Controls />
      <div className={styles.cols} style={{["--days" as any]: days}}>
        <Scale />
        {Array.from({ length: days }).map((_, i) => (
          <DayColumn weekday={i + 1} key={i} />
        ))}
      </div>
    </TimetableContext.Provider>
  );
};
