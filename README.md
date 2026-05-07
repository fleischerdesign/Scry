# Scry

Scry is an agnostic personal data aggregator with a plugin-based architecture. It ingests events from multiple sources (GitHub, Spotify, weather, etc.), enriches them with semantic metadata, and provides a unified timeline, entity graph, and analytics dashboard.

## Development

### Branching Strategy

This project follows a lightweight Git Flow model:

- **`main`** — Stable production branch. Only merged from `develop` via PR. Protected: requires status checks and prohibits direct pushes.
- **`develop`** — Integration branch. All feature work merges here via PR. Protected: requires status checks and PR review.
- **`feature/*`**, **`fix/*`**, **`refactor/*`**, **`docs/*`**, **`chore/*`** — Short-lived branches created from `develop` and merged back via PR.

```text
main          ●─────●─────●  (releases)
              ↑     ↑     ↑
develop       ●──●──●──●──●  (integration)
              ↑  ↑  ↑
feature/x     ●──●  │
feature/y         ●──●
```

### Commit Conventions

All commits must follow the [Conventional Commits](https://www.conventionalcommits.org/) format:

```
type(scope): description
```

Allowed types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `ci`, `style`, `perf`

Common scopes: `core`, `web`, `plugins`, `github`, `spotify`, `weather`, `sdk`, `proto`

### Git Hooks

This project uses Git hooks for pre-commit validation. To activate them:

```bash
git config core.hooksPath .githooks
```

If using the Nix flake (`nix develop` or direnv), hooks are activated automatically.

- **`pre-commit`**: Runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and `pnpm --prefix web check`.
- **`commit-msg`**: Validates that the commit message follows Conventional Commits format.

### CI Pipeline

On every PR and push to `main`, GitHub Actions runs:

| Job | Command |
|---|---|
| **Format** | `cargo fmt --all -- --check` |
| **Clippy** | `cargo clippy -- -D warnings` |
| **Type Check** | `pnpm --prefix web check` (svelte-check + tsc) |

All three checks must pass before merging.

## Architecture

Scry is a monorepo with a Rust workspace and a Svelte frontend.

### Backend (Rust)

| Crate | Purpose |
|---|---|
| `scry-core` | Main application: Axum server, SQLite via sqlx, WASM plugin runtime |
| `scry-proto` | Shared types between core and plugins (Event, EntityRef) |
| `scry-plugin-sdk` | Plugin development kit (macros, host bindings, semantic vocabulary) |
| `scry-github-plugin` | GitHub data ingestion plugin (compiles to WASM) |
| `scry-spotify-plugin` | Spotify data ingestion plugin (compiles to WASM) |
| `scry-weather-plugin` | Weather data ingestion plugin (compiles to WASM) |

### Frontend (Svelte + TypeScript)

- **Framework**: Svelte 5 with runes ($state, $derived, $props)
- **Bundler**: Vite 8 with Rolldown
- **Styling**: Tailwind CSS v4 + daisyUI
- **Data fetching**: TanStack Query (svelte-query)
- **Charts**: Chart.js via svelte-chartjs
- **Type safety**: Types auto-generated from Rust via ts-rs

### Development Environment

Uses Nix flakes for reproducible development environments:

```bash
nix develop  # or direnv allow
```

The dev shell provides: Rust (stable with WASM targets), Node.js, pnpm, SQLite, OpenSSL, and Just.

### Task Runner

Use `just` for common operations:

```bash
just build            # Build all components
just build backend    # Build Rust only
just build frontend   # Build frontend only
just build plugins    # Build WASM plugins

just dev              # Start dev servers (backend + frontend)

just test             # Run all checks
just test backend     # cargo test
just test frontend    # svelte-check + tsc
```

## License

This project is licensed under the GNU General Public License v3.0.
