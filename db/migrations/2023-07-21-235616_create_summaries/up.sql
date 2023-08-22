-- Your SQL goes here
CREATE TABLE summaries (
    id integer PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    buyin DOUBLE NOT NULL,
    date TEXT NOT NULL,
    play_time TEXT NOT NULL,
    entries INTEGER NOT NULL,
    mode TEXT NOT NULL,
    tournament_type TEXT NOT NULL,
    speed TEXT NOT NULL,
    finish_place INTEGER NOT NULL,
    won DOUBLE NULL
)