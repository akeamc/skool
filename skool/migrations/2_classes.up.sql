CREATE TABLE classes (
  school BYTEA,
  reference VARCHAR(255),
  name VARCHAR(255) NOT NULL,
  PRIMARY KEY (school, reference)
);

ALTER TABLE
  credentials
ADD
  COLUMN school BYTEA;

ALTER TABLE
  credentials
ADD
  COLUMN class_reference VARCHAR(255);

ALTER TABLE
  credentials
ADD
  CONSTRAINT class_fkey FOREIGN KEY (school, class_reference) REFERENCES classes(school, reference);
