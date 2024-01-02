-- Create users table.
create table if not exists users
(
    id           integer primary key autoincrement,
    username     text not null unique,
    display_name text unique,
    access_token text not null
);
