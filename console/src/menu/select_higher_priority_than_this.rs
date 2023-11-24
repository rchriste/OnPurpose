use std::fmt::Display;

use inquire::Select;
use surrealdb::opt::RecordId;

use crate::{base_data::item::Item, display::display_item::DisplayItem};

enum HigherPriorityThan<'e> {
    Item(DisplayItem<'e>),
    PutAtTheTop,
}

impl Display for HigherPriorityThan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HigherPriorityThan::Item(display_item) => write!(f, "{}", display_item),
            HigherPriorityThan::PutAtTheTop => write!(f, "Put at the top"),
        }
    }
}

impl<'e> HigherPriorityThan<'e> {
    pub(crate) fn create_list(items: &'e [&'e Item<'_>]) -> Vec<HigherPriorityThan<'e>> {
        let mut list = Vec::with_capacity(items.len() + 1);
        list.push(HigherPriorityThan::PutAtTheTop);
        for item in items.iter() {
            let display_item = DisplayItem::new(item);
            list.push(HigherPriorityThan::Item(display_item));
        }
        list
    }
}

pub(crate) fn select_higher_priority_than_this(items: &[&Item<'_>]) -> Option<RecordId> {
    let list = HigherPriorityThan::create_list(items);
    let selected = Select::new("Select put priority after this|", list)
        .prompt()
        .unwrap();
    match selected {
        HigherPriorityThan::Item(display_item) => {
            let surreal_item = display_item.get_surreal_record_id();
            Some(surreal_item.clone())
        }
        HigherPriorityThan::PutAtTheTop => None,
    }
}
