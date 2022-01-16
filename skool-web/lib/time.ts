import { DateTime } from "luxon";
import { useEffect, useState } from "react";

export function useTime(refreshInterval = 1000): DateTime {
		const [time, setTime] = useState(DateTime.now);

		useEffect(() => {
				const interval = setInterval(() => {
						setTime(DateTime.now);
				}, refreshInterval);

				return () => clearInterval(interval);
		}, [refreshInterval]);

		return time;
}
