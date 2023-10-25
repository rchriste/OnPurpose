use chrono::{DateTime, Local};

use crate::{base_data::ItemType, surrealdb_layer::surreal_item::SurrealItem};

use super::{item::Item, Covering, CoveringUntilDateTime};

pub(crate) struct PersonOrGroup<'s> {
    item: &'s Item<'s>,
}

impl<'s> PersonOrGroup<'s> {
    pub(crate) fn new(item: &'s Item<'s>) -> Self {
        assert!(item.item_type == &ItemType::PersonOrGroup);
        Self { item }
    }

    pub(crate) fn get_summary(&self) -> &str {
        self.item.summary
    }

    pub(crate) fn get_item(&self) -> &Item<'s> {
        self.item
    }

    pub(crate) fn get_surreal_item(&self) -> &SurrealItem {
        self.item.get_surreal_item()
    }

    pub(crate) fn is_covering_another_item(&self, coverings: &[Covering<'_>]) -> bool {
        self.item.is_covering_another_item(coverings)
    }

    pub(crate) fn is_covered(
        &'s self,
        coverings: &'s [Covering<'s>],
        coverings_until_date_time: &'s [CoveringUntilDateTime<'s>],
        now: &DateTime<Local>,
    ) -> bool {
        self.item
            .is_covered(coverings, coverings_until_date_time, now)
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.item.is_finished()
    }

    pub(crate) fn is_circumstances_met(
        &self,
        now: &DateTime<Local>,
        currently_in_focus_time: bool,
    ) -> bool {
        self.item.is_circumstances_met(now, currently_in_focus_time)
    }
}
