-- Your SQL goes here
CREATE TABLE hands (
    id TEXT PRIMARY KEY NOT NULL,
    hole_card_1 VARCHAR(2) NOT NULL,
    hole_card_2 VARCHAR(2) NOT NULL,
    tournament_id INT NULL,
    cash_game_name TEXT NULL,
    datetime TEXT NOT NULL,
    button INTEGER NOT NULL,
    max_players INTEGER NOT NULL,
    hero TEXT NOT NULL
)