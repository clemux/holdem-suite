-- Your SQL goes here
CREATE TABLE seats
(
    hand_id     TEXT    NOT NULL REFERENCES hands (id),
    player_name TEXT    NOT NULL,
    seat_number INTEGER NOT NULL,
    stack DOUBLE NOT NULL,
    bounty DOUBLE NULL,
    card1 TEXT NULL,
    card2 TEXT NULL,
    PRIMARY KEY (hand_id, seat_number)
)