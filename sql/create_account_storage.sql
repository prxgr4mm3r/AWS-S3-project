CREATE TABLE s3_account(
  id SERIAL PRIMARY KEY,
  key_id VARCHAR NOT NULL,
  key_secret VARCHAR NOT NULL,
  region VARCHAR NOT NULL,
  bucket_name VARCHAR NOT NULL
);

