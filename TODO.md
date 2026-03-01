# Scry Project Roadmap

## Phase 1: Core Foundation (In Progress)
- [x] Multi-tenant event storage (SQLite)
- [x] Wasmtime Plugin System (WASI P2)
- [x] Agnostic UI Architecture (Plugins provide display summaries)
- [x] Universal FTS5 Search (Events + Entities)
- [x] **Semantic Knowledge Graph (Triple Store)**
- [x] **Agnostic Discovery Engine (Correlation Discovery)**
- [x] **User Identity Node (`self` entity in graph)**

## Phase 2: Intelligence & Visualization
- [ ] **Semantic Brain** (Local LLM integration via Ollama for graph queries)
- [ ] **Graph Visualizer** (Force-directed network view of entities)
- [ ] **Data Importer** (Import legacy data from Spotify/Google/Apple)
- [ ] Automated daily "Life Insights" reports
- [ ] Correlation Heatmaps in Discovery Lab

## Phase 3: Ecosystem
- [ ] Mobile companion app (Native/Flutter)
- [ ] Plugin marketplace
- [ ] End-to-end encrypted sync between nodes

## Completed Today
- Implemented full Subject-Predicate-Object relationship system.
- Refactored frontend to be 100% agnostic using `display_title`.
- Added automated correlation discovery (Pearson coefficient) between numeric traits.
- Integrated User Profile as a first-class entity in the Knowledge Graph.
- Fixed WASI P2 stability issues and SQL schema multi-tenancy alignment.
