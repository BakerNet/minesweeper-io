# Minesweeper-io

Multiplayer minesweeper web app built in [Rust](https://www.rust-lang.org/) with [Leptos](https://leptos.dev/), [Axum](https://github.com/tokio-rs/axum), and [Tailwind CSS](https://tailwindcss.com/)

## Live Demo

[https://mines.hansbaker.com](https://mines.hansbaker.com)

Deployed on [Fly.io](https://fly.io/)

## Project Status

Early development - very much lacking in features, but in a playable demo state

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
