CREATE TABLE links (
  owner UUID NOT NULL,
  id BYTEA PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ,
  range DATERANGE NOT NULL
);