import type { NextPage } from "next";
import { useAuth } from "../lib/auth";
import { useLessons, useTimetables } from "../lib/schedule";
import { FunctionComponent } from "react";
import { Login } from "../components/login";
import { Timetable } from "../components/timetable/timetable";

const Home: NextPage = () => {
  const { login, logout, ...auth } = useAuth();
  const { data: timetables } = useTimetables();

  return (
    <div>
      <h1>Skålplattformen 🥂</h1>
      <pre>{JSON.stringify(auth, null, 2)}</pre>
      {!auth.authenticated && <Login />}
      {auth.authenticated && (
        <button onClick={logout} type="button">
          Log out
        </button>
      )}
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

export default Home;
