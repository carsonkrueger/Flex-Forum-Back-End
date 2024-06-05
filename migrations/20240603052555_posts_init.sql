-- Add migration script here
CREATE SCHEMA IF NOT EXISTS post_management;

CREATE TABLE IF NOT EXISTS post_management.posts (
    id bigint GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
    user_id bigint REFERENCES user_management.users (id) ON DELETE CASCADE,
    image_ids smallint[5],
    description varchar(1000)
);
