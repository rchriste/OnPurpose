use chrono::{DateTime, Local};

use crate::base_data::{
    grouped_item::GroupedItem, item::Item, person_or_group::PersonOrGroup, Covering,
    CoveringUntilDateTime,
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

    pub(crate) fn create_grouped_work(&self) -> Vec<GroupedItem<'s>> {
        todo!("I will need to plumb through the information back to the Surreal layer to know why the item is being grouped in that place. Probably one of the grouped types to store in the DB is to carry the reason from the parent item or something to that effect. I am not quite sure how to properly hook this up")
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
            if !x.is_covered(coverings, coverings_until_date_time, current_date)
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
