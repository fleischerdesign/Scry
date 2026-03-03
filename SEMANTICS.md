# Scry Semantics: Roadmap to "Level 3"
**Goal:** Transition from a "Namespace/ID" system to a fully Typed and Linked Data Graph.

## 1. Status Quo (Level 2)
Wir nutzen aktuell Namespaces (`scry.core/name`) und Hierarchien (`environment.temperature`). Das ist stabil, aber es fehlt die **Explizität** (Was ist eine Zahl? Was ist ein Ding?).

---

## 2. Verbesserungsvorschlag: Explizite Typ-Präfixe (Type Hinting)
Aktuell wissen wir bei `music.artist` nicht, ob es ein Name (String) oder eine Entität (ID) ist.

### Vorschlag:
Wir führen obligatorische Präfixe für die **Semantic Types** ein:
*   `metric.*`: Reine Messwerte (Zahlen).
    *   *Beispiel:* `metric.environment.temperature`, `metric.system.cpu`
*   `entity.*`: Verweise auf Objekte im Knowledge Graph.
    *   *Beispiel:* `entity.music.artist`, `entity.core.user`
*   `state.*`: Binäre oder diskrete Zustände.
    *   *Beispiel:* `state.light.on`, `state.plugin.active`

**Vorteil:** Das Frontend kann automatisch entscheiden: `metric` -> Graph/Stat, `entity` -> Link/Avatar, `state` -> Toggle/Badge.

---

## 3. Verbesserungsvorschlag: Schema.org Bridge (Interoperabilität)
Unsere internen Traits (`scry.core/name`) sind sicher, aber isoliert.

### Vorschlag:
Wir führen eine **Mapping-Schicht** im Backend ein, die interne Traits auf den weltweiten Standard **Schema.org** mappt:
*   `scry.core/name` -> `https://schema.org/name`
*   `scry.visual/photo` -> `https://schema.org/image`
*   `scry.core/city` -> `https://schema.org/addressLocality`

**Vorteil:** Wenn wir später eine KI (wie Gemini/GPT) oder externe APIs anbinden, verstehen diese sofort die Bedeutung unserer Daten, ohne dass wir Scry-spezifischen Code schreiben müssen.

---

## 4. Verbesserungsvorschlag: Semantische Einheiten (Unit Standardization)
Aktuell steht die Einheit `"°C"` hart im Widget-Code oder in der Plugin-Konfiguration.

### Vorschlag:
Wir verschieben die Einheiten in die **DataField-Definition** im Manifest:
```json
{
  "semantic_type": "metric.environment.temperature",
  "unit": "celsius",
  "base_unit": "kelvin"
}
```
**Vorteil:** Das System kann automatisch zwischen Celsius und Fahrenheit umrechnen, basierend auf den User-Einstellungen, ohne dass das Plugin davon wissen muss.

---

## 5. Advanced Semantics: The "Missing Depth"

### 5.1 Privacy & Sensitivity (PII Tagging)
Traits can be sensitive. We should allow marking traits as PII (Personally Identifiable Information).
*   **Goal:** `scry.core/name` is PII, while `environment.temperature` is not.
*   **UI Impact:** The frontend can automatically blur or truncate PII fields in "Public Mode" or "Demo Mode" without per-widget logic.

### 5.2 Confidence & Data Quality
Data from sensors or AI enrichers is not always 100% accurate.
*   **Goal:** Add a standard `confidence` field (0.0 - 1.0) to enriched event data.
*   **UI Impact:** Widgets can render low-confidence data with a "warning" icon or decreased opacity to avoid misleading the user.

### 5.3 Temporal Semantics (Absolute vs. Delta)
Differentiating between a "Current State" and an "Incremental Change".
*   **Metric Types:**
    *   `metric.absolute.*`: A snapshot (e.g., current temperature, current bank balance).
    *   `metric.delta.*`: A change over time (e.g., steps taken, revenue in the last hour).
*   **UI Impact:** Line charts for absolute values, Bar charts for deltas.

---

## 6. Zusammenfassung der nächsten Schritte
1.  Refactoring der Plugin-Manifeste auf die neuen Präfixe (`metric.`, `entity.`).
2.  Hinzufügen eines `unit`-Feldes zum `DataField` im `scry-plugin-sdk`.
3.  Implementierung eines `SemanticResolver` im Backend, der die Brücke zu `Schema.org` schlägt.
4.  Einführung von Privacy-Flags für Core-Traits.
5.  Standardisierung auf URIs (`scry://[namespace]/[type]/[id]`) für Entitäten.
