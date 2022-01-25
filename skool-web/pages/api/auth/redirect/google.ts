import { NextApiHandler } from "next";
import { GOOGLE_REDIRECT_URL } from "../../../../lib/google/oauth";

const { NEXT_PUBLIC_GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET } = process.env;

const handler: NextApiHandler = async (req, res) => {
  const { code } = req.query;

  const { access_token } = await fetch(`https://oauth2.googleapis.com/token`, {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body: `code=${code}&client_id=${NEXT_PUBLIC_GOOGLE_CLIENT_ID}&client_secret=${GOOGLE_CLIENT_SECRET}&redirect_uri=${GOOGLE_REDIRECT_URL}&grant_type=authorization_code`,
  }).then((res) => res.json());

  res.redirect(`/export/google?google_access_token=${access_token}`);
};

export default handler;
