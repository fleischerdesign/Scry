# Scry Codebase Audit Report
**Date:** March 3, 2026
**Focus:** Architectural cleanliness, "Standardized Semantic Layer" adherence, technical debt, and hacks.

## 1. Frontend: Incomplete Agnosticism in `Widget.svelte`
*   **Location:** `web/src/lib/components/Widget.svelte`
*   **Issue:** The widget contains a legacy `resolvePath` function that attempts to manually traverse the `payload` JSON object (e.g., prepending `payload.` to paths) if the new `display_value` is missing.
*   **Why it's a problem:** This is a "leaky abstraction" and a hack. It breaks the 100% agnostic nature of the UI. The frontend should not need to know about the internal payload structure.
*   **Recommendation:** Remove the `resolvePath` logic entirely. The frontend should rely strictly on the `display_value` provided by the backend's semantic enrichment.

## 2. Backend: Fragile SQL Queries in `EntityRepository`
*   **Location:** `crates/scry-core/src/repository/entity_repo.rs` (Method: `get_entities_by_type`)
*   **Issue:** The SQL query uses generic string matching for traits: `t.trait_id LIKE '%name' OR t.trait_id LIKE '%title'` and `t.trait_id LIKE '%photo'`.
*   **Why it's a problem:** This is imprecise and ignores the strict semantic vocabulary we established in `scry_plugin_sdk::schema::traits`. It could lead to false positives (e.g., matching a trait named `my.custom/photo_editor`) and is generally a "quick fix" pattern.
*   **Recommendation:** Update the SQL query to explicitly match the constants defined in the SDK (e.g., `scry.visual/photo`, `scry.core/name`).

## 3. Backend: Test Schema Drift
*   **Location:** `crates/scry-core/src/services/event_service.rs` (Tests: `setup_test_db`)
*   **Issue:** The mock database setup in the tests creates an `events` table that does not include the newly added `display_value` column.
*   **Why it's a problem:** Running `cargo test` will likely fail when the repository tries to insert or select from the `events` table, as the mock schema is out of sync with the actual application schema (defined in `event_repo.rs`).
*   **Recommendation:** Update the `CREATE TABLE events` statement in the test setup to include `display_value REAL`.

## 4. Backend: Performance Bottleneck in `EventService`
*   **Location:** `crates/scry-core/src/services/event_service.rs`
*   **Issue:** Methods like `enrich_event_context`, `search_semantic`, and others repeatedly call `self.plugin_manager.get_plugin_manifests().await`. 
*   **Why it's a problem:** This function acquires a read lock (`RwLock`) on the plugin map and clones the manifests every time it's called. For high-throughput event ingestion or search, this creates unnecessary overhead.
*   **Recommendation:** Implement a caching mechanism for manifests within `EventService` or optimize how the semantic mapping is resolved without cloning all manifests for every single event.

## 5. Security & Sandbox: Hardcoded Limits
*   **Location:** `crates/scry-core/src/plugins/context.rs` and `manager.rs`
*   **Issue:** The Wasmtime `ResourceLimiter` correctly enforces limits, but they are hardcoded (`const MAX_MEMORY: usize = 256 * 1024 * 1024;` and 1M Fuel).
*   **Why it's a problem:** While it prevents catastrophic memory leaks, hardcoded limits lack flexibility. Different environments (Raspberry Pi vs. Cloud Server) might need different limits.
*   **Recommendation:** Make these limits configurable via a core system configuration file or environment variables.

## 6. Robustness: Silent Failures in Metric Extraction
*   **Location:** `crates/scry-core/src/services/event_service.rs` (Method: `enrich_event_context`)
*   **Issue:** When extracting the `display_value`, the code uses `.and_then(|v| v.as_f64())`. 
*   **Why it's a problem:** If a plugin incorrectly sends a metric as a string (e.g., `"22.5"`) instead of a number, `as_f64()` will return `None`, and the `display_value` will be silently dropped.
*   **Recommendation:** Implement a slightly more robust parsing fallback here, or strictly validate plugin exports against their schema during initialization.

## Summary
The architecture is fundamentally solid, professional, and heavily decoupled. The "Standardized Semantic Layer" is a massive architectural win. The remaining issues are primarily cleanup tasks: removing legacy fallback logic, aligning SQL queries with the new SDK constants, and fixing the test suite schema.