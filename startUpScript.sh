#!/bin/bash

service postgresql start
# shellcheck disable=SC2117
su postgres -c "psql -d postgres -c \"CREATE TABLE s3_events(
                                        id SERIAL PRIMARY KEY,
                                        key_id VARCHAR NOT NULL,
                                        event_type VARCHAR NOT NULL,
                                        bucket_name VARCHAR NOT NULL,
                                        file_name VARCHAR NOT NULL,
                                        file_type VARCHAR NOT NULL,
                                        file_size VARCHAR NOT NULL,
                                        time TIMESTAMP NOT NULL DEFAULT NOW()+3*INTERVAL '1 hour'
                                     );\" "
su postgres -c "psql -d postgres -c \"CREATE TABLE s3_account(
                                         id SERIAL PRIMARY KEY,
                                         key_id VARCHAR NOT NULL,
                                         key_secret VARCHAR NOT NULL,
                                         region VARCHAR NOT NULL,
                                         bucket_name VARCHAR NOT NULL
                                       );\" "
su postgres -c "psql -d postgres -c \"ALTER USER postgres PASSWORD 'postgres';\""

