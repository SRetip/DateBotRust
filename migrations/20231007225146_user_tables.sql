-- Add migration script here
CREATE TABLE Users (
    chat_id BIGINT PRIMARY KEY,
    age INT,
    gender CHAR(10),
    city VARCHAR(255),
    name VARCHAR(255),
    telegram_name VARCHAR(255),
    about TEXT,
    interested_min_age INT,
    interested_max_age INT,
    interested_gender CHAR(10)
);
