# Korelator

> *"An event unexamined is a threat undetected. The collective examines everything."*
> — Komrad Engineering Collective, 2026

Korelator is the correlation engine of the Komrad ecosystem. It consumes a stream of JSON events, evaluates each one against a set of declarative detection rules, and persists alerts to PostgreSQL when a rule is satisfied. It consumes [Kompiler](https://github.com/komrad-company/Kompiler) for rule parsing, [Khronika](https://github.com/komrad-company/Khronika) for logging, and [Konnect](https://github.com/komrad-company/Konnect) for database access. Nothing else enters. Nothing else leaves without scrutiny.

```
JSON events (stdin | Quickwit)
    └──► Korelator (evaluation engine)
              ├── Kompiler   (rule parsing — Vec<Rule>)
              ├── Khronika   (logging — every decision is traced)
              ├── Konnect    (PostgreSQL — alert persistence)
              └──► AlertSink (stderr JSON — pluggable trait)
```

The binary reads one JSON event per line from the configured datasource, evaluates every rule against every event, and emits an [`Alert`](#public-types) for each match through the configured [`AlertSink`](#public-types). Malformed JSON lines are skipped with a warning; an empty line is ignored.

---

## Configuration

Korelator reads its configuration from a JSON file. The path is resolved from the `CONFIGURATION_PATH` environment variable, falling back to `configuration.json` in the working directory.

**stdin** — reads JSON events line by line from standard input:

```json
{
    "datasource": "stdin",
    "rules_path": "/etc/korelator/rules",
    "log": {
        "level": "error",
        "file": "output/korelator.log"
    },
    "database": {
        "host": "localhost",
        "port": 5432,
        "database": "komrad",
        "user": "korelator",
        "password": "...",
        "schema": "korelator",
        "search_path": "korelator"
    }
}
```

**Quickwit** — polls a Quickwit index, paginating with `search_after`:

```json
{
    "datasource": {
        "quickwit": {
            "url": "http://quickwit.internal:7280",
            "index": "logs"
        }
    },
    "rules_path": "/etc/korelator/rules",
    "log": {
        "level": "error",
        "file": "output/korelator.log"
    },
    "database": {
        "host": "localhost",
        "port": 5432,
        "database": "komrad",
        "user": "korelator",
        "password": "...",
        "schema": "korelator",
        "search_path": "korelator"
    }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `datasource` | `"stdin"` \| `{ "quickwit": { "url", "index" } }` | ✅ | Event source |
| `rules_path` | `string` | ✅ | Path to rule files, passed directly to `kompiler::parse_rules` |
| `log` | `TelemetryConfiguration` | ✅ | Khronika logger configuration |
| `database` | `DatabaseConfig` | ✅ | PostgreSQL connection — see [Konnect](https://github.com/komrad-company/Konnect) |

A missing or malformed configuration file is fatal. The binary exits with code `1`. No partial start is tolerated.

A ready-to-use template is available at [`examples/configuration_template.json`](examples/configuration_template.json).

---

## Running locally

**1. Copy and fill in the template:**

```sh
cp examples/configuration_template.json configuration.json
# edit configuration.json — set database credentials and rules_path
```

**2. Feed events via stdin:**

```sh
echo '{"process_name": "bash_shell", "username": "deploy"}' | cargo run
```

The binary resolves its config from the `CONFIGURATION_PATH` environment variable, falling back to `configuration.json` in the working directory. No CLI arguments are accepted.

```sh
# override config path
CONFIGURATION_PATH=/etc/korelator/config.json cargo run
```

---

## API

### Public types

| Type | Role |
|---|---|
| `Configuration` | Deserialised runtime configuration |
| `DatasourceType` | Enum — `Stdin` or `Quickwit { url, index }`. Drives source selection at startup. |
| `StdinSource` | Reads JSON events line by line from standard input. |
| `QuickwitSource` | Polls a Quickwit index, paginating with `search_after`. Sleeps 1 s between polls when the index returns no hits. |
| `PreparedRule` | A parsed [`kompiler::Rule`] with its evaluation context built once at load time. Built via `From<Rule>`. Call `fires_on(&event)` to evaluate the rule, `to_alert(event)` to materialise an `Alert`. |
| `Alert` | One detection record: `rule_id`, `title`, `level`, `event`, `timestamp_unix`. Serialises to JSON. Defined in Kodeks. |
| `AlertSink` | Trait — `fn emit(&self, &Alert)`. `Send + Sync`, ready to share across threads. Must be imported explicitly to call `emit`. |
| `StderrJsonSink` | Default sink — writes one JSON alert per line on stderr. `emit_to(&alert, writer)` accepts any `Write` for testing. |
| `AlertStore` | PostgreSQL store for alerts — implements `konnect::Store`. `AlertStore::setup(&config)` opens the pool and runs migrations. `spawn_persist_task()` spawns a background Tokio task and returns an `UnboundedSender<Alert>` — send an alert to persist it asynchronously. |
| `load_rules` | Loads and compiles rules from `rules_path` into a `Vec<PreparedRule>`. Fatal on parse error (exit code 2). |
| `run_datasource` | Drives the event loop: pulls events from the configured source and routes them through the evaluation engine. |

### Matcher support

| Matcher | Status |
|---|---|
| `Single` | ✅ Implemented — fires on every event satisfying the condition |
| `Threshold` | ⏳ Not yet implemented — rules using it are loaded but never fire; a warning is logged on each event |

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Normal exit |
| `1` | Fatal error — configuration, database connection, or migration failure |
| `2` | Fatal error parsing rules |
| `3` | Fatal datasource stream error |

---

## Dependencies

Each dependency was evaluated by the collective before admission. None were added lightly.

| Crate | Source | Purpose |
|---|---|---|
| `kompiler` | komrad-company, git tag | Rule parsing and typed representation |
| `khronika` | komrad-company, git tag | Structured logging and telemetry |
| `konnect` | komrad-company, git tag | PostgreSQL connection pool and store trait |
| `sqlx` | crates.io | SQL query execution and migrations |
| `serde` + `serde_json` | crates.io | Configuration deserialisation, event handling |
| `reqwest` | crates.io | HTTP client for Quickwit datasource |
| `tokio` | crates.io | Async runtime |

---

## License

AGPL-3.0-or-later — the source remains open, as all things should be.