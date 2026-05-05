# Korelator

> Moteur de corrélation d'évènements, de déclenchement de règles et d'envoi d'alertes — écrit en Rust.

Korelator est le composant central d'une chaîne de traitement d'évènements (typiquement des logs ou évènements de sécurité). Il consomme un flux d'évènements (sous forme de JSON), les confronte à un ensemble de règles déclaratives, et déclenche des alertes lorsqu'une règle est satisfaite.

Il fait partie de l'écosystème **komrad-company** et s'appuie sur deux briques internes :

- [`Kompiler`](https://github.com/komrad-company/Kompiler) — parsing, compilation et représentation typée des règles de corrélation, ainsi que les types d'erreurs (`UnforgivableErrors`).
- [`Khronika`](https://github.com/komrad-company/Khronika) — système de logging / télémétrie (logs locaux + endpoint distant).

---

## Table des matières

1. [À quoi ça sert](#à-quoi-ça-sert)
2. [Architecture générale](#architecture-générale)
3. [Structure du projet](#structure-du-projet)
4. [Configuration](#configuration)
5. [Le moteur d'évaluation en détail](#le-moteur-dévaluation-en-détail)
6. [Format des évènements](#format-des-évènements)
7. [Compilation et exécution](#compilation-et-exécution)
8. [Tests](#tests)
9. [Sécurité et audit des dépendances](#sécurité-et-audit-des-dépendances)
10. [Intégration continue](#intégration-continue)
11. [Roadmap / état actuel](#roadmap--état-actuel)
12. [Licence](#licence)

---

## À quoi ça sert

Le besoin de base est simple : à partir d'un flux d'évènements (par exemple des logs stockés dans **Quickwit**), on veut pouvoir dire :

> « Si un évènement ressemble à *ceci*, alors déclenche *cela*. »

Korelator est l'outil qui :

1. **Charge une configuration** décrivant où trouver les règles, où aller chercher les évènements, et comment se logger.
2. **Compile les règles** (déléguées à `Kompiler`) en structures Rust exploitables (`FieldFilter`, `Filters`, etc.).
3. **Évalue chaque évènement** contre ces règles via son moteur d'évaluation, basé sur le trait `Evaluate`.
4. **Déclenche des actions** (alertes, triggers) quand une règle matche — partie en cours de mise en place.

Concrètement, c'est la pièce qui transforme un volume brut d'évènements (souvent ingouvernable manuellement) en signaux exploitables pour un SOC, un outil d'observabilité, ou n'importe quel pipeline de détection.

### Cas d'usage typiques

- **Détection d'intrusion** : repérer dans les logs système une suite suspecte (`process_name = bash` + `parent = sshd` + `account startswith "adm"`).
- **Surveillance applicative** : alerter quand un endpoint renvoie trop d'erreurs 5xx (`status_code >= 500` répété N fois).
- **Conformité / audit** : tracer toute opération sensible (`action = "delete_user"` sur un compte privilégié).
- **Corrélation multi-source** : croiser des évènements provenant de plusieurs systèmes pour reconstruire un comportement plus complexe.

---

## Architecture générale

```
                     ┌────────────────────────────┐
                     │   Fichiers de règles       │
                     │   (parsés par Kompiler)    │
                     └──────────────┬─────────────┘
                                    │
                                    ▼
┌──────────────┐  évènements  ┌──────────────────────┐   alertes
│   Quickwit   │ ───────────▶ │      Korelator       │ ───────────▶
│  (logs/data) │     JSON     │  (moteur d'évaluation)│   (sink)
└──────────────┘              └──────────┬───────────┘
                                         │
                                         ▼
                                  ┌────────────┐
                                  │  Khronika  │
                                  │  (logging) │
                                  └─────┬──────┘
                                        │
                                        ▼
                              fichier local + endpoint
                                    distant
```

### Pipeline interne

À l'intérieur de Korelator, un évènement traverse les étapes suivantes :

1. **Ingestion** : un évènement JSON arrive (depuis Quickwit ou une autre source).
2. **Désérialisation** : il est représenté en `serde_json::Value`.
3. **Évaluation** : on passe l'évènement à chaque règle compilée. Chaque règle est composée de filtres (`FieldFilter`) qui implémentent le trait `Evaluate`.
4. **Décision** : si une règle est satisfaite, l'action associée est déclenchée.
5. **Logging** : Khronika trace ce qui s'est passé (niveau configurable).

---

## Structure du projet

```
Korelator/
├── Cargo.toml                          # manifeste du crate
├── deny.toml                           # politique cargo-deny (licences, sources, advisories)
├── LICENSE                             # AGPL-3.0-or-later
├── README.md                           # ce fichier
├── .github/workflows/ci.yml            # CI mutualisée (Kontinuous-integration)
├── examples/
│   └── configuration_template.json     # exemple de configuration
└── src/
    ├── lib.rs                          # API publique du crate (load_configuration)
    ├── main.rs                         # binaire : entrypoint
    ├── configuration.rs                # struct Configuration (désérialisée depuis JSON)
    ├── evaluation_engine.rs            # trait Evaluate + EvaluationContext
    └── evaluation_engine/
        └── filter.rs                   # impl Evaluate pour FieldFilter + tests
```

### Modules du crate

| Module | Fichier | Rôle |
|---|---|---|
| `configuration` | `src/configuration.rs` | Définit la struct `Configuration` désérialisée depuis le fichier JSON. |
| `evaluation_engine` | `src/evaluation_engine.rs` | Définit le trait `Evaluate` et le `EvaluationContext` qui porte les filtres partagés (`Arc<HashMap<String, Filters>>`). |
| `evaluation_engine::filter` | `src/evaluation_engine/filter.rs` | Implémente `Evaluate` pour `FieldFilter` — la logique de comparaison champ ↔ valeurs attendues. |
| `lib.rs` | `src/lib.rs` | Expose `load_configuration()`, le point d'entrée pour charger la conf depuis disque. |
| `main.rs` | `src/main.rs` | Binaire : charge la conf, initialise le logger, parse les règles. |

### Dépendances

| Crate | Version / Source | Rôle |
|---|---|---|
| `serde` | `1` (features = `derive`) | Sérialisation/désérialisation. |
| `serde_json` | `1` | Manipulation des évènements et de la configuration JSON. |
| `khronika` | git (komrad-company), tag `v1.0.2` | Logger / télémétrie. |
| `kompiler` | git (komrad-company) | Parsing et types des règles. |

L'edition Rust utilisée est `2024` (toolchain récente requise).

---

## Configuration

La configuration est chargée depuis un fichier JSON par la fonction `load_configuration()` exposée dans `lib.rs`. Le chemin est lu depuis la variable d'environnement `CONFIGURATION_PATH`, avec `configuration.json` (dans le répertoire courant) comme fallback.

### Algorithme de chargement

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

Deux cas d'erreur fatals :

| Erreur | Cause | Action |
|---|---|---|
| `MissingConfigurationFile { path }` | Le fichier n'existe pas / pas accessible. | Le binaire affiche `Fatal Error: ...` sur `stderr` et sort avec le code `1`. |
| `InvalidFormat(...)` | Le JSON est mal formé ou un champ requis est manquant. | Idem : `exit(1)`. |

### Format

Exemple minimal (et plus complet que `examples/configuration_template.json`) :

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

### Champs

| Champ | Type | Obligatoire | Description |
|---|---|---|---|
| `quickwit_url` | `string` | ✅ | URL de l'instance Quickwit qui sert de source d'évènements. |
| `rules_path` | `string` | ✅ | Chemin (fichier ou dossier) où trouver les règles à parser. Passé directement à `kompiler::rules::parse_rules`. |
| `log` | `TelemetryConfiguration` (Khronika) | ✅ | Configuration du logger. |
| `log.level` | `string` | ✅ | Niveau minimum (`error`, `warn`, `info`, `debug`, `trace`). |
| `log.file` | `string` | ✅ | Chemin du fichier de log local. |
| `log.remote` | `string` | ✅ | Endpoint distant pour l'envoi de la télémétrie. |

> ⚠️ **Note sur le template d'exemple** : le fichier `examples/configuration_template.json` ne contient pas encore le champ `quickwit_url` qui est pourtant requis par la struct. La désérialisation échouera tant qu'il n'est pas ajouté. À corriger soit en complétant le template, soit en rendant le champ optionnel (`Option<String>`) dans `configuration.rs`.

> 🔒 **Note sur le `.gitignore`** : `configuration.json` et `output/` sont ignorés par Git, ce qui évite de committer une conf locale ou des logs.

---

## Le moteur d'évaluation en détail

### Le trait `Evaluate`

C'est le contrat central :

```rust
pub trait Evaluate {
    fn evaluate(&self, event: &Value, ctx: &EvaluationContext) -> bool;
}
```

Toute structure capable d'être confrontée à un évènement implémente ce trait. À ce jour, l'implémentation existe pour `FieldFilter`, mais le design permet de l'étendre à des compositions de filtres, des règles entières, etc.

### `EvaluationContext`

```rust
pub struct EvaluationContext {
    pub filters: Arc<HashMap<String, Filters>>,
}
```

Le contexte d'évaluation transporte une map de filtres nommés, partageable entre threads via `Arc`. Cela permet à un filtre d'en référencer un autre par nom (par exemple un filtre composite qui réutilise des sous-filtres déclarés ailleurs).

### Évaluation d'un `FieldFilter`

Un `FieldFilter` (défini dans `Kompiler`) a trois éléments :

```rust
FieldFilter {
    field: String,            // nom du champ JSON à inspecter
    condition: FilterTypes,   // opérateur de comparaison
    values: Vec<Types>,       // liste de valeurs attendues
}
```

L'algorithme appliqué :

1. Lire `event[field]`. S'il est absent → `false`.
2. Pour chaque valeur attendue dans `values`, tester si `(condition, valeur_du_champ, valeur_attendue)` matche.
3. Retourner `true` dès qu'une valeur matche (**OR implicite** sur `values`).

### Conditions supportées

| Condition | Type d'opérande | Sémantique |
|---|---|---|
| `Contains` | String | Le champ contient la sous-chaîne |
| `Startswith` | String | Le champ commence par la chaîne |
| `Endswith` | String | Le champ se termine par la chaîne |
| `Exact` | String | Égalité de chaînes |
| `Exact` | Integer | Égalité d'entiers (i64) |
| `Gt` | Integer | Strictement supérieur |
| `Gte` | Integer | Supérieur ou égal |
| `Lt` | Integer | Strictement inférieur |
| `Lte` | Integer | Inférieur ou égal |

Pour faire un AND, il faut composer plusieurs filtres au niveau supérieur (logique gérée par les règles compilées par Kompiler).

### Cas particuliers et garanties

- **Champ absent** dans l'évènement → `false` (silencieux, pas d'erreur).
- **Type incompatible** (ex. condition numérique sur un champ string, ou inversement) → `false`.
- **Aucune valeur attendue** (`values` vide) → `false` (le `.any()` sur un itérateur vide renvoie `false`).
- L'évaluation est **pure** : pas d'effet de bord, pas d'allocation cachée. Elle peut être appelée en boucle serrée sans surprise.
- L'`EvaluationContext` étant partagé par `Arc`, l'évaluation est sûre à paralléliser.

---

## Compilation et exécution

### Pré-requis

- **Accès Git** aux dépôts `komrad-company/Khronika` et `komrad-company/Kompiler` (publics ou via SSH selon la politique du projet).

### Build

```bash
# Build debug (rapide, non optimisé)
cargo build

# Build release (optimisé)
cargo build --release
```

Le binaire produit se trouve dans `target/release/korelator` (ou `target/debug/korelator`).

### Lancement

```bash
# Avec le chemin par défaut (./configuration.json)
cargo run
```

### Codes de sortie

| Code | Signification |
|---|---|
| `0` | Sortie normale. |
| `1` | Erreur fatale au chargement de la configuration (fichier absent, JSON invalide). |
| `2` | Erreur fatale au parsing des règles (`UnforgivableErrors` remontée par Kompiler). |

### Comportement actuel du binaire (`main.rs`)

Étape par étape :

1. **`load_configuration()`** : lit et désérialise le fichier de conf.
   - Si erreur → `eprintln!` + `exit(1)`.
2. **`intialize_logger(configuration.log)`** : initialise Khronika avec la conf de télémétrie.
3. **`debug!("Korelator successfully initiated")`** : trace de démarrage.
4. **`parse_rules(rules_path)`** : Kompiler charge et compile les règles depuis `rules_path`.
   - Si erreur → `error!` + `exit(2)`.
5. **`dbg!(parsed_rules.len())`** : affiche pour le moment juste le nombre de règles parsées.

> La boucle d'ingestion d'évènements et le déclenchement effectif des alertes ne sont **pas encore branchés** dans `main.rs`. La fondation est posée (config + règles + moteur d'évaluation), il reste à connecter la source (Quickwit) et le sink (alertes).

---

## Tests

Les tests unitaires actuels couvrent les évaluations de filtres dans `src/evaluation_engine/filter.rs` :

| Test | Ce qu'il vérifie |
|---|---|
| `contains_matches_substring` | `Contains` matche bien sur sous-chaîne, et ne matche pas sur valeur absente. |
| `contains_multiple_values_is_or` | Plusieurs valeurs → OR implicite. |
| `startswith_matches_prefix` | `Startswith` matche un préfixe et pas un suffixe. |
| `exact_integer_matches` | `Exact` sur entier. |
| `gt_integer_matches` | `Gt` strict, donc `5` ne matche pas pour `> 5`. |
| `missing_field_returns_false` | Un champ absent renvoie `false` sans paniquer. |

Lancer la suite :

```bash
cargo test
```

---

## Sécurité et audit des dépendances

Le projet utilise [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) pour auditer les dépendances. La politique se trouve dans `deny.toml`.

```bash
cargo deny check
```

### Politique en place

- **Cible** : `x86_64-unknown-linux-gnu`.
- **Avis de sécurité** (`[advisories]`) : version 2, `yanked = "deny"` — toute crate yanked est refusée.
- **Licences autorisées** (`[licenses]`, confiance ≥ 0.90) : MIT, Apache-2.0, BSD-3-Clause, ISC, 0BSD, Zlib, AGPL-3.0, AGPL-3.0-or-later, Unicode-3.0.
- **Bans** :
  - Versions multiples → `warn`.
  - Wildcards de version → `warn`.
- **Sources** :
  - Registries inconnus → `deny`.
  - Dépôts Git inconnus → `deny`.
  - Seuls autorisés : `https://github.com/rust-lang/crates.io-index`, `Khronika.git`, `Kompiler.git`.

Cela garantit qu'aucune dépendance externe non identifiée ne peut entrer dans le projet sans modification explicite de `deny.toml`.

---

## Intégration continue

La CI vit dans `.github/workflows/ci.yml` et délègue à des workflows partagés du dépôt **Kontinuous-integration** :

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

- Le job **`security`** lance la pipeline de sécurité (typiquement `cargo deny`, audits de dépendances, scans).
- Le job **`pipeline`** lance la pipeline Rust standard (build, tests, lint, releases sur tag `v*`) — il dépend du succès du job sécurité.

---

## Roadmap / état actuel

État au moment de la rédaction :

- ✅ Chargement de configuration (JSON + variable d'env).
- ✅ Initialisation du logger Khronika.
- ✅ Parsing des règles via Kompiler.
- ✅ Trait `Evaluate` + impl pour `FieldFilter`.
- ✅ Tests unitaires sur les filtres.
- ✅ CI mutualisée + politique `cargo-deny`.
- ⏳ Implémentation des règles composites (au-delà de `FieldFilter` seul).
- ⚠️ Champ `quickwit_url` requis par la conf mais absent du template d'exemple.
- ⏳ Connexion à Quickwit pour l'ingestion d'évènements.
- ⏳ Module de déclenchement / envoi d'alertes (le « trigger and alert sender »).
- ⏳ Boucle d'évènements dans `main.rs` (actuellement on ne fait que compter les règles parsées).

---

## Licence

Le projet est distribué sous **AGPL-3.0-or-later**. Voir le fichier [`LICENSE`](LICENSE) pour le texte complet.
