#![forbid(unsafe_code)]

pub(crate) mod alert;
pub(crate) mod configuration;
pub(crate) mod datasources;
pub(crate) mod errors;
pub(crate) mod evaluation_engine;

pub use alert::{Alert, AlertSink, StderrJsonSink};
pub use configuration::{Configuration, load as load_configuration};
pub use datasources::{DatasourceType, QuickwitSource, StdinSource};
pub use errors::Error;
pub use evaluation_engine::{Evaluate, EvaluationContext, PreparedRule};
pub use kompiler::RuleLevel;
