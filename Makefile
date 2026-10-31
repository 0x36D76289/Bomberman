.PHONY: all assets release run install uninstall clean

all: release

assets:
	./scripts/fetch_assets.sh

release: assets
	cargo build --release
	ln -sf target/release/Bomberman Bomberman

run: release
	./Bomberman

install: release
	./scripts/install.sh

uninstall:
	./scripts/install.sh --uninstall

clean:
	cargo clean
	rm -f Bomberman
