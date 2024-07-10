-- Add migration script here
CREATE SCHEMA IF NOT EXISTS post_management;

CREATE TYPE post_type AS ENUM ('images', 'workout');

CREATE TABLE IF NOT EXISTS post_management.posts (
    id bigint GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
    username varchar(32) NOT NULL REFERENCES user_management.users (username) ON DELETE CASCADE,
    num_images smallint CHECK (num_images < 6),
    description varchar(1000),
    post_type post_type NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    deactivated_at timestamp with time zone DEFAULT NULL
);
