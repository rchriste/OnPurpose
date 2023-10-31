use chrono::{DateTime, Utc};
use tokio::sync::mpsc::Sender;

use crate::base_data::{
    covering::Covering,
    covering_until_date_time::CoveringUntilDateTime,
    item::{Item, ItemVecExtensions},
    life_area::LifeArea,
    routine::Routine,
};

use super::{
    surreal_covering::SurrealCovering,
    surreal_covering_until_date_time::SurrealCoveringUntilDatetime, surreal_item::SurrealItem,
    surreal_life_area::SurrealLifeArea, surreal_required_circumstance::SurrealRequiredCircumstance,
    surreal_routine::SurrealRoutine, surreal_specific_to_hope::SurrealSpecificToHope,
    DataLayerCommands,
};

#[derive(Debug)]
pub(crate) struct SurrealTables {
    pub(crate) surreal_items: Vec<SurrealItem>,
    pub(crate) surreal_specific_to_hopes: Vec<SurrealSpecificToHope>,
    pub(crate) surreal_coverings: Vec<SurrealCovering>,
    pub(crate) surreal_required_circumstances: Vec<SurrealRequiredCircumstance>,
    pub(crate) surreal_coverings_until_date_time: Vec<SurrealCoveringUntilDatetime>,
    pub(crate) surreal_life_areas: Vec<SurrealLifeArea>,
    pub(crate) surreal_routines: Vec<SurrealRoutine>,
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

    pub(crate) fn make_coverings<'a>(&'a self, items: &'a [Item<'a>]) -> Vec<Covering<'a>> {
        self.surreal_coverings
            .iter()
            .map(|x| Covering {
                smaller: items.lookup_from_record_id(&x.smaller).unwrap(),
                parent: items.lookup_from_record_id(&x.parent).unwrap(),
                surreal_covering: x,
            })
            .collect()
    }

    pub(crate) fn make_coverings_until_date_time<'a>(
        &'a self,
        items: &'a [Item<'a>],
    ) -> Vec<CoveringUntilDateTime<'a>> {
        self.surreal_coverings_until_date_time
            .iter()
            .map(|x| {
                let until_utc: DateTime<Utc> = x.until.clone().into();
                CoveringUntilDateTime {
                    cover_this: items.lookup_from_record_id(&x.cover_this).unwrap(),
                    until: until_utc.into(),
                }
            })
            .collect()
    }

    pub(crate) fn make_life_areas(&self) -> Vec<LifeArea<'_>> {
        self.surreal_life_areas.iter().map(LifeArea::new).collect()
    }

    pub(crate) fn make_routines(&self) -> Vec<Routine<'_>> {
        self.surreal_routines.iter().map(Routine::new).collect()
    }
}
