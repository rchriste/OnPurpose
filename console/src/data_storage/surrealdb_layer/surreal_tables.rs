use chrono::{DateTime, Utc};
use tokio::sync::mpsc::Sender;

#[cfg(test)]
use derive_builder::Builder;

use crate::base_data::{item::Item, time_spent::TimeSpent};

use super::{
    data_layer_commands::DataLayerCommands,
    surreal_in_the_moment_priority::SurrealInTheMomentPriority, surreal_item::SurrealItem,
    surreal_time_spent::SurrealTimeSpent,
};

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Builder), builder(setter(into)))]
pub(crate) struct SurrealTables {
    #[cfg_attr(test, builder(default))]
    pub(crate) surreal_items: Vec<SurrealItem>,

    #[cfg_attr(test, builder(default))]
    pub(crate) surreal_time_spent_log: Vec<SurrealTimeSpent>,

    #[cfg_attr(test, builder(default))]
    pub(crate) surreal_in_the_moment_priorities: Vec<SurrealInTheMomentPriority>,
}

impl SurrealTables {
    pub(crate) async fn new(
        sender: &Sender<DataLayerCommands>,
    ) -> Result<Self, tokio::sync::oneshot::error::RecvError> {
        DataLayerCommands::get_raw_data(sender).await
    }

    pub(crate) fn make_items<'a>(&'a self, now: &'a DateTime<Utc>) -> Vec<Item<'a>> {
        self.surreal_items
            .iter()
            .map(|x| x.make_item(now))
            .collect()
    }

    pub(crate) fn make_time_spent_log(&self) -> impl Iterator<Item = TimeSpent<'_>> {
        self.surreal_time_spent_log.iter().map(TimeSpent::new)
    }

    pub(crate) fn get_surreal_in_the_moment_priorities(&self) -> &[SurrealInTheMomentPriority] {
        &self.surreal_in_the_moment_priorities
    }
}
