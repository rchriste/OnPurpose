use std::fmt::Display;

use inquire::Select;

use crate::{
    base_data::item::Item, display::display_item::DisplayItem,
    surrealdb_layer::surreal_item::SurrealItem,
};

enum HigherPriorityThan<'e> {
    Item(DisplayItem<'e>),
    PutAtTheBottom,
}

impl Display for HigherPriorityThan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HigherPriorityThan::Item(display_item) => write!(f, "{}", display_item),
            HigherPriorityThan::PutAtTheBottom => write!(f, "Put at the bottom"),
        }
    }
}

impl<'e> HigherPriorityThan<'e> {
    pub(crate) fn create_list(items: &'e [&'e Item<'_>]) -> Vec<HigherPriorityThan<'e>> {
        let mut list = Vec::with_capacity(items.len() + 1);
        for item in items.iter() {
            let display_item = DisplayItem::new(item);
            list.push(HigherPriorityThan::Item(display_item));
        }
        list.push(HigherPriorityThan::PutAtTheBottom);
        list
    }
}

pub(crate) fn select_higher_priority_than_this(items: &[&Item<'_>]) -> Option<SurrealItem> {
    let list = HigherPriorityThan::create_list(items);
    let selected = Select::new("Select higher priority than this", list)
        .prompt()
        .unwrap();
    match selected {
        HigherPriorityThan::Item(display_item) => {
            let surreal_item = display_item.get_surreal_item();
            Some(surreal_item.clone())
        }
        HigherPriorityThan::PutAtTheBottom => None,
    }
}
