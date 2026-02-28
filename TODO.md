# Scry TODO List

## 🛡️ Phase 1: Foundation & Security
- [x] **API Authentication**: Implement API-Key based access. ✅
- [x] **SQLite Optimization**: WAL-Mode and performance tweaks. ✅
- [x] **Plugin State API**: Persistent memory for plugins. ✅
- [ ] **Field-Level Encryption**: Support for encrypting sensitive payload fields at rest.
- [ ] **Structured Logging**: Let plugins emit logs that appear in the main Scry log.
- [ ] **Robust Error Handling**: Replace remaining `.unwrap()` calls with proper error types.

## 🔌 Phase 2: Plugin Ecosystem & SDK (The "Kernel" Approach)
- [x] **Capability Manifests**: Plugins declare identity and basic metadata. ✅
- [x] **High-level SDK**: Fully automate WASM/WIT via `scry_plugin!` macro. ✅
- [x] **Generic Query API**: Flexible JSON-based query DSL. ✅
- [x] **Granular Polling**: Scheduler respects individual plugin intervals. ✅
- [ ] **Semantic Discovery API**: Host provides a catalog of available data capabilities (e.g., "Who provides temperature?").
- [ ] **Plugin Hot-Reload**: Done! ✅ (Stable and reactive).
- [ ] **Sandboxed File Storage**: Private, persistent directory for plugins.

## 📊 Phase 3: Visualization & Interactivity
- [ ] **Web Dashboard**: Built-in UI using standard widgets rendered from plugin data.
- [ ] **Dynamic Forms API**: Plugins define UI schemas for data entry.
- [ ] **Real-time Feed**: WebSockets for live monitoring.
- [ ] **Universal Search**: Global search API across all event categories.

## 🧠 Phase 4: Intelligence & Automation
- [x] **Temporal Join Engine**: Correlate different data streams via `JoinNearest`. ✅
- [x] **Centralized Correlation Service**: API-driven joins between arbitrary categories. ✅
- [ ] **Cross-Plugin Correlation**: Use Semantic Discovery to link data without hardcoding categories.
- [ ] **Anomaly Detection**: Statistical background tasks to find unusual patterns.
- [ ] **Goal & Habit Tracking**: Trigger actions/notifications based on data trends.
- [ ] **Personal LLM Chat**: Integration with local LLMs to query logs.

## 📱 Phase 5: Clients & Integration
- [ ] **Scry CLI**: Command-line tool for management and manual logging.
- [ ] **Official Importers**: High-quality plugins for Spotify, Last.fm, Apple Health, etc.
- [ ] **IoT Bridge SDK**: Lightweight SDK for ESP32/Arduino integration.
