--- Create Game Log ----
create table if not exists game_log
(
    game_id text not null,
    log     text not null,
    FOREIGN KEY(game_id) REFERENCES games(game_id)
);
