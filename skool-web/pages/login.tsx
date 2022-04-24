import { ErrorMessage, Field, Form, Formik } from "formik";
import { useAuth } from "../lib/auth";
import { NextPage } from "next";
import { useRouter } from "next/router";
import { useEffect } from "react";

const LoginPage: NextPage = () => {
  const { login, authenticated } = useAuth();
  const router = useRouter();
  const redirect = router.query.redirect?.toString() || "/"; // callback url

  useEffect(() => {
    if (authenticated) {
      router.push(redirect);
    }
  }, [authenticated, redirect, router]);

  return (
    <Formik
      initialValues={{ username: "", password: "" }}
      onSubmit={({ username, password }, { setStatus }) => {
        setStatus();
        login(username, password).catch((e) => setStatus(e.toString()));
      }}
    >
      {({ status }) => (
        <Form>
          <label htmlFor="username">Username</label>
          <Field id="username" name="username" placeholder="ab12345" />
          <ErrorMessage name="username" />

          <label htmlFor="password">Password</label>
          <Field id="password" name="password" type="password" />
          <ErrorMessage name="password" />

          <button type="submit">Logga in</button>

          <div>{status}</div>
        </Form>
      )}
    </Formik>
  );
};

export default LoginPage;
