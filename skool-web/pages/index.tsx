import type { NextPage } from "next";
import Layout from "../components/layout/layout";

const Home: NextPage = () => (
  <Layout>
    <h1>The cooler Skolplattformen</h1>
    <img src="/assets/logo.svg" />
    <style jsx>{`
      h1 {
        font-size: 96px;
        letter-spacing: -0.02em;
      }
    `}</style>
  </Layout>
);

export default Home;
