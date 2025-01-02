use chrono::{DateTime, Utc};
use derive_builder::Builder;

#[derive(Builder, Clone, Debug)]
#[builder(setter(into))]
pub(crate) struct NewEvent {
    pub(crate) summary: String,

    #[builder(default = "false")]
    pub(crate) triggered: bool,

    #[builder(default = "Utc::now()")]
    pub(crate) last_updated: DateTime<Utc>,
}
