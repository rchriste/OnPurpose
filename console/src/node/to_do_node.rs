use chrono::{DateTime, Local};

use crate::base_data::{item::Item, to_do::ToDo, Covering, CoveringUntilDateTime};

use super::{create_growing_nodes, GrowingItemNode};

pub struct ToDoNode<'a> {
    pub to_do: &'a ToDo<'a>,
    pub larger: Vec<GrowingItemNode<'a>>,
}

pub fn create_to_do_nodes<'a>(
    next_steps: &'a [ToDo],
    coverings: &'a [Covering<'a>],
    coverings_until_date_time: &'a [CoveringUntilDateTime<'a>],
    current_date: &DateTime<Local>,
    currently_in_focus_time: bool,
) -> Vec<ToDoNode<'a>> {
    next_steps
        .iter()
        .filter_map(|x| {
            if !x.is_covered(coverings, coverings_until_date_time, current_date)
                && !x.is_finished()
                && x.is_circumstances_met(current_date, currently_in_focus_time)
            {
                Some(create_to_do_node(x, coverings))
            } else {
                None
            }
        })
        .collect()
}

pub fn create_to_do_node<'a>(to_do: &'a ToDo, coverings: &'a [Covering<'a>]) -> ToDoNode<'a> {
    let item: &Item = to_do.into();
    let parents = item.find_parents(coverings);
    let larger = create_growing_nodes(parents, coverings);

    ToDoNode { to_do, larger }
}

impl<'a> ToDoNode<'a> {
    #[allow(dead_code)]
    pub fn get_summary(&'a self) -> &'a str {
        self.to_do.get_summary()
    }
}
