SHELL=/bin/bash

up: mines.db build run

mines.db:
	touch mines.db

build:
	docker build -t minesweeper-io .

run:
	docker run -dp 127.0.0.1:8080:8080 --mount type=bind,source=./mines.db,target=/app/mines.db --name minesweeper-io minesweeper-io

start:
	docker start minesweeper-io

stop:
	docker stop minesweeper-io

clean: stop
	docker rm minesweeper-io
