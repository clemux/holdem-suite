-- Your SQL goes here
CREATE TABLE hands (
    id TEXT PRIMARY KEY NOT NULL,
    hole_card_1 VARCHAR(2) NOT NULL,
    hole_card_2 VARCHAR(2) NOT NULL,
    tournament_id INT NULL,
    datetime TEXT NOT NULL
)