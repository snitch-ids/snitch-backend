run_dev:
	cd docker && docker compose up redis-stack --no-recreate -d && cd -
	SNITCH_PASSWORD_SECRET=asdfasdf RUST_BACKTRACE=1 RUST_LOG=debug  cargo run -- local-dev.env

docker_build:
	docker build -t emrius11/snitch-backend:main .

docker_push:
	docker push emrius11/snitch-backend:main
