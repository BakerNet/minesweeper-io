SHELL=/bin/bash

all: .minesweeper-up

.minesweeper-up: mines.db
	docker build -t minesweeper-io .
	docker run -dp 127.0.0.1:8080:8080 \
		--mount type=bind,source=./db/mines.db,target=/app/db/mines.db \
		--mount type=bind,source=./.env,target=/app/.env \
		--name minesweeper-io minesweeper-io
	touch .minesweeper-up

mines.db:
	touch db/mines.db

start:
	docker start minesweeper-io

stop:
	docker stop minesweeper-io

clean: stop
	docker rm minesweeper-io
	rm .minesweeper-up
