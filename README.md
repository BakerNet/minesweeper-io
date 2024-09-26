# Minesweeper-io

Multiplayer minesweeper web app built in [Rust](https://www.rust-lang.org/) with [Leptos](https://leptos.dev/), [Axum](https://github.com/tokio-rs/axum), and [Tailwind CSS](https://tailwindcss.com/)

## Live Demo

[https://mines.hansbaker.com](https://mines.hansbaker.com)

Deployed on [Fly.io](https://fly.io/)

## Project Status

Playable (see Live Demo site above)

Features:

- Log in with OAuth2 - Google, Reddit, and Github Providers
  - Only username/login is used as an internal ID.  No other user data is used or stored (and internal ID is not accessible via UI)
- Create Multiplayer & Singleplayers games
  - Any Custom game with 1 max player is considered Singleplayer
- Spectate any active game
- Join Multiplayers games created by someone else
- Watch Replay of a game
  - Singleplayer Replays always have flags
  - Multiplayer Replays only logged-in users their own flags
- View Replay analysis
  - Highlights guaranteed plays
- Logged in users have profile page to see their game history or set their display name
- Basic keyboard commands (not yet mappable) as alternative to mouse buttons

## Minesweeper-lib

[minesweeper-lib](minesweeper-lib) contains the core minesweeper game logic as a library

## Run the project

Requires `.env` (see `.env.example`) and `db/mines.db` (can use `touch` or `sqlite3` to create)

### Develop

```
cargo leptos watch
```

Using development setup, runs on `http://localhost:3000`

For more information see [web README](web/README.md)

### Build & Run in Docker

```
make
```

Using built docker setup, runs on `http://localhost:8080`

To stop and clean up the Docker container:

```
make clean
```
