-- Add up migration script here

CREATE TABLE
    IF NOT EXISTS generated_values (
        id CHAR(36) PRIMARY KEY NOT NULL,
        num TINYINT
    );

CREATE TABLE 
    IF NOT EXISTS users (
        id CHAR(36) PRIMARY KEY NOT NULL,
        username VARCHAR(100),
        password VARCHAR(100)
);

