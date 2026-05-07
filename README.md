# Korelator

> Event correlation engine, rule triggering and alert dispatcher — written in Rust.

Korelator is the central component of an event processing pipeline (typically logs or security events). It consumes a stream of events (as JSON), evaluates them against a set of declarative rules, and triggers alerts when a rule is satisfied.

It is part of the **komrad-company** ecosystem and relies on two internal crates:

- [`Kompiler`](https://github.com/komrad-company/Kompiler) — parsing, compilation and typed representation of correlation rules, as well as error types (`UnforgivableErrors`).
- [`Khronika`](https://github.com/komrad-company/Khronika) — logging / telemetry system (local logs + remote endpoint).

---

## Table of contents

1. [Purpose](#purpose)
2. [General architecture](#general-architecture)
3. [Project structure](#project-structure)
4. [Configuration](#configuration)
5. [Evaluation engine in depth](#evaluation-engine-in-depth)
6. [Event format](#event-format)
7. [Build and run](#build-and-run)
8. [Tests](#tests)
9. [Security and dependency audit](#security-and-dependency-audit)
10. [Continuous integration](#continuous-integration)
11. [Roadmap / current status](#roadmap--current-status)
12. [License](#license)

---

## Purpose

The core need is simple: given a stream of events (e.g. logs stored in **Quickwit**), be able to say:

> "If an event looks like *this*, then trigger *that*."

Korelator is the tool that:

1. **Loads a configuration** describing where to find rules, where to fetch events from, and how to log.
2. **Compiles rules** (delegated to `Kompiler`) into usable Rust structures (`FieldFilter`, `Filters`, etc.).
3. **Evaluates each event** against those rules via its evaluation engine, built on the `Evaluate` trait.
4. **Triggers actions** (alerts, triggers) when a rule matches — this part is being implemented.

Concretely, it transforms a raw volume of events (often unmanageable manually) into actionable signals for a SOC, an observability tool, or any detection pipeline.

### Typical use cases

- **Intrusion detection**: spotting a suspicious sequence in system logs (`process_name = bash` + `parent = sshd` + `account startswith "adm"`).
- **Application monitoring**: alerting when an endpoint returns too many 5xx errors (`status_code >= 500` repeated N times).
- **Compliance / audit**: tracing every sensitive operation (`action = "delete_user"` on a privileged account).
- **Multi-source correlation**: cross-referencing events from multiple systems to reconstruct more complex behaviour.

---

## General architecture

```
                     ┌────────────────────────────┐
                     │       Rule files           │
                     │   (parsed by Kompiler)     │
                     └──────────────┬─────────────┘
                                    │
                                    ▼
┌──────────────┐    events    ┌──────────────────────┐   alerts
│   Quickwit   │ ───────────▶ │      Korelator       │ ───────────▶
│  (logs/data) │     JSON     │  (evaluation engine) │   (sink)
└──────────────┘              └──────────┬───────────┘
                                         │
                                         ▼
                                  ┌────────────┐
                                  │  Khronika  │
                                  │  (logging) │
                                  └─────┬──────┘
                                        │
                                        ▼
                              local file + remote endpoint
```

### Internal pipeline

Inside Korelator, an event goes through the following steps:

1. **Ingestion**: a JSON event arrives (from Quickwit or another source).
2. **Deserialisation**: it is represented as `serde_json::Value`.
3. **Evaluation**: the event is passed to each compiled rule. Each rule is composed of filters (`FieldFilter`) that implement the `Evaluate` trait.
4. **Decision**: if a rule is satisfied, the associated action is triggered.
5. **Logging**: Khronika traces what happened (configurable level).

---

## Project structure

```
Korelator/
├── Cargo.toml                          # crate manifest
├── deny.toml                           # cargo-deny policy (licences, sources, advisories)
├── LICENSE                             # AGPL-3.0-or-later
├── README.md                           # this file
├── .github/workflows/ci.yml            # CI using shared Kontinuous-integration workflows
├── examples/
│   └── configuration_template.json     # example configuration
└── src/
    ├── lib.rs                          # crate public API (load_configuration)
    ├── main.rs                         # binary entrypoint
    ├── configuration.rs                # Configuration struct (deserialised from JSON)
    ├── evaluation_engine.rs            # Evaluate trait + EvaluationContext
    └── evaluation_engine/
        └── filter.rs                   # impl Evaluate for FieldFilter + tests
```

### Crate modules

| Module | File | Role |
|---|---|---|
| `configuration` | `src/configuration.rs` | Defines the `Configuration` struct deserialised from the JSON file. |
| `evaluation_engine` | `src/evaluation_engine.rs` | Defines the `Evaluate` trait and the `EvaluationContext` carrying shared filters (`Arc<HashMap<String, Filters>>`). |
| `evaluation_engine::filter` | `src/evaluation_engine/filter.rs` | Implements `Evaluate` for `FieldFilter` — the field ↔ expected values comparison logic. |
| `lib.rs` | `src/lib.rs` | Exposes `load_configuration()`, the entry point for loading config from disk. |
| `main.rs` | `src/main.rs` | Binary: loads config, initialises logger, parses rules. |

### Dependencies

| Crate | Version / Source | Role |
|---|---|---|
| `serde` | `1` (features = `derive`) | Serialisation/deserialisation. |
| `serde_json` | `1` | Manipulation of events and JSON configuration. |
| `khronika` | git (komrad-company), tag `v1.0.2` | Logger / telemetry. |
| `kompiler` | git (komrad-company) | Rule parsing and types. |

Rust edition used: `2024` (recent toolchain required).

---

## Configuration

The configuration is loaded from a JSON file by `load_configuration()` exposed in `lib.rs`. The path is read from the `CONFIGURATION_PATH` environment variable, with `configuration.json` (in the current directory) as fallback.

### Loading algorithm

```rust
pub fn load_configuration() -> Result<Configuration, UnforgivableErrors> {
    let configuration_path: String = env::var("CONFIGURATION_PATH")
        .unwrap_or_else(|_| "configuration.json".to_string());

    let file = File::open(&configuration_path)
        .map_err(|_| UnforgivableErrors::MissingConfigurationFile { path: configuration_path })?;

    let reader = BufReader::new(file);
    let conf = from_reader(reader).map_err(UnforgivableErrors::InvalidFormat)?;

    Ok(conf)
}
```

Two fatal error cases:

| Error | Cause | Action |
|---|---|---|
| `MissingConfigurationFile { path }` | File does not exist or is not accessible. | Binary prints `Fatal Error: ...` on `stderr` and exits with code `1`. |
| `InvalidFormat(...)` | JSON is malformed or a required field is missing. | Same: `exit(1)`. |

### Format

Minimal example (more complete than `examples/configuration_template.json`):

```json
{
    "quickwit_url": "http://quickwit.internal:7280",
    "rules_path": "/etc/korelator/rules",
    "log": {
        "level": "error",
        "file": "output/korelator.log",
        "remote": "https://telemetry.korelator.org"
    }
}
```

### Fields

| Field | Type | Required | Description |
|---|---|---|---|
| `quickwit_url` | `string` | ✅ | URL of the Quickwit instance used as event source. |
| `rules_path` | `string` | ✅ | Path (file or directory) where rules are found. Passed directly to `kompiler::rules::parse_rules`. |
| `log` | `TelemetryConfiguration` (Khronika) | ✅ | Logger configuration. |
| `log.level` | `string` | ✅ | Minimum level (`error`, `warn`, `info`, `debug`, `trace`). |
| `log.file` | `string` | ✅ | Path of the local log file. |
| `log.remote` | `string` | ✅ | Remote endpoint for telemetry forwarding. |

> ⚠️ **Note on the example template**: `examples/configuration_template.json` is missing the `quickwit_url` field which is required by the struct. Deserialisation will fail until it is added. Either complete the template or make the field optional (`Option<String>`) in `configuration.rs`.

> The `configuration.json` file and the `output/` directory are git-ignored, preventing local config or logs from being committed.

---

## Evaluation engine in depth

### The `Evaluate` trait

This is the central contract:

```rust
pub trait Evaluate {
    fn evaluate(&self, event: &Value, ctx: &EvaluationContext) -> bool;
}
```

Any structure that can be matched against an event implements this trait. Currently the implementation exists for `FieldFilter`, but the design allows extension to filter compositions, full rules, etc.

### `EvaluationContext`

```rust
pub struct EvaluationContext {
    pub filters: Arc<HashMap<String, Filters>>,
}
```

The evaluation context carries a map of named filters, shareable across threads via `Arc`. This allows a filter to reference another by name (e.g. a composite filter reusing sub-filters declared elsewhere).

### Evaluating a `FieldFilter`

A `FieldFilter` (defined in `Kompiler`) has three elements:

```rust
FieldFilter {
    field: String,            // JSON field name to inspect
    condition: FilterTypes,   // comparison operator
    values: Vec<Types>,       // list of expected values
}
```

The algorithm:

1. Read `event[field]`. If absent → `false`.
2. For each expected value in `values`, test if `(condition, field_value, expected_value)` matches.
3. Return `true` as soon as one value matches (**implicit OR** over `values`).

### Supported conditions

| Condition | Operand type | Semantics |
|---|---|---|
| `Contains` | String | Field contains the substring |
| `Startswith` | String | Field starts with the string |
| `Endswith` | String | Field ends with the string |
| `Exact` | String | String equality |
| `Exact` | Integer | Integer equality (i64) |
| `Gt` | Integer | Strictly greater than |
| `Gte` | Integer | Greater than or equal |
| `Lt` | Integer | Strictly less than |
| `Lte` | Integer | Less than or equal |

To AND conditions, compose multiple filters at the rule level (logic handled by Kompiler-compiled rules).

### Edge cases and guarantees

- **Absent field** in the event → `false` (silent, no error).
- **Incompatible type** (e.g. numeric condition on a string field, or vice versa) → `false`.
- **No expected values** (`values` empty) → `false` (`.any()` on an empty iterator returns `false`).
- Evaluation is **pure**: no side effects, no hidden allocations. Safe to call in a tight loop.
- `EvaluationContext` is shared via `Arc`, so evaluation is safe to parallelise.

---

## Build and run

### Prerequisites

- Git access to `komrad-company/Khronika` and `komrad-company/Kompiler` repositories.

### Build

```bash
# Debug build (fast, unoptimised)
cargo build

# Release build (optimised)
cargo build --release
```

The produced binary is at `target/release/korelator` (or `target/debug/korelator`).

### Run

```bash
# Using the default path (./configuration.json)
cargo run
```

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Normal exit. |
| `1` | Fatal error loading configuration (file missing, invalid JSON). |
| `2` | Fatal error parsing rules (`UnforgivableErrors` raised by Kompiler). |

### Current binary behaviour (`main.rs`)

Step by step:

1. **`load_configuration()`**: reads and deserialises the config file.
   - On error → `eprintln!` + `exit(1)`.
2. **`intialize_logger(configuration.log)`**: initialises Khronika with the telemetry config.
3. **`debug!("Korelator successfully initiated")`**: startup trace.
4. **`parse_rules(rules_path)`**: Kompiler loads and compiles rules from `rules_path`.
   - On error → `error!` + `exit(2)`.
5. **`dbg!(parsed_rules.len())`**: currently just prints the number of parsed rules.

> The event ingestion loop and actual alert triggering are **not yet wired** in `main.rs`. The foundation is in place (config + rules + evaluation engine); connecting the Quickwit source and the alert sink is still to come.

---

## Tests

Current unit tests cover filter evaluations in `src/evaluation_engine/filter.rs`:

| Test | What it verifies |
|---|---|
| `contains_matches_substring` | `Contains` matches on substring, and does not match on absent value. |
| `contains_multiple_values_is_or` | Multiple values → implicit OR. |
| `startswith_matches_prefix` | `Startswith` matches a prefix and not a suffix. |
| `exact_integer_matches` | `Exact` on integer. |
| `gt_integer_matches` | `Gt` is strict, so `5` does not match for `> 5`. |
| `missing_field_returns_false` | An absent field returns `false` without panicking. |

Run the suite:

```bash
cargo test
```

---

## Security and dependency audit

The project uses [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) to audit dependencies. The policy is in `deny.toml`.

```bash
cargo deny check
```

### Policy in place

- **Target**: `x86_64-unknown-linux-gnu`.
- **Security advisories** (`[advisories]`): version 2, `yanked = "deny"` — any yanked crate is rejected.
- **Allowed licences** (`[licenses]`, confidence ≥ 0.90): MIT, Apache-2.0, BSD-3-Clause, ISC, 0BSD, Zlib, AGPL-3.0, AGPL-3.0-or-later, Unicode-3.0.
- **Bans**:
  - Multiple versions → `warn`.
  - Version wildcards → `warn`.
- **Sources**:
  - Unknown registries → `deny`.
  - Unknown Git repositories → `deny`.
  - Allowed only: `https://github.com/rust-lang/crates.io-index`, `Khronika.git`, `Kompiler.git`.

This ensures no unidentified external dependency can enter the project without an explicit change to `deny.toml`.

---

## Continuous integration

The CI lives in `.github/workflows/ci.yml` and delegates to shared workflows from the **Kontinuous-integration** repository:

```yaml
on:
  push:
    branches: [main]
    tags: ['v*']
  pull_request:

jobs:
  security:
    uses: komrad-company/Kontinuous-integration/.github/workflows/security-pipeline.yml@main
  pipeline:
    permissions:
      contents: write
    needs: security
    uses: komrad-company/Kontinuous-integration/.github/workflows/rust-pipeline.yml@main
```

- The **`security`** job runs the security pipeline (gitleaks secret scanning, `cargo deny` audit).
- The **`pipeline`** job runs the standard Rust pipeline (build, tests, lint, releases on `v*` tags) — it depends on the security job succeeding.

---

## Roadmap / current status

Done:
- ✅ JSON configuration loading + env var override.
- ✅ Khronika logger initialisation.
- ✅ Rule parsing via Kompiler.
- ✅ `Evaluate` trait + impl for `FieldFilter`.
- ✅ Unit tests on filters.
- ✅ Shared CI + `cargo-deny` policy.

Pending:
- ⏳ Composite rule evaluation (beyond `FieldFilter` alone).
- ⚠️ `quickwit_url` field required by config struct but missing from the example template.
- ⏳ Quickwit connection for event ingestion.
- ⏳ Alert dispatch / trigger sink module.
- ⏳ Event loop in `main.rs` (currently just counts parsed rules).

---

## License

Distributed under **AGPL-3.0-or-later**. See [`LICENSE`](LICENSE) for the full text.
