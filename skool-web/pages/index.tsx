import type { NextPage } from "next";
import { useAuth } from "../lib/auth";
import { useLessons, useTimetables } from "../lib/schedule";
import { FunctionComponent } from "react";
import { Login } from "../components/Login";

interface Props {
  id: string;
}

const Timetable: FunctionComponent<Props> = ({ id }) => {
  const { data: lessons } = useLessons(id);

  return <pre>{JSON.stringify(lessons, null, 2)}</pre>;
};

const Home: NextPage = () => {
  const { login, logout, ...auth } = useAuth();
  const { data: timetables } = useTimetables();

  return (
    <div>
      <h1>Skålplattformen 🥂</h1>
      <pre>{JSON.stringify(auth, null, 2)}</pre>
      {!auth.authenticated && !auth.loading && <Login />}
      {auth.authenticated && (
        <button onClick={logout} type="button">
          Log out
        </button>
      )}
      <section>
        {timetables?.map(({ timetable_id, first_name }) => (
          <div key={timetable_id}>
            <h2>{first_name}</h2>
            <Timetable id={timetable_id} />
          </div>
        ))}
      </section>
    </div>
  );
};

export default Home;
