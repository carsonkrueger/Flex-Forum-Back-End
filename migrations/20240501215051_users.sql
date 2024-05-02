-- Add migration script here

CREATE SCHEMA IF NOT EXISTS user_management;

CREATE TYPE hash_scheme AS ENUM ('argon2');

CREATE TABLE IF NOT EXISTS user_management.users (
    first_name VARCHAR(32) NOT NULL,
    last_name VARCHAR(32) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(32) NOT NULL,
    password_hash VARCHAR(60) NOT NULL,
    salt VARCHAR(16) NOT NULL,
    hash_scheme hash_scheme NOT NULL,
    created_at TIMESTAMP NOT NULL,
    deactivated_at TIMESTAMP
)
