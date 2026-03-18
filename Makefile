
build:
	cd run && cargo build --release

publish:
	cd run && set -a && source .env && set +a && cargo run -- publish

dry-run:
	cd run && set -a && source .env && set +a && cargo run -- dry-run

docker-build:
	docker build -t envrac-rust ./run
