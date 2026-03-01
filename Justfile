# Scry Monorepo Task Runner

# --- Main Entry Points with Subcommands ---

# Build components (options: backend, frontend, all)
build component="all":
	@case "{{component}}" in \
		backend) just _build-backend ;; \
		frontend) just _build-frontend ;; \
		plugins) just _build-plugins ;; \
		all) just _build-backend _build-frontend _build-plugins ;; \
		*) echo "Unknown component: {{component}} (Use: backend, frontend, plugins, all)"; exit 1 ;; \
	esac

# Start dev servers (options: backend, frontend, all)
dev component="all":
	@case "{{component}}" in \
		backend) just _dev-backend ;; \
		frontend) just _dev-frontend ;; \
		all) echo "Starting everything..." && (just _dev-backend & just _dev-frontend & wait) ;; \
		*) echo "Unknown component: {{component}}"; exit 1 ;; \
	esac

# Run tests/checks (options: backend, frontend, all)
test component="all":
	@case "{{component}}" in \
		backend) just _test-backend ;; \
		frontend) just _test-frontend ;; \
		all) just _test-backend _test-frontend ;; \
		*) echo "Unknown component: {{component}}"; exit 1 ;; \
	esac

# Install all dependencies
install:
	cargo fetch
	pnpm --prefix web install

# --- Internal Commands (hidden) ---

_build-backend:
	cargo build --release

_build-frontend:
	pnpm --prefix web build

_dev-backend:
	cargo run -p scry-core

_dev-frontend:
	pnpm --prefix web dev

_test-backend:
	cargo test

_test-frontend:
	pnpm --prefix web check

_build-plugins:
	@mkdir -p plugins
	cargo build --release -p scry-music-plugin --target wasm32-wasip2
	cp target/wasm32-wasip2/release/scry_music_plugin.wasm plugins/music.wasm
	cargo build --release -p scry-weather-plugin --target wasm32-wasip2
	cp target/wasm32-wasip2/release/scry_weather_plugin.wasm plugins/weather.wasm
	cargo build --release -p scry-music-enricher --target wasm32-wasip2
	cp target/wasm32-wasip2/release/scry_music_enricher.wasm plugins/enricher.wasm
