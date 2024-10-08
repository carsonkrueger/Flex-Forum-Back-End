-- Add migration script here

CREATE TABLE IF NOT EXISTS post_management.seen_posts (
    id bigint GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
    post_id bigint NOT NULL REFERENCES post_management.posts (id) ON DELETE CASCADE,
    username varchar(32) NOT NULL REFERENCES user_management.users (username) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX ON post_management.seen_posts(username);
