use surrealdb::{opt::RecordId, sql::Thing};

use crate::{
    base_data::item::Item,
    surrealdb_layer::surreal_item::{ItemType, SurrealStaging},
};

use super::{
    item_node::ItemNode,
    item_status::{ItemStatus, PriorityLevel},
    Filter,
};

#[derive(Clone, Debug)]
pub(crate) struct ItemLapCount<'s> {
    item_status: ItemStatus<'s>,
    lap_count: f32,
    smaller: Vec<ItemStatus<'s>>,
}

impl PartialEq for ItemLapCount<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.item_status.get_item() == other.item_status.get_item()
    }
}

impl<'s> ItemLapCount<'s> {
    pub(crate) fn new(item_status: ItemStatus<'s>, all_status: &[ItemStatus<'s>]) -> Self {
        let lap_count = item_status.get_lap_count();
        let lap_count = lap_count.resolve(all_status);
        let smaller = calculate_smaller(&item_status, all_status, Filter::All);
        Self {
            item_status,
            lap_count,
            smaller,
        }
    }

    pub(crate) fn get_type(&self) -> &ItemType {
        self.item_status.get_type()
    }

    pub(crate) fn get_item_status(&self) -> &ItemStatus {
        &self.item_status
    }

    pub(crate) fn get_lap_count(&self) -> f32 {
        self.lap_count
    }

    pub(crate) fn get_surreal_record_id(&self) -> &RecordId {
        self.item_status.get_surreal_record_id()
    }

    pub(crate) fn get_item_node(&self) -> &ItemNode {
        self.item_status.get_item_node()
    }

    pub(crate) fn get_item(&self) -> &Item {
        self.item_status.get_item()
    }

    pub(crate) fn get_staging(&self) -> &SurrealStaging {
        self.item_status.get_staging()
    }

    pub(crate) fn is_snoozed(&self) -> bool {
        self.item_status.is_snoozed()
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.item_status.is_finished()
    }

    pub(crate) fn is_person_or_group(&self) -> bool {
        self.item_status.is_person_or_group()
    }

    pub(crate) fn has_children(&self, filter: Filter) -> bool {
        self.item_status.has_children(filter)
    }

    pub(crate) fn has_larger(&self, filter: Filter) -> bool {
        self.item_status.has_larger(filter)
    }

    pub(crate) fn get_thing(&self) -> &Thing {
        self.item_status.get_thing()
    }

    pub(crate) fn is_type_undeclared(&self) -> bool {
        self.item_status.is_type_undeclared()
    }

    pub(crate) fn is_staging_not_set(&self) -> bool {
        self.item_status.is_staging_not_set()
    }

    pub(crate) fn is_staging_mentally_resident(&self) -> bool {
        self.item_status.is_staging_mentally_resident()
    }

    pub(crate) fn get_smaller(
        &'s self,
        filter: Filter,
    ) -> Box<dyn Iterator<Item = &'s ItemStatus<'s>> + 's + Send> {
        match filter {
            Filter::Active => Box::new(self.smaller.iter().filter(|x| x.is_active())),
            Filter::All => Box::new(self.smaller.iter()),
            Filter::Finished => Box::new(self.smaller.iter().filter(|x| x.is_finished())),
        }
    }

    pub(crate) fn get_summary(&self) -> &str {
        self.item_status.get_summary()
    }

    pub(crate) fn is_type_motivation(&self) -> bool {
        self.item_status.is_type_motivation()
    }

    pub(crate) fn get_priority_level(&self) -> PriorityLevel {
        self.item_status.get_priority_level()
    }
}

fn calculate_smaller<'a>(
    item_status: &ItemStatus<'a>,
    all_status: &[ItemStatus<'a>],
    filter: Filter,
) -> Vec<ItemStatus<'a>> {
    item_status
        .get_smaller(filter)
        .map(|x| {
            all_status
                .iter()
                .find(|y| y.get_item() == x.get_item())
                .expect("Comes from this list so it will always be there")
                .clone()
        })
        .collect()
}
