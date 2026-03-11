
hello:
	echo "Hello, World!"

start:
	cd run && set -a && source .env && set +a && cargo run