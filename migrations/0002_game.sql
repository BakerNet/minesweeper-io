-- Create games table.
create table if not exists games
(
    game_id      text not null primary key,
    owner        integer not null, -- users.id
    rows         integer not null,
    cols         integer not null,
    num_mines    integer not null,
    max_players  integer not null,
    is_started   integer not null default 0,
    is_completed integer not null default 0,
    final_board  text,
    FOREIGN KEY(owner) REFERENCES users(id)
);

create table if not exists players
(
    game_id      text not null,
    user         integer not null, -- users.id
    player       integer not null,
    dead         integer not null default 0,
    score        integer not null default 0,
    FOREIGN KEY(game_id) REFERENCES games(game_id),
    FOREIGN KEY(user) REFERENCES users(id)
);
