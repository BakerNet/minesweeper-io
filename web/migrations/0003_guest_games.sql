PRAGMA foreign_keys=off;

-- Remove not null constraint from owner
alter table games rename to games_old;

create table if not exists games
(
    game_id      text not null primary key,
    owner        integer, -- users.id (optional)
    rows         integer not null,
    cols         integer not null,
    num_mines    integer not null,
    max_players  integer not null,
    is_started   integer not null default 0,
    is_completed integer not null default 0,
    final_board  text,
    FOREIGN KEY(owner) REFERENCES users(id)
);

insert into games select * from games_old;

-- Remove not null constraint from player
alter table players rename to players_old;

create table if not exists players
(
    game_id      text not null,
    user         integer, -- users.id (optional)
    player       integer not null,
    dead         integer not null default 0,
    score        integer not null default 0,
    FOREIGN KEY(game_id) REFERENCES games(game_id),
    FOREIGN KEY(user) REFERENCES users(id)
);

insert into players select * from players_old;

PRAGMA foreign_keys=on;
