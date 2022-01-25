import { FunctionComponent } from "react";
import { ErrorMessage, Field, Form, Formik } from "formik";
import { useAuth } from "../lib/auth";

export const Login: FunctionComponent = () => {
  const { login } = useAuth();

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
          <label htmlFor="username">Email</label>
          <Field id="username" name="username" placeholder="ab12345" />
          <ErrorMessage name="username" />

          <label htmlFor="password">Password</label>
          <Field id="password" name="password" type="password" />
          <ErrorMessage name="password" />

          <button type="submit">Submit</button>

          <div>{status}</div>
        </Form>
      )}
    </Formik>
  );
};
