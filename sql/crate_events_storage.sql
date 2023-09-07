CREATE TABLE s3_events(
   id SERIAL PRIMARY KEY,
   key_id VARCHAR NOT NULL,
   event_type VARCHAR NOT NULL,
   bucket_name VARCHAR NOT NULL,
   file_name VARCHAR NOT NULL,
   file_type VARCHAR NOT NULL,
   file_size INTEGER NOT NULL,
   time TIMESTAMP NOT NULL DEFAULT NOW()
);