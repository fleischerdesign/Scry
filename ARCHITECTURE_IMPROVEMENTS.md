# Scry: Architektur-Analyse & Verbesserungsvorschläge

Dieses Dokument fasst die Analyse des aktuellen Code-Stands zusammen und bietet konkrete Strategien, um die Codebase **DRYer (Don't Repeat Yourself)**, **modularer** und **robuster** zu gestalten.

---

## 1. Backend: Repository Pattern (DRY & Sicherheit)

### Problem
Die `user_id` zur Filterung der Mehrmandantenfähigkeit wird in fast jeder SQL-Abfrage manuell in `handlers.rs` und `event_service.rs` eingebunden. Dies ist redundant und birgt das Risiko von "Data Leaks", falls eine Filterung vergessen wird.

### Lösung: Repository-Wrapper
Einführung einer Schicht, die den Datenbank-Kontext inklusive `user_id` kapselt.

```rust
// Vorschlag für eine Repository-Struktur
pub struct EventRepository<'a> {
    pool: &'a SqlitePool,
    user_id: i64,
}

impl<'a> EventRepository<'a> {
    pub fn new(pool: &'a SqlitePool, user_id: i64) -> Self {
        Self { pool, user_id }
    }

    pub async fn list_by_category(&self, category: &str, limit: u32) -> Result<Vec<DbEvent>> {
        sqlx::query_as!(
            DbEvent,
            "SELECT * FROM events WHERE user_id = ? AND category = ? LIMIT ?",
            self.user_id,
            category,
            limit
        )
        .fetch_all(self.pool)
        .await
    }
}
```

---

## 2. Backend: Modularisierung der Handler

### Problem
Die Datei `crates/scry-core/src/handlers.rs` fungiert als "God-Object". Mit über 570 Zeilen enthält sie Logik für Auth, Dashboards, Plugins, Suche und Analytics.

### Lösung: Thematische Aufteilung
Aufspaltung in ein Modul-System unter `src/handlers/`:
- `auth.rs`: Registrierung, Login, API-Keys.
- `events.rs`: Ingest, Timeline, Detail-Ansichten.
- `dashboards.rs`: CRUD für Dashboards und Widgets.
- `plugins.rs`: Plugin-Management, Config, Reports.
- `analytics.rs`: Korrelationen, Stats, Discovery.

---

## 3. Plugin-System: Semantische Host-API

### Problem
Die aktuelle `query`-Funktion in `host_impl.rs` erlaubt Plugins das Ausführen von rohem SQL (zwar begrenzt auf SELECT und via CTE gefiltert). Dies ist schwer zu warten und für Plugin-Autoren fehleranfällig.

### Lösung: WIT-basierte semantische API
Statt SQL sollten Plugins High-Level-Funktionen in der `scry.wit` nutzen.

```wit
// scry.wit Vorschlag
interface host {
    get-events: func(filter: event-filter) -> list<event>;
    get-entities: func(type: string) -> list<entity>;
    get-state: func(key: string) -> option<string>;
}
```
*Vorteil:* Maximale Sicherheit, da das Host-System kontrolliert, welche Daten wie abgefragt werden.

---

## 4. Fullstack: Shared Types (End-to-End Typsicherheit)

### Problem
Frontend-Modelle (Svelte) und Backend-Modelle (Rust) werden separat gepflegt. API-Antworten im Frontend nutzen oft `any`.

### Lösung: Typ-Generierung
Einsatz von Tools wie `ts-rs` oder `specta` im Rust-Backend.

```rust
// In Rust
#[derive(Serialize, TS)]
#[ts(export, export_to = "../../web/src/lib/types/")]
pub struct Event { ... }
```
*Vorteil:* Änderungen am Rust-Modell führen bei der Kompilierung sofort zu Fehlern im Svelte-Frontend, falls die Typen nicht mehr matchen.

---

## 5. Frontend: Modernes Data-Fetching

### Problem
Der manuelle `ScryAPI`-Wrapper in `api.ts` bietet kein Caching, keine automatischen Retries und kein konsistentes State-Management für Loading/Error-Zustände.

### Lösung: TanStack Query (Svelte Query)
Kombination aus `api.ts` (nur für Fetching) und `@tanstack/svelte-query` für die Logik.

```typescript
// Beispiel in einer Komponente
const query = createQuery({
    queryKey: ['events', category],
    queryFn: () => api.getData(`category/${category}`)
});
```
*Vorteil:* Reduziert den Code in `.svelte.ts` Files massiv, da Caching und Re-fetching automatisch gehandhabt werden.

---

## 6. Plugin-Sicherheit: Wasmtime-Sandboxing (Verifiziert)

### Status Quo
In `crates/scry-core/src/plugins/manager.rs` wird eine Standard-`Config` für Wasmtime erstellt. Es fehlen explizite Ressourcen-Limits.

### Lösung: Fuel & Memory Limits
Einführung von harten Limits, um "Denial of Service" durch fehlerhafte Plugins zu verhindern.

```rust
// Vorschlag für PluginManager::new
let mut config = Config::new();
config.consume_fuel(true); // Erlaubt das Limitieren von CPU-Zyklen
config.static_memory_maximum_size(512 * 1024 * 1024); // Max 512MB RAM
```
*Vorteil:* Ein Plugin kann niemals den gesamten RAM des Hosts fressen oder in einer Endlosschleife den Prozessor blockieren.

---

## 7. Observability: Tracing & Monitoring (Verifiziert)

### Status Quo
Kritische Pfade in `manager.rs` (z.B. `run_ingest_pipeline`) und `event_service.rs` nutzen keine Tracing-Spans. Das macht das Debugging von Fehlern in der Plugin-Kette schwierig.

### Lösung: Structured Logging (Tracing)
Nutzung von `#[tracing::instrument]` in allen Service- und Plugin-Methoden.

```rust
#[tracing::instrument(skip(self, event), fields(event_id = %event.id, user_id = %user_id))]
pub async fn run_ingest_pipeline(&self, user_id: i64, mut event: ScryEvent) -> Result<ScryEvent> { ... }
```
*Vorteil:* In den Logs lässt sich genau nachverfolgen, welcher Schritt in der Pipeline wie lange gedauert hat und wo genau ein Fehler aufgetreten ist.

---

## 8. Reliable Ingestion: Background Worker (Verifiziert)

### Status Quo
In `event_service.rs` werden Enricher via `tokio::spawn` benachrichtigt. Es gibt keine Queue und kein Fehlermanagement.

### Lösung: Internal Message Queue
Einführung einer `tokio::sync::mpsc` Queue mit einem dedizierten Worker-Loop.
*Vorteil:* Unterstützung für Retries bei Plugin-Fehlern und Schutz vor Überlastung (Backpressure), falls zu viele Events gleichzeitig eingehen.

---

## Status der Infrastruktur (Review abgeschlossen)

- **Justfile:** ✅ **Optimal.** WASM-Builds und Dev-Server sind sauber integriert.
- **Flake.nix:** ✅ **Optimal.** Alle Toolchains (WASM, Node, Rust) sind korrekt konfiguriert.
- **Database Migrations:** ⚠️ **Wartung empfohlen.** Bei Gelegenheit "Squashing" der 18 Migrations-Dateien durchführen.

---

## 9. Datenbank-Integrität: JSON-Validierung (Verifiziert)

### Status Quo
In den Migrationen (z. B. `20240227000004_multi_tenancy.sql`) werden `payload` und `metadata` als `BLOB` definiert. Es gibt keine Prüfung auf SQL-Ebene, ob der Inhalt valides JSON ist.

### Lösung: CHECK-Constraints
Umstellung auf `TEXT` (oder `JSON` Alias in SQLite) mit `json_valid()` Check.

```sql
ALTER TABLE events ADD COLUMN payload_new TEXT CHECK (json_valid(payload_new));
```
*Vorteil:* Verhindert "Silent Data Corruption", falls ein Bug im Backend oder in einem Plugin invalides JSON in die Datenbank schreibt.

---

## 10. Frontend: Design Tokens & Konsistentes Styling

### Status Quo
Komponenten wie `Card.svelte` und `Widget.svelte` nutzen oft hartcodierte CSS-Werte (Farben, Abstände). Ein einheitliches System fehlt.

### Lösung: CSS-Variablen (App-weit)
Zentralisierung aller UI-Werte in `web/src/app.css` unter `:root`.

```css
:root {
  --color-bg: #0f172a;
  --color-surface: #1e293b;
  --radius-lg: 0.75rem;
  --spacing-md: 1rem;
}
```
*Vorteil:* Ermöglicht einfachen Dark/Light Mode Support und garantiert, dass alle Komponenten visuell zusammenpassen (Spacing, Rundungen).

---

## 11. Fullstack: Automatisches Error-Reporting

### Status Quo
Der API-Client in `web/src/lib/api.ts` wirft Fehler, die in jeder Svelte-Seite manuell mit `try-catch` abgefangen und an den `ui`-State gemeldet werden müssen.

### Lösung: Globales Error-Handling im API-Wrapper
Integration des `ui.notify()` in die `request`-Methode von `ScryAPI`.

```typescript
if (!response.ok) {
    const error = await response.json().catch(() => ({ error: "Unknown error" }));
    ui.notify("API Fehler", error.error, "error"); // Automatische Notification
    throw new Error(error.error);
}
```
*Vorteil:* Massiv weniger Code in Svelte-Komponenten (DRY) und garantierte Rückmeldung an den User bei jedem API-Fehler.

---

## 12. Plugin-Evolution: State Migrations (Verifiziert)

### Status Quo
Plugins speichern ihren Zustand via `set_state` in der `plugin_state` Tabelle. Wenn ein Plugin aktualisiert wird (neue Version in `manifest.version`), gibt es keinen Mechanismus, um alten State in ein neues Format zu migrieren.

### Lösung: Migration-Hook im WASM-Interface
Erweiterung der `scry.wit` um eine `on-update(old-version: string)` Funktion.
*Vorteil:* Plugins können ihren eigenen gespeicherten Zustand bei Versionsprüngen transformieren, ohne dass der Host die interne Datenstruktur des Plugins kennen muss.

---

## 13. Frontend: Router Auth Guards & Query Params

### Status Quo
Der minimalistische Svelte 5 Router in `router.svelte.ts` prüft keine Berechtigungen. Jede Seite (`Overview.svelte`, `Settings.svelte`, etc.) muss selbst prüfen, ob `auth.isAuthenticated` wahr ist.

### Lösung: Zentralisierte Navigation-Logic
Einführung von "Route Metadata" (z.B. `requiresAuth: true`) im Router.
*Vorteil:* DRY (Don't Repeat Yourself) bei der Zugriffskontrolle und ein saubereres Handling von Deep Links (z.B. Weiterleitung zum Login und zurück zur ursprünglichen Seite nach Erfolg).

---

## 14. Performance: Prioritized Discovery Orchestrator

### Status Quo
Die "Discovery Engine" (Pearson-Korrelationen) läuft aktuell in `tokio::spawn`. Bei großen Datenmengen (z.B. nach einem Import) kann dies zu CPU-Spikes führen, die die API-Responsivität beeinträchtigen.

### Lösung: Priority-Queue für Hintergrund-Tasks
Einführung eines Orchestrators, der Discovery-Tasks mit niedriger Priorität und in kleinen Batches abarbeitet.
*Vorteil:* Der Server bleibt auch während komplexer Analyse-Vorgänge für Benutzeranfragen (UI) reaktionsschnell.

---

## 15. Resilience: Scheduler Exponential Backoff (Verifiziert)

### Status Quo
Der Background-Scheduler in `main.rs` pollt Plugins in festen Intervallen. Schlägt ein Poll fehl (z.B. wegen einer API-Downtime), wird er im nächsten Zyklus (60s) blind wiederholt.

### Lösung: Per-Plugin Backoff-Status
Einführung eines Fehler-Zählers pro Plugin/User-Kombination. Bei Fehlern wird das nächste Poll-Intervall exponentiell vergrößert (z.B. 1m, 2m, 4m, ... bis max 24h).
*Vorteil:* Schont Systemressourcen und verhindert das "Spammen" von externen APIs bei Ausfällen.

---

## 16. Frontend: Vite Environment Integration (Verifiziert)

### Status Quo
In `web/src/lib/api.ts` ist die API-URL `http://127.0.0.1:3000` hart codiert. Dies verhindert Deployments in unterschiedlichen Umgebungen (Docker, Cloud, Local).

### Lösung: Environment Variables (.env)
Nutzung von `import.meta.env.VITE_API_BASE_URL` im API-Client.
*Vorteil:* Scry kann ohne Code-Änderungen auf jedem Server deployed werden; die API-URL wird einfach über die Umgebungsvariablen gesteuert.

---

## 17. Security: Multi-Tenant Rate Limiting (Verifiziert)

### Status Quo
Die `auth_middleware` in `main.rs` validiert API-Keys, begrenzt aber nicht die Anzahl der Anfragen. Ein einzelner User/Client könnte den gesamten Server durch zu viele Requests lahmlegen.

### Lösung: Key-Based Rate Limiting
Integration einer Middleware (z.B. `tower-governor`), die Anfragen pro API-Key limitiert.
*Vorteil:* Garantiert "Fair Use" der Systemressourcen in einem Multi-Tenant-Szenario und schützt vor (unabsichtlichen) DoS-Attacken durch Clients.

---

## 18. Plugin-SDK: SDK-to-Host Logic Migration (Verifiziert)

### Status Quo
Das `scry-plugin-sdk` implementiert aktuell Hilfsfunktionen wie `count_over_time` oder `join_nearest` durch die Generierung von rohem SQL-Code, der an den Host gesendet wird. Dies bricht die Abstraktion der semantischen Host-API.

### Lösung: Semantische Host-Methoden
Verschiebung der SQL-Logik vom SDK in den Host. Das SDK sollte nur noch typsichere Funktionen in der `scry.wit` aufrufen (z.B. `host::get_stats(params)`).
*Vorteil:* Plugins werden unabhängig vom zugrundeliegenden DB-Schema (SQLite, PostgreSQL, etc.) und der Host kann die Abfragen zentral optimieren (Caching, Indizes).

---

## 19. API-Standards: RFC 7807 Problem Details

### Status Quo
Die Fehlerbehandlung in `error.rs` gibt zwar HTTP-Statuscodes und interne Codes zurück, folgt aber keinem Industriestandard für detaillierte Fehlermeldungen.

### Lösung: Implementierung von RFC 7807
Erweiterung der Fehlerantworten um standardisierte Felder wie `type`, `title`, `detail` und `instance`.
*Vorteil:* Verbessert die Integration von Drittanbieter-Clients und Tools, die standardisierte Fehlerformate erwarten, und bietet Entwicklern präzisere Infos zur Fehlerursache.

---

## 20. Onboarding: Comprehensive Documentation

### Status Quo
Dem Projekt fehlt eine zentrale `README.md` und eine Einsteiger-Dokumentation für die komplexe Plugin-Architektur.

### Lösung: Architektur-Guide & Setup-Docs
Erstellung einer Einstiegsdokumentation, die die drei Säulen von Scry erklärt:
1. **Core:** Rust/Axum Backend & Multi-tenancy.
2. **Plugins:** Wasmtime/WASI Sandboxing.
3. **Graph:** Knowledge Graph & Discovery Engine.
*Vorteil:* Senkt die Hürde für neue Mitentwickler und Plugin-Autoren massiv.

---

## 21. Infrastructure: Multi-Stage Docker Deployment

### Status Quo
Es gibt aktuell kein Docker-Setup. Der Nutzer muss Rust, Node, pnpm und alle Abhängigkeiten manuell installieren, um Scry zu starten.

### Lösung: Dockerfile & docker-compose.yml
Erstellung eines optimierten Multi-Stage-Builds:
1. **Frontend Stage:** Baut das Svelte-Frontend (`pnpm build`).
2. **Backend Stage:** Kompiliert das Rust-Backend (`cargo build --release`).
3. **Final Stage:** Ein schlankes Image (z.B. Alpine oder Debian Slim), das nur die Binärdatei und das statische Frontend enthält.
*Vorteil:* Einfaches Deployment auf NAS (z.B. Synology), Raspberry Pi oder Cloud-Servern mit einem einzigen Befehl.

---

## 22. Code Quality: Strict Linting & Formatting

### Status Quo
Das Projekt nutzt keine expliziten Konfigurationsdateien für Code-Style. Dies führt langfristig zu inkonsistentem Code bei mehreren Entwicklern.

### Lösung: Standardisierte Configs
Einführung von `.rustfmt.toml`, `clippy.toml` und `.prettierrc` / `.eslintrc` im Frontend.
*Vorteil:* Automatisierte Prüfung der Code-Qualität im `Justfile` (`just lint`) und in der CI/CD-Pipeline.

---

## 23. Frontend: i18n Readiness (Mehrsprachigkeit)

### Status Quo
Alle UI-Texte ("Dashboard", "Settings", "Explorer") sind hartcodiert im Quelltext hinterlegt. Ein Wechsel der Sprache ist ohne Code-Änderung nicht möglich.

### Lösung: Translation-Keys
Einführung einer i18n-Library (z.B. `svelte-i18n`) oder eines Rune-basierten Stores für Übersetzungen.
*Vorteil:* Erleichtert die Lokalisierung für eine breitere Nutzerbasis und trennt UI-Logik von Inhalten.

---

## 24. Validation: E2E Testing Suite

### Status Quo
Es gibt Unit-Tests im Backend, aber keine automatisierten Tests, die das Zusammenspiel von Frontend und Backend (z.B. Login-Prozess, Widget-Erstellung) validieren.

### Lösung: Playwright Integration
Einführung einer kleinen E2E-Testsuite, die die kritischen Pfade (User Journey) automatisiert durchläuft.
*Vorteil:* Verhindert Regressionen bei UI-Änderungen und stellt sicher, dass die Kernfunktionen nach jedem Update stabil bleiben.

---

## 25. Data Ownership: Export & Backup (Verifiziert)

### Status Quo
Alle Daten liegen in der `scry.db`. Es gibt keine einfache Möglichkeit für den Nutzer, seine gesammelten Daten in einem Standardformat (JSONL/CSV) zu exportieren oder eine konsistente Sicherung im laufenden Betrieb zu erstellen.

### Lösung: Export Service & SQLite Backup API
1. **Export:** Ein API-Endpunkt, der einen Stream von Events generiert.
2. **Backup:** Nutzung der SQLite Online-Backup-API in Rust, um eine konsistente Kopie der DB zu erstellen, während der Server läuft.
*Vorteil:* Maximale Datensouveränität für den Nutzer und Schutz vor Datenverlust.

---

## 26. Search UX: Advanced FTS5 Features (Verifiziert)

### Status Quo
Die Suche in `20240301000002_universal_search.sql` nutzt FTS5, schöpft aber das Potenzial für UX nicht aus (keine Snippets, einfaches Ranking).

### Lösung: Snippets & BM25 Scoring
Anpassung der Suchanfragen in `handlers.rs`, um das `snippet()` Kommando von SQLite zu nutzen und die Ergebnisse nach Relevanz (BM25) zu gewichten.
*Vorteil:* Nutzer sehen direkt in den Suchergebnissen den gematchten Textteil (Highlighting), was die Navigation massiv beschleunigt.

---

## 27. Mobile Experience: Progressive Web App (PWA)

### Status Quo
Scry ist eine reine Web-Anwendung. Auf Mobilgeräten fehlt das "native" Gefühl (Adressleiste stört, kein Offline-Icon).

### Lösung: vite-plugin-pwa
Integration eines Web-Manifests und eines Service-Workers.
*Vorteil:* Scry kann auf dem Smartphone "installiert" werden (Home-Screen-Icon, Vollbild-Modus, Splash-Screen). Dies überbrückt die Zeit, bis eine echte native App verfügbar ist.

---

## 28. Security: Granular Scope Enforcement (Verifiziert)

### Status Quo
API-Keys besitzen ein `scopes` Feld (z.B. `data:read`), das in der Middleware geladen wird. In den Handlern findet jedoch keine Prüfung statt; jeder valide Key hat Vollzugriff.

### Lösung: Permission-Check in Handlern/Middleware
Einführung eines Guard-Systems (z.B. via Axum Middleware oder einem `require_scope!` Makro).
*Vorteil:* Erlaubt die Erstellung von Read-Only Keys für Dashboards oder Write-Only Keys für Sensoren, was das Angriffsrisiko massiv senkt.

---

## 29. Plugin-Sicherheit: Permission System (Least Privilege)

### Status Quo
Plugins können alle Host-Funktionen (`http_get`, `query`, etc.) uneingeschränkt nutzen, sobald sie geladen sind.

### Lösung: Manifest-basierte Berechtigungen
Plugins müssen im Manifest deklarieren, welche Host-Funktionen sie benötigen (z.B. `permissions: ["network", "storage"]`). Der Host verweigert den Zugriff, falls eine Funktion nicht deklariert wurde.
*Vorteil:* Verhindert, dass bösartige oder kompromittierte Plugins unbemerkt Daten nach außen senden oder auf fremde Datenbank-Bereiche zugreifen.

---

## 30. Integration: Outgoing Webhooks & Triggers

### Status Quo
Scry fungiert als Daten-Senke. Es gibt keinen Mechanismus, um externe Systeme aktiv über neue Erkenntnisse oder Events zu benachrichtigen.

### Lösung: Rule-based Webhook Engine
Ein Service, der bei bestimmten Event-Kategorien oder Korrelationen HTTP-Requests an externe URLs sendet.
*Vorteil:* Nahtlose Integration in Automatisierungs-Ökosysteme (n8n, Node-RED, Home Assistant). Scry wird vom passiven Archiv zum aktiven Steuerzentrum.

---

## 31. Knowledge Graph: Semantic Query DSL

### Status Quo
Abfragen über Entitäten und Beziehungen erfordern aktuell komplexe SQL-Joins über mehrere Tabellen (`entities`, `entity_traits`, `entity_relationships`).

### Lösung: Rust-basierte Fluent API
Implementierung eines Query-Builders für den Graphen.
*Vorteil:* Entwickler (und Plugins über die Host-API) können komplexe Fragen wie "Welche Alben habe ich diesen Monat gehört, deren Genre 'Jazz' ist?" in einer Zeile Code beantworten, ohne SQL-Fehler zu riskieren.

---

## 32. Performance: Batch Ingestion & Vector Support

### Status Quo
Jedes Event triggert einen vollständigen Ingest-Prozess (Plugins, DB-Writes, Broadcast). Bei Import-Szenarien führt dies zu massiven Overheads.

### Lösung: Bulk-API & Embedding Hooks
1. **Bulk-Ingest:** API-Endpunkt für Listen von Events.
2. **Vector DB:** Integration von `sqlite-vss` zur Speicherung von Text-Embeddings.
*Vorteil:* Drastische Beschleunigung von Daten-Imports und Vorbereitung auf Semantic Search / LLM-Integration (Phase 2 der Roadmap).

---

## 33. UI: Dynamic & Extensible Widget System

### Status Quo
Das Frontend besitzt eine feste Liste an Widget-Templates (`Metric`, `Trend`, etc.). Neue Visualisierungen erfordern Änderungen am Svelte-Core.

### Lösung: Web Components oder JSON-UI-Schemas
Erlaubt Plugins, eigene Visualisierungs-Logik zu definieren (z.B. via Vega-Lite JSON für Charts oder sogar isolierte Web Components).
*Vorteil:* Maximale Flexibilität für Plugin-Autoren, ohne das Frontend aufzublähen.

---

## 34. Observability: Prometheus & Metrics (Backend)

### Status Quo
Es gibt keine aggregierten Performancedaten. Probleme wie langsame Plugins oder DB-Bottlenecks werden nur reaktiv in Logs sichtbar.

### Lösung: Prometheus Exporter
Integration des `metrics` Crates in Axum, um einen `/metrics` Endpunkt für Prometheus bereitzustellen.
*Vorteil:* Visualisierung der Systemlast, Event-Durchsatz und Plugin-Latenzen in Dashboards (z.B. Grafana).

---

## 35. Security: Secure Session Management (Fullstack)

### Status Quo
Die Svelte-App nutzt langlebige API-Keys, die im `localStorage` gespeichert werden. Dies ist anfällig für XSS-Angriffe.

### Lösung: JWTs & HTTP-Only Cookies
Umstellung der Web-UI auf kurzlebige JWT-Access-Tokens und Refresh-Tokens in `httpOnly` Cookies.
*Vorteil:* Drastisch erhöhte Sicherheit für die Browser-Nutzung, während API-Keys weiterhin für CLI/Skripte verfügbar bleiben.

---

## 36. Validation: Advanced Integration Testing (Backend)

### Status Quo
Tests beschränken sich auf Unit-Tests in Handlern. Es fehlt eine automatisierte Prüfung des Zusammenspiels von Core, Wasm-Host und Datenbank.

### Lösung: Integration Test Suite mit Mock-Host
Einführung eines dedizierten `tests/` Verzeichnisses, das den gesamten Request-Lifecycle simuliert und externe Plugin-Abhängigkeiten (HTTP) mockt.
*Vorteil:* Garantiert die Stabilität der Plattform bei Refactorings am Plugin-Host oder der Service-Ebene.

---

## 37. Performance: Wasm Instance Pooling (Backend)

### Status Quo
In `manager.rs` wird bei jedem Event (`with_instance`) das Wasm-Modul (`crate::plugins::Plugin::instantiate_async`) komplett neu instantiiert. Dies verursacht massiven Overhead bei hohem Event-Durchsatz.

### Lösung: Instance Pool (z.B. deadpool)
Bereits initialisierte Wasm-Instanzen werden in einem Pool vorgehalten und wiederverwendet (nach Reset des Speichers).
*Vorteil:* Senkt die Latenz beim Ingest drastisch und erhöht die Skalierbarkeit des Servers.

---

## 38. Stability: SQLite Busy Timeouts (Backend)

### Status Quo
In `main.rs` wird die SQLite-Verbindung ohne `busy_timeout` konfiguriert. Bei parallelen Schreibzugriffen (Ingest + Background Analytics) kann dies sofort zu "database is locked" Fehlern führen.

### Lösung: Konfiguration des Connection Pools
Setzen von `.busy_timeout(std::time::Duration::from_secs(5))` in den `SqliteConnectOptions`.
*Vorteil:* Erhöht die Robustheit. Threads warten kurz auf Freigabe der Datenbank, statt sofort mit einem Fehler abzubrechen.

---

## 39. Security: Secure Auth Persistence (Frontend)

### Status Quo
In `auth.svelte.ts` wird der sensible `apiKey` im `localStorage` gespeichert. Dies ist ein potenzielles Ziel für Cross-Site Scripting (XSS) Angriffe.

### Lösung: HTTP-Only Cookies für Auth-Tokens
Nutzung des `localStorage` nur für nicht-sensible UI-Zustände. Der API-Key (oder JWT) wird ausschließlich in einem `httpOnly` Cookie gespeichert, auf den JavaScript keinen Zugriff hat.
*Vorteil:* Schützt den wichtigsten Schlüssel des Nutzers vor Diebstahl durch fehlerhafte oder bösartige Front-End-Scripts.

---

## 40. Reliability: Strict Manifest Validation (Backend)

### Status Quo
In `manager.rs` wird das von einem Plugin zurückgegebene Manifest beim Laden blind akzeptiert.

### Lösung: Host-seitige Validierung
Validierung der `id`, `name` und deklarierten `exports` gegen ein striktes Schema (z.B. Prüfung auf gültige semantische Pfade), bevor das Plugin dem System hinzugefügt wird.
*Vorteil:* Verhindert inkonsistente Systemzustände durch fehlerhaft programmierte oder bösartige Plugins.

---

## 41. Privacy: Encryption at Rest (SQLCipher)

### Status Quo
Die `scry.db` liegt als unverschlüsselte SQLite-Datei auf der Festplatte. Jeder mit Zugriff auf das Dateisystem kann alle persönlichen Daten und Korrelationen einsehen.

### Lösung: SQLCipher Integration
Umstellung auf SQLCipher (via `sqlx` Features). Der Datenbank-Schlüssel wird beim Start (z.B. über eine Environment-Variable oder interaktive Eingabe) bereitgestellt.
*Vorteil:* Schützt sensible "Life Logging" Daten vor Diebstahl oder unbefugtem Zugriff bei physischer Kompromittierung des Hosts.

---

## 42. Connectivity: CRDT-based Synchronization

### Status Quo
Aktuell ist Scry eine Standalone-Instanz. Ein Abgleich zwischen mehreren Geräten (z.B. Desktop und Server) ist nicht vorgesehen.

### Lösung: Conflict-free Replicated Data Types (CRDTs)
Implementierung einer Sync-Logik für den Knowledge Graph und Plugin-Zustände basierend auf CRDTs (z.B. via `automerge` oder `yjs`-ähnlichen Ansätzen in Rust).
*Vorteil:* Nahtlose Synchronisation zwischen mehreren Instanzen; Änderungen können offline vorgenommen werden und verschmelzen automatisch ohne Datenverlust.

---

## 43. Scalability: Semantic Materialized Views

### Status Quo
Dashboards und Statistiken werden bei jedem Aufruf durch teure SQL-Aggregationen über die gesamte `events` Tabelle berechnet. Dies skaliert nicht bei Millionen von Einträgen.

### Lösung: Inkrementelle Aggregation
Einführung von Materialisierten Views für häufig genutzte semantische Metriken. Ein Hintergrundprozess aktualisiert diese Statistiken inkrementell bei jedem Ingest.
*Vorteil:* Dashboards laden in Millisekunden, unabhängig von der Gesamtzahl der gespeicherten Events.

---

## Zusammenfassung der Prioritäten (Master Roadmap)

1.  **Critical:** Wasmtime-Sandboxing (Sicherheit).
2.  **Critical:** Multi-Tenant Rate Limiting & Scope Enforcement (Sicherheit).
3.  **Critical:** Secure Auth Persistence (Sicherheit).
4.  **High:** Aufteilung der `handlers.rs` (Wartbarkeit).
5.  **High:** Einführung von `ts-rs` für Typsicherheit (DRY).
6.  **High:** SDK-to-Host Logic Migration (Abstraktion).
7.  **Medium:** Wasm Instance Pooling (Performance).
8.  **Medium:** SQLite Busy Timeouts & SQLCipher (Stabilität/Privacy).
9.  **Medium:** Semantic Materialized Views (Performance).
10. **Medium:** Plugin Permissions (Sicherheit).
11. **Medium:** Globales Error-Handling im API-Wrapper (DX/UX).
12. **Medium:** Multi-Stage Docker Deployment (UX/Hosting).
13. **Low:** CRDT Sync & Webhooks Engine.
14. **Low:** Semantic Graph DSL & Discovery Orchestrator.
15. **Low:** E2E Testing Suite & i18n Readiness.
16. **Low:** RFC 7807, Documentation, Linting & Design Tokens.
