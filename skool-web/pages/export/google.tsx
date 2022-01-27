import { NextPage } from "next";
import {
  GoogleCalendarExport,
  GoogleCalendarProvider,
  GoogleCalendarSelector,
} from "../../components/google/calendars";
import { useAuth, useSessionCredentials } from "../../lib/auth";
import { useCalendarColors } from "../../lib/google/calendar";
import { useTimetables } from "../../lib/schedule";

const GoogleExport: NextPage = () => {
  const { sessionToken } = useAuth();
  const { data } = useTimetables();
  const timetable = data?.[0].timetable_id;
  const { data: sessionCredentials } = useSessionCredentials();
  const { data: colors } = useCalendarColors();

  return (
    <GoogleCalendarProvider>
      <h1>Importera kalender</h1>
      <GoogleCalendarSelector />
      {sessionToken && timetable && sessionCredentials && colors && (
        <GoogleCalendarExport
          sessionToken={sessionToken}
          timetable={timetable}
          sessionCredentials={sessionCredentials}
          colors={colors}
        />
      )}
    </GoogleCalendarProvider>
  );
};

export default GoogleExport;
