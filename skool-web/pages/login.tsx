import { Field, Form, Formik } from "formik";
import { useAuth } from "../lib/auth";
import { NextPage } from "next";
import { useRouter } from "next/router";
import { useEffect } from "react";
import Layout from "../components/layout/layout";
import styles from "./login.module.scss";

const LoginPage: NextPage = () => {
  const { login, authenticated, loggingIn } = useAuth();
  const router = useRouter();
  const redirect = router.query.redirect?.toString() || "/"; // callback url

  useEffect(() => {
    if (authenticated) {
      router.push(redirect);
    }
  }, [authenticated, redirect, router]);

  return (
    <Layout navbar={false}>
      <div className="root">
        <div className="card">
          <h1>Logga in</h1>
          <p>
            Dina inloggningsuppgifter krypteras och sparas bara på den här
            enheten. Var försiktig!
          </p>
          <Formik
            initialValues={{ username: "", password: "" }}
            onSubmit={({ username, password }, { setStatus }) => {
              setStatus();
              login(username, password).catch((e) => setStatus(e.toString()));
            }}
          >
            {({ status }) => (
              <Form className={styles.form}>
                <label htmlFor="username">Användarnamn</label>
                <Field id="username" name="username" placeholder="ab12345" />

                <label htmlFor="password">Lösenord</label>
                <Field id="password" name="password" type="password" />

                <div>{status}</div>

                <button type="submit" disabled={loggingIn}>
                  {loggingIn ? "Loggar in ..." : "Logga in"}
                </button>
              </Form>
            )}
          </Formik>
        </div>
      </div>
      <style jsx>{`
        .root {
          display: flex;
          flex-direction: column;
          align-items: center;
          min-height: 100vh;
        }

        .card {
          padding: 0 var(--page-gutter);
          margin: calc(var(--navbar-height) + var(--page-gutter)) 0;
        }

        h1 {
          font-size: 32px;
          font-weight: 500;
          letter-spacing: -0.025em;
          margin: 16px 0;
        }

        p {
          color: var(--text-muted);
          font-size: 14px;
          line-height: 1.5;
          letter-spacing: -0.006em;
          margin: 16px 0;
          font-weight: 400;
        }

        h1,
        p {
          text-align: center;
        }

        button {
          background-color: var(--brand-primary);
          color: #fff;
          border: 0;
          font-family: var(--font-main);
          border-radius: 4px;
          width: 100%;
          height: 48px;
          font-size: 14px;
          line-height: 1;
          cursor: pointer;
          transition: all 0.1s;
        }

        button:hover {
          background-color: var(--brand-primary-dark);
        }

        button:disabled {
          opacity: 0.1;
          cursor: default;
        }

        @media (min-width: 480px) {
          .root {
            background-color: var(--background-secondary);
            justify-content: center;
          }

          .card {
            padding: var(--page-gutter);
            width: 400px;
            border-radius: 12px;
            box-shadow: 0px 1px 1px rgba(0, 0, 0, 0.1);
            background-color: var(--background-primary);
          }
        }
      `}</style>
    </Layout>
  );
};

export default LoginPage;
