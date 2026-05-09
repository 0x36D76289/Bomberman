all: release

debug:
	cargo build
	ln -s -f target/debug/Bomberman Bomberman

release:
	cargo build -r
	ln -s -f target/release/Bomberman Bomberman

docker:
	docker build --platform linux/amd64 --tag bomberman .
	docker run --rm -v .:/bomberman bomberman

clean:
	cargo clean
	rm Bomberman