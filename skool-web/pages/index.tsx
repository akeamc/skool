import type { NextPage } from "next";
import { useAuth, withAuth } from "../lib/auth";
import { useTimetables } from "../lib/schedule";
import { Timetable } from "../components/timetable/timetable";

const Home: NextPage = () => {
  const { login, logout, ...auth } = useAuth();
  const { data: timetables } = useTimetables();

  return (
    <div>
      <h1>Skålplattformen 🥂</h1>
      <section>
        {timetables?.map(({ timetable_id, first_name, last_name }) => (
          <div key={timetable_id}>
            <h2>
              Var hälsad, {first_name} {last_name}
            </h2>
            <Timetable id={timetable_id} />
          </div>
        ))}
      </section>
    </div>
  );
};

export default withAuth(Home);
