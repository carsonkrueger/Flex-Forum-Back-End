-- Add migration script here
CREATE UNIQUE INDEX users_index ON user_management.users (id, username);
