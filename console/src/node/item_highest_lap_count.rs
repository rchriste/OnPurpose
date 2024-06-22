use surrealdb::{opt::RecordId, sql::Thing};

use crate::base_data::item::Item;

use super::{
    item_lap_count::ItemLapCount,
    item_node::ItemNode,
    item_status::{ItemStatus, PriorityLevel},
    Filter,
};

#[derive(Clone, Debug)]
pub(crate) struct ItemHighestLapCount<'s> {
    item_lap_count: ItemLapCount<'s>,
    highest_lap_count: ItemLapCount<'s>,
}

impl<'s> ItemHighestLapCount<'s> {
    pub(crate) fn new(item_lap_count: ItemLapCount<'s>, all_status: &[ItemLapCount<'s>]) -> Self {
        let highest_lap_count = {
            calculate_highest_lap_count_item(&item_lap_count, all_status, Vec::default()).clone()
        };
        Self {
            item_lap_count,
            highest_lap_count,
        }
    }

    pub(crate) fn get_item_lap_count(&'s self) -> &'s ItemLapCount<'s> {
        &self.item_lap_count
    }

    pub(crate) fn has_children(&self, filter: Filter) -> bool {
        self.item_lap_count.has_children(filter)
    }

    pub(crate) fn get_highest_lap_count_item(&self) -> &ItemLapCount<'s> {
        &self.highest_lap_count
    }

    pub(crate) fn get_surreal_record_id(&self) -> &RecordId {
        self.item_lap_count.get_surreal_record_id()
    }

    pub(crate) fn get_summary(&self) -> &str {
        self.item_lap_count.get_summary()
    }

    pub(crate) fn get_item(&self) -> &Item {
        self.item_lap_count.get_item()
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.item_lap_count.is_finished()
    }

    pub(crate) fn get_item_node(&self) -> &ItemNode {
        self.item_lap_count.get_item_node()
    }

    pub(crate) fn has_larger(&self, filter: Filter) -> bool {
        self.item_lap_count.has_larger(filter)
    }

    pub(crate) fn is_type_motivation(&self) -> bool {
        self.item_lap_count.is_type_motivation()
    }

    pub(crate) fn get_item_status(&self) -> &ItemStatus {
        self.item_lap_count.get_item_status()
    }

    pub(crate) fn is_snoozed(&self) -> bool {
        self.item_lap_count.is_snoozed()
    }

    pub(crate) fn is_person_or_group(&self) -> bool {
        self.item_lap_count.is_person_or_group()
    }

    pub(crate) fn get_thing(&self) -> &Thing {
        self.item_lap_count.get_thing()
    }

    pub(crate) fn is_type_undeclared(&self) -> bool {
        self.item_lap_count.is_type_undeclared()
    }

    pub(crate) fn is_staging_not_set(&self) -> bool {
        self.item_lap_count.is_staging_not_set()
    }

    pub(crate) fn get_lap_count(&self) -> f32 {
        self.item_lap_count.get_lap_count()
    }

    pub(crate) fn get_priority_level(&self) -> PriorityLevel {
        self.item_lap_count.get_priority_level()
    }
}

fn calculate_highest_lap_count_item<'a, 'b>(
    item_lap_count: &'a ItemLapCount<'b>,
    all_item_status: &'a [ItemLapCount<'b>],
    mut visited: Vec<&'a ItemLapCount<'b>>,
) -> &'a ItemLapCount<'b> {
    if visited.contains(&item_lap_count) {
        return item_lap_count;
    } else {
        visited.push(item_lap_count);
    }
    if item_lap_count.has_children(Filter::Active) {
        let highest_lap_count = item_lap_count
            .get_smaller(Filter::Active)
            .map(|x| {
                all_item_status
                    .iter()
                    .find(|y| y.get_item() == x.get_item())
                    .expect("Comes from this list so it will always be there")
            })
            .reduce(|a, b| {
                let a_highest =
                    calculate_highest_lap_count_item(a, all_item_status, visited.clone());
                let b_highest =
                    calculate_highest_lap_count_item(b, all_item_status, visited.clone());
                if a_highest.get_lap_count() > b_highest.get_lap_count() {
                    a_highest
                } else {
                    b_highest
                }
            })
            .expect("has_children is true so there is at least one item");

        // Reduce is not called if there is only one child so in that scenario this is needed to ensure that we select the deepest child
        if highest_lap_count.has_children(Filter::Active) {
            let children = highest_lap_count
                .get_smaller(Filter::Active)
                .map(|x| {
                    all_item_status
                        .iter()
                        .find(|y| y.get_item() == x.get_item())
                        .expect("Comes from this list so it will always be there")
                })
                .collect::<Vec<_>>();
            let child = children
                .into_iter()
                .reduce(|a, b| {
                    let a_highest =
                        calculate_highest_lap_count_item(a, all_item_status, visited.clone());
                    let b_highest =
                        calculate_highest_lap_count_item(b, all_item_status, visited.clone());
                    if a_highest.get_lap_count() > b_highest.get_lap_count() {
                        a_highest
                    } else {
                        b_highest
                    }
                })
                .expect("This if statement is for has_children");
            let a = calculate_highest_lap_count_item(child, all_item_status, visited);
            a
        } else {
            assert!(!highest_lap_count.has_children(Filter::Active), "This should only happen if reduce is never called, meaning there is only one child, highest_lap_count summary: {}", highest_lap_count.get_summary());
            highest_lap_count
        }
    } else {
        assert!(
            !item_lap_count.has_children(Filter::Active),
            "This should only happen if reduce is never called, meaning there is only one child, item_status summary: {}",
            item_lap_count.get_summary());
        item_lap_count
    }
}
