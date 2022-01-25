import { NextPage } from "next";
import {
  GoogleCalendarProvider,
  GoogleCalendarSelector,
} from "../../components/google/calendars";

const GoogleExport: NextPage = () => {
  return (
    <GoogleCalendarProvider>
      <h1>Importera kalender</h1>
      <GoogleCalendarSelector />
    </GoogleCalendarProvider>
  );
};

export default GoogleExport;
