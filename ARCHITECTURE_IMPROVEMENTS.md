# Scry: Architektur-Analyse & Roadmap

Dieses Dokument bietet eine Übersicht über die Architektur-Strategie von Scry.

---

## 1. Backend: Repository Pattern (DRY & Sicherheit)
**Status: ✅ Umgesetzt**
Einführung der Repository-Schicht unter `src/repository/` zur Kapselung der `user_id`-Filterung.

---

## 2. Backend: Modularisierung der Handler
**Status: ✅ Umgesetzt**
Aufspaltung der `handlers.rs` in ein Modul-System unter `src/handlers/`.

---

## 3. Plugin-System: Semantische Host-API (SDK Logic Migration)
**Status: ⏳ Offen**
- **Problem:** SDK implementiert SQL-Logik (z.B. `count_over_time`).
- **Lösung:** Verschiebung der Logik in den Host. Das SDK sollte nur typsichere WIT-Funktionen aufrufen.

---

## 4. Fullstack: Shared Types
**Status: ⏳ Teilweise** (TS-Typen vorhanden, aber `ts-rs` Integration noch nicht vollständig automatisiert).

---

## 5. Frontend: Modernes Data-Fetching (TanStack Query)
**Status: ✅ Umgesetzt**
Vollständige Migration auf Svelte Query, inklusive zentraler Query Keys und reaktivem SSE-Streaming.

---

## 6. Plugin-Sicherheit: Sandbox-Härtung
**Status: ✅ Umgesetzt**
Aktivierung des `ResourceLimiter` für RAM-Begrenzung (256MB) und CPU-Fuel-Limiting (1M Zyklen).

---

## 7. Observability: Tracing & Monitoring
**Status: ⏳ Offen**
Nutzung von `#[tracing::instrument]` in allen Service- und Plugin-Methoden für detaillierte Logs.

---

## 8. Reliable Ingestion: Background Worker
**Status: ⏳ Offen**
Einführung einer `tokio::sync::mpsc` Queue mit Worker-Loop für Retries und Backpressure-Schutz.

---

## 9. Datenbank-Integrität: JSON-Validierung
**Status: ⏳ Offen**
Umstellung auf `TEXT CHECK (json_valid(...))` für `payload`, `metadata` und `context`.

---

## 10. Frontend: Design Tokens & Styling
**Status: ⏳ Offen**
Zentralisierung von UI-Werten (Farben, Abstände) in CSS-Variablen in `app.css`.

---

## 11. Security: Multi-Tenant Rate Limiting
**Status: ⏳ Offen (Hohe Priorität)**
Integration einer Middleware (z.B. `tower-governor`) zur Begrenzung von Anfragen pro API-Key.

---

## 12. Security: Secure Session Management
**Status: ⏳ Offen**
Umstellung auf kurzlebige JWTs und `httpOnly` Cookies für das Frontend (XSS-Schutz).

---

## 13. Resilience: Scheduler Exponential Backoff
**Status: ⏳ Offen**
Fehler-Zähler pro Plugin/User. Bei Fehlern wird das Poll-Intervall exponentiell vergrößert.

---

## 14. Performance: Wasm Instance Pooling
**Status: ⏳ Offen**
Pool für initialisierte Wasm-Instanzen zur Reduzierung der Latenz beim Ingest.

---

## 15. Stability: SQLite Busy Timeouts
**Status: ✅ Umgesetzt**
Konfiguration von `.busy_timeout(5s)` zur Vermeidung von "database is locked" Fehlern.

---

## 16. Search UX: Advanced FTS5 Features
**Status: ⏳ Offen**
Nutzung von `snippet()` und BM25-Scoring für bessere Suchergebnisse.

---

## 17. Plugin Evolution: State Migrations
**Status: ⏳ Offen**
Erweiterung der `scry.wit` um einen `on-update(old_version)` Hook für Daten-Migrationen.

---

## 18. Mobile Experience: PWA
**Status: ⏳ Offen**
Integration von `vite-plugin-pwa` für ein natives Gefühl auf Mobilgeräten.

---

## 19. Data Ownership: Export & Backup
**Status: ⏳ Offen**
API für Event-Streams (JSONL/CSV) und Nutzung der SQLite Online-Backup-API.

---

## 20. Knowledge Graph: Semantic Query DSL
**Status: ⏳ Offen**
Rust-basierte Fluent API für komplexe Graph-Abfragen über Entitäten und Beziehungen.

---

## 21. Privacy: Encryption at Rest
**Status: ⏳ Offen**
Integration von SQLCipher zur Verschlüsselung der `scry.db` auf der Festplatte.

---

## 22. Scalability: Semantic Materialized Views
**Status: ⏳ Offen**
Inkrementelle Aggregation von Metriken im Hintergrund für blitzschnelle Dashboards.

---

## 23. Infrastruktur: Docker & Docs
- **Docker:** Multi-Stage Build.
- **Documentation:** Architektur-Guide & README.
- **Testing:** Playwright E2E Suite.
