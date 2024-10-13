# Phony targets
.PHONY: all clean linux apple x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu linux-arm64-v8

# Use the correct Docker Compose command
DOCKER_COMPOSE := docker compose

# Default target
all: linux apple

# Clean target
clean:
	rm -rf target
	$(DOCKER_COMPOSE) down --rmi all --volumes --remove-orphans

# Linux builds
linux: x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu linux-arm64-v8

x86_64-unknown-linux-gnu:
	$(DOCKER_COMPOSE) run --rm x86_64-unknown-linux-gnu cargo build --release --target x86_64-unknown-linux-gnu

aarch64-unknown-linux-gnu:
	$(DOCKER_COMPOSE) run --rm aarch64-unknown-linux-gnu cargo build --release --target aarch64-unknown-linux-gnu

linux-arm64-v8:
	$(DOCKER_COMPOSE) run --rm linux-arm64-v8 cargo build --release --target aarch64-unknown-linux-gnu

# Apple builds
apple:
	./build_apple_targets.sh

# Verify all builds
verify:
	@echo "Verifying Linux builds..."
	@$(DOCKER_COMPOSE) run --rm x86_64-unknown-linux-gnu ls -l /usr/src/myapp/target/x86_64-unknown-linux-gnu/release/libdji_log_parser.a || true
	@$(DOCKER_COMPOSE) run --rm aarch64-unknown-linux-gnu ls -l /usr/src/myapp/target/aarch64-unknown-linux-gnu/release/libdji_log_parser.a || true
	@$(DOCKER_COMPOSE) run --rm linux-arm64-v8 ls -l /usr/src/myapp/target/aarch64-unknown-linux-gnu/release/libdji_log_parser.a || true
	@echo "Verifying Apple builds..."
	@ls -l target/aarch64-apple-darwin/release/libdji_log_parser.a || true
	@ls -l target/x86_64-apple-darwin/release/libdji_log_parser.a || true