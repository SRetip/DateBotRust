-- Add migration script here
CREATE TABLE Duplets (
    id VARCHAR(36) PRIMARY KEY,
    first_user_id BIGINT,
    first_user_react VARCHAR(255),
    second_user_id BIGINT,
    second_user_react VARCHAR(255),
    status VARCHAR(255),
    FOREIGN KEY (first_user_id) REFERENCES Users(chat_id),
    FOREIGN KEY (second_user_id) REFERENCES Users(chat_id)
);
