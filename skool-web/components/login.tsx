import { FunctionComponent } from "react";
import { Field, Form, Formik } from "formik";
import { useAuth } from "../lib/auth";

export const Login: FunctionComponent = () => {
  const { login } = useAuth();

  return (
    <Formik
      initialValues={{ username: "", password: "" }}
      onSubmit={({ username, password }) => login(username, password)}
    >
      <Form>
        <label htmlFor="username">Email</label>
        <Field id="username" name="username" placeholder="ab12345" />

        <label htmlFor="password">Password</label>
        <Field id="password" name="password" type="password" />

        <button type="submit">Submit</button>
      </Form>
    </Formik>
  );
};
