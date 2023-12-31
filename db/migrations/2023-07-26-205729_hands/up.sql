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
    hero TEXT NOT NULL,
    ante DOUBLE NULL,
    small_blind DOUBLE NOT NULL,
    big_blind DOUBLE NOT NULL,
    pot DOUBLE NOT NULL,
    rake DOUBLE NULL,
    flop1 VARCHAR(2) NULL,
    flop2 VARCHAR(2) NULL,
    flop3 VARCHAR(2) NULL,
    turn VARCHAR(2) NULL,
    river VARCHAR(2) NULL
)