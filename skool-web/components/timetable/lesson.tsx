import classNames from "classnames/bind";
import { DateTime } from "luxon";
import { FunctionComponent } from "react";
import { useContainerQuery } from "react-container-query";
import { Query } from "react-container-query/lib/interfaces";
import { Lesson } from "../../lib/schedule";
import { useTime } from "../../lib/time";
import styles from "./lesson.module.scss";

const cx = classNames.bind(styles);

const containerQuery: Query = {
  horizontal: {
    maxHeight: 48,
  },
  narrow: {
    maxWidth: 192,
  },
};

export const FloatingLesson: FunctionComponent<FloatingLessonProps> = ({
  start,
  end,
  startSecs,
  durationSecs,
  course,
  teacher,
  left,
  width
}) => {
  const [params, containerRef] = useContainerQuery(containerQuery, {});
  const now = useTime();

  return (
    <div
      ref={containerRef}
      className={cx("event", params, { past: now >= end })}
      style={{
        ["--start-secs" as any]: startSecs,
        ["--duration-secs" as any]: durationSecs,
        ["--left" as any]: left,
        ["--width" as any]: width
      }}
    >
      <div className={cx("content")}>
        <h3>{course}</h3>
        <span>
          <time>{start.toLocaleString(DateTime.TIME_SIMPLE)}</time>–
          <time>{end.toLocaleString(DateTime.TIME_SIMPLE)}</time>
          {["", location, teacher]
            .filter((v) => typeof v == "string")
            .join(" · ")}
        </span>
      </div>
    </div>
  );
};

export interface FloatingLessonProps extends Lesson {
  startSecs: number;
  durationSecs: number;
  left: number;
  width: number;
}
