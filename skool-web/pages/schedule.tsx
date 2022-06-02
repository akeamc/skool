import type { NextPage } from "next";
import { useAuth, withAuth } from "../lib/auth";
import { useTimetables } from "../lib/schedule";
import { Timetable } from "../components/timetable/timetable";
import Layout from "../components/layout/layout";

const Schedule: NextPage = () => {
  const { login, logout, ...auth } = useAuth();
  const { data: timetables } = useTimetables();

  return (
    <Layout>
      <section>
        {timetables?.map(({ timetable_id }) => (
          <Timetable id={timetable_id} key={timetable_id} />
        ))}
      </section>
    </Layout>
  );
};

export default withAuth(Schedule);
