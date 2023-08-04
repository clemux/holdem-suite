-- Your SQL goes here
CREATE TABLE actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    hand_id TEXT NOT NULL REFERENCES hands(id),
    player_name TEXT NOT NULL ,
    action_type TEXT NOT NULL ,
    amount DOUBLE NULL,
    is_all_in INTEGER NOT NULL ,
    street TEXT NOT NULL
);