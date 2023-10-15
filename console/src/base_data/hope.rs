use chrono::{DateTime, Local};
use surrealdb::sql::{Datetime, Thing};

use crate::surrealdb_layer::{
    surreal_item::SurrealItem,
    surreal_specific_to_hope::{Permanence, Staging, SurrealSpecificToHope},
};

use super::{item::Item, Covering, CoveringUntilDateTime, ItemType};

/// Could have a review_type with options for Milestone, StoppingPoint, and ReviewPoint
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Hope<'a> {
    pub id: &'a Thing,
    pub summary: &'a String,
    pub finished: &'a Option<Datetime>,
    pub hope_specific: SurrealSpecificToHope,
    item: &'a Item<'a>,
}

impl<'a> From<Hope<'a>> for Thing {
    fn from(value: Hope) -> Self {
        value.id.clone()
    }
}

impl<'a> From<&'a Hope<'a>> for &'a SurrealItem {
    fn from(value: &'a Hope<'a>) -> Self {
        value.item.into()
    }
}

impl PartialEq<Hope<'_>> for Item<'_> {
    fn eq(&self, other: &Hope<'_>) -> bool {
        self == other.item
    }
}

impl PartialEq<Item<'_>> for Hope<'_> {
    fn eq(&self, other: &Item) -> bool {
        self.item == other
    }
}

impl<'a> Hope<'a> {
    pub(crate) fn new(item: &'a Item, hope_specific: SurrealSpecificToHope) -> Self {
        //TODO: Add assert that it is a hope
        Hope {
            id: item.id,
            summary: item.summary,
            finished: item.finished,
            hope_specific,
            item,
        }
    }

    pub fn is_mentally_resident(&self) -> bool {
        self.hope_specific.staging == Staging::MentallyResident
    }

    pub fn is_project(&self) -> bool {
        self.hope_specific.permanence == Permanence::Project
            && self.hope_specific.staging == Staging::MentallyResident
    }

    pub fn is_maintenance(&self) -> bool {
        self.hope_specific.permanence == Permanence::Maintenance
    }

    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }

    pub fn is_covered_by_another_hope(&self, coverings: &[Covering<'_>]) -> bool {
        let mut covered_by = coverings.iter().filter(|x| {
            self == x.parent && x.smaller.item_type == &ItemType::Hope && !x.smaller.is_finished()
        });
        //Now see if the items that are covering are finished or active
        covered_by.any(|x| !x.smaller.is_finished())
    }

    pub fn is_covered_by_another_item(&self, coverings: &[Covering<'_>]) -> bool {
        self.item.is_covered_by_another_item(coverings)
    }

    pub fn is_covered_by_date_time(
        &self,
        coverings_until_date_time: &[CoveringUntilDateTime<'_>],
        now: &DateTime<Local>,
    ) -> bool {
        self.item
            .is_covered_by_date_time(coverings_until_date_time, now)
    }

    pub fn is_covered(
        &self,
        coverings: &[Covering<'_>],
        coverings_until_date_time: &[CoveringUntilDateTime<'_>],
        now: &DateTime<Local>,
    ) -> bool {
        self.item
            .is_covered(coverings, coverings_until_date_time, now)
    }

    pub fn covered_by(&self, coverings: &[Covering<'a>]) -> Vec<&'a Item<'a>> {
        self.item.covered_by(coverings)
    }

    pub fn who_am_i_covering(&self, coverings: &[Covering<'a>]) -> Vec<&'a Item<'a>> {
        self.item.who_am_i_covering(coverings)
    }

    pub fn get_surreal_item(&self) -> &'a SurrealItem {
        self.item.surreal_item
    }

    pub fn get_item(&self) -> &'a Item<'a> {
        self.item
    }
}
