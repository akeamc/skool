import { NextPage } from "next";
import {
  GoogleCalendarExport,
  GoogleCalendarProvider,
  GoogleCalendarSelector,
} from "../../components/google/calendars";
import { useAuth, useSessionCredentials } from "../../lib/auth";
import { useTimetables } from "../../lib/schedule";

const GoogleExport: NextPage = () => {
  const { sessionToken } = useAuth();
  const { data } = useTimetables();
  const timetable = data?.[0].timetable_id;
  const { data: sessionCredentials } = useSessionCredentials();

  console.log(sessionCredentials);

  return (
    <GoogleCalendarProvider>
      <h1>Importera kalender</h1>
      <GoogleCalendarSelector />
      {sessionToken && timetable && sessionCredentials && (
        <GoogleCalendarExport
          sessionToken={sessionToken}
          timetable={timetable}
          sessionCredentials={sessionCredentials}
        />
      )}
    </GoogleCalendarProvider>
  );
};

export default GoogleExport;
