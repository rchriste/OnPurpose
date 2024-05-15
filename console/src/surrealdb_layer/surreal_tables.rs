use tokio::sync::mpsc::Sender;

#[cfg(test)]
use derive_builder::Builder;

use crate::base_data::{
    covering::Covering,
    covering_until_date_time::CoveringUntilDateTime,
    item::{Item, ItemVecExtensions},
    life_area::LifeArea,
    routine::Routine,
    time_spent::TimeSpent,
};

use super::{
    surreal_covering::SurrealCovering,
    surreal_covering_until_date_time::SurrealCoveringUntilDatetime, surreal_item::SurrealItem,
    surreal_life_area::SurrealLifeArea, surreal_required_circumstance::SurrealRequiredCircumstance,
    surreal_routine::SurrealRoutine, surreal_time_spent::SurrealTimeSpent, DataLayerCommands,
};

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Builder), builder(setter(into)))]
pub(crate) struct SurrealTables {
    #[cfg_attr(test, builder(default))]
    pub(crate) surreal_items: Vec<SurrealItem>,

    #[cfg_attr(test, builder(default))]
    pub(crate) surreal_coverings: Vec<SurrealCovering>,

    #[cfg_attr(test, builder(default))]
    pub(crate) surreal_required_circumstances: Vec<SurrealRequiredCircumstance>,

    #[cfg_attr(test, builder(default))]
    pub(crate) surreal_coverings_until_date_time: Vec<SurrealCoveringUntilDatetime>,

    #[cfg_attr(test, builder(default))]
    pub(crate) surreal_life_areas: Vec<SurrealLifeArea>,

    #[cfg_attr(test, builder(default))]
    pub(crate) surreal_routines: Vec<SurrealRoutine>,

    #[cfg_attr(test, builder(default))]
    pub(crate) surreal_time_spent_log: Vec<SurrealTimeSpent>,
}

impl SurrealTables {
    pub(crate) async fn new(
        sender: &Sender<DataLayerCommands>,
    ) -> Result<Self, tokio::sync::oneshot::error::RecvError> {
        DataLayerCommands::get_raw_data(sender).await
    }

    pub(crate) fn make_items(&self) -> Vec<Item<'_>> {
        self.surreal_items
            .iter()
            .map(|x| x.make_item(&self.surreal_required_circumstances))
            .collect()
    }

    pub(crate) fn make_coverings<'a>(&'a self, items: &'a [&'a Item<'a>]) -> Vec<Covering<'a>> {
        self.surreal_coverings
            .iter()
            .filter_map(|x| {
                //Items that are not found should just be filtered out. The main scenario for this is to filter out covering for items that are finished
                let smaller = items.lookup_from_record_id(&x.smaller)?;
                let parent = items.lookup_from_record_id(&x.parent)?;
                Some(Covering {
                    smaller,
                    parent,
                    _surreal_covering: x,
                })
            })
            .collect()
    }

    pub(crate) fn make_coverings_until_date_time<'a>(
        &'a self,
        items: &'a [&'a Item<'a>],
    ) -> Vec<CoveringUntilDateTime<'a>> {
        self.surreal_coverings_until_date_time
            .iter()
            .filter_map(|x| {
                let cover_this = items.lookup_from_record_id(&x.cover_this)?;
                let until = x.until.clone().into();
                Some(CoveringUntilDateTime { cover_this, until })
            })
            .collect()
    }

    pub(crate) fn make_life_areas(&self) -> Vec<LifeArea<'_>> {
        self.surreal_life_areas.iter().map(LifeArea::new).collect()
    }

    pub(crate) fn make_routines(&self) -> Vec<Routine<'_>> {
        self.surreal_routines.iter().map(Routine::new).collect()
    }

    pub(crate) fn make_time_spent_log(&self) -> impl Iterator<Item = TimeSpent<'_>> {
        self.surreal_time_spent_log.iter().map(TimeSpent::new)
    }
}
