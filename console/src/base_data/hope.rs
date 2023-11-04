use surrealdb::sql::{Datetime, Thing};

use crate::surrealdb_layer::surreal_item::{ItemType, SurrealItem};

use super::{covering::Covering, item::Item};

/// Could have a review_type with options for Milestone, StoppingPoint, and ReviewPoint
#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct Hope<'a> {
    pub(crate) id: &'a Thing,
    pub(crate) summary: &'a str,
    pub(crate) finished: &'a Option<Datetime>,
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

impl<'s> From<&&'s Hope<'s>> for &'s Item<'s> {
    fn from(value: &&'s Hope<'s>) -> Self {
        value.item
    }
}

impl<'s> From<&'s Hope<'s>> for &'s Item<'s> {
    fn from(value: &'s Hope<'s>) -> Self {
        value.item
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
    pub(crate) fn new(item: &'a Item) -> Self {
        //TODO: Add assert that it is a hope
        Hope {
            id: item.get_id(),
            summary: item.get_summary(),
            finished: item.get_finished(),
            item,
        }
    }

    pub(crate) fn is_mentally_resident(&self) -> bool {
        self.item.is_mentally_resident()
    }

    pub(crate) fn is_staging_not_set(&self) -> bool {
        self.item.is_staging_not_set()
    }

    pub(crate) fn is_project(&self) -> bool {
        self.item.is_project()
    }

    pub(crate) fn is_permanence_not_set(&self) -> bool {
        self.item.is_permanence_not_set()
    }

    pub(crate) fn is_maintenance(&self) -> bool {
        self.item.is_maintenance()
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.finished.is_some()
    }

    pub(crate) fn is_covered_by_another_hope(&self, coverings: &[Covering<'_>]) -> bool {
        let mut covered_by = coverings.iter().filter(|x| {
            self == x.parent && x.smaller.item_type == &ItemType::Hope && !x.smaller.is_finished()
        });
        //Now see if the items that are covering are finished or active
        covered_by.any(|x| !x.smaller.is_finished())
    }

    pub(crate) fn covered_by(
        &self,
        coverings: &[Covering<'a>],
        all_items: &'a [&Item<'a>],
    ) -> Vec<&'a Item<'a>> {
        self.item.covered_by(coverings, all_items)
    }

    pub(crate) fn get_surreal_item(&self) -> &'a SurrealItem {
        self.item.get_surreal_item()
    }

    pub(crate) fn get_item(&self) -> &'a Item<'a> {
        self.item
    }
}
