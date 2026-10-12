NAME	= Bomberman
DOCKER	?= docker

all: release

assets:
	./scripts/fetch_assets.sh

pack-assets:
	./scripts/pack_assets.sh

release: assets
	cargo build --release
	ln -sf target/release/$(NAME) $(NAME)

run: release
	./$(NAME)

package: release
	./scripts/package.sh

install: package
	./scripts/install.sh

uninstall:
	./scripts/install.sh --uninstall

docker:
	$(DOCKER) build --pull --tag bomberman .
	$(DOCKER) run --rm -e CARGO_TARGET_DIR=target/docker -v "$(CURDIR)":/bomberman bomberman

clean:
	cargo clean
	rm -f $(NAME)

fclean: clean
	rm -rf assets dist

re: fclean all

.PHONY: all release assets pack-assets run package install uninstall docker clean fclean re
