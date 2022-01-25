import { DateTime, DateTimeUnit } from "luxon";
import { useEffect, useState } from "react";

export function useTime(
  refreshInterval = 1000,
  compare?: DateTimeUnit
): DateTime {
  const [time, setTime] = useState(DateTime.now);

  useEffect(() => {
    const interval = setInterval(() => {
      setTime((prev) => {
        const now = DateTime.now();

        if (!compare || !now.hasSame(prev, compare)) {
          return now;
        }

        return prev;
      });
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [compare, refreshInterval]);

  return time;
}
