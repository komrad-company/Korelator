#![forbid(unsafe_code)]

pub(crate) mod alert;
pub(crate) mod configuration;
pub(crate) mod datasources;
pub(crate) mod errors;
pub(crate) mod evaluation_engine;
pub(crate) mod store;

pub use alert::{AlertSink, StderrJsonSink};
pub use configuration::{Configuration, load as load_configuration};
pub use datasources::{DatasourceType, QuickwitSource, StdinSource, run as run_datasource};
pub use errors::Error;
pub use evaluation_engine::{PreparedRule, load_rules};
pub use kompiler::RuleLevel;
pub use store::AlertStore;
