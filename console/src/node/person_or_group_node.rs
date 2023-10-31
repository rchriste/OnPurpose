use chrono::{DateTime, Local};

use crate::{
    base_data::{
        covering::Covering, covering_until_date_time::CoveringUntilDateTime, item::Item,
        person_or_group::PersonOrGroup,
    },
    surrealdb_layer::surreal_item::SurrealItem,
};

use super::item_node::ItemNode;

pub(crate) struct PersonOrGroupNode<'s> {
    person_or_group: &'s PersonOrGroup<'s>,
    item_node: ItemNode<'s>,
}

impl<'s> PersonOrGroupNode<'s> {
    pub(crate) fn new(
        person_or_group: &'s PersonOrGroup<'s>,
        coverings: &'s [Covering<'s>],
        possible_parents: &'s [&'s Item<'s>],
    ) -> Self {
        let item_node = ItemNode::new(person_or_group.get_item(), coverings, possible_parents);

        Self {
            person_or_group,
            item_node,
        }
    }

    pub(crate) fn person_or_group(&self) -> &'s PersonOrGroup<'s> {
        self.person_or_group
    }

    pub(crate) fn create_parent_chain(&'s self) -> Vec<&'s Item<'s>> {
        self.item_node.create_next_step_parents()
    }

    pub(crate) fn get_surreal_item(&'s self) -> &'s SurrealItem {
        self.item_node.get_surreal_item()
    }
}

pub(crate) fn create_person_or_group_nodes<'s>(
    create_nodes_from: &'s [PersonOrGroup<'s>],
    coverings: &'s [Covering<'s>],
    coverings_until_date_time: &'s [CoveringUntilDateTime<'s>],
    items: &'s [&'s Item<'s>],
    current_date: &DateTime<Local>,
    currently_in_focus_time: bool,
) -> Vec<PersonOrGroupNode<'s>> {
    create_nodes_from
        .iter()
        .filter_map(|x| {
            if !x.is_covered(coverings, coverings_until_date_time, items, current_date)
                && !x.is_finished()
                && x.is_circumstances_met(current_date, currently_in_focus_time)
            {
                Some(PersonOrGroupNode::new(x, coverings, items))
            } else {
                None
            }
        })
        .collect()
}
