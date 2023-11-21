use std::fmt::Display;

use surrealdb::opt::RecordId;

use crate::{base_data::item::Item, surrealdb_layer::surreal_item::ItemType};

/// DisplayItem was created to make it centralize all of the different ways of displaying or printing an Item without
/// putting that onto the core Item type that should not be tied to specific display or printing logic for a console
/// application.
pub(crate) struct DisplayItem<'s> {
    pub(crate) item: &'s Item<'s>,
}

impl<'s> From<&&'s Item<'s>> for DisplayItem<'s> {
    fn from(item: &&'s Item<'s>) -> Self {
        DisplayItem { item }
    }
}

impl<'s> From<&'s Item<'s>> for DisplayItem<'s> {
    fn from(item: &'s Item<'s>) -> Self {
        DisplayItem { item }
    }
}

impl<'s> From<DisplayItem<'s>> for &'s Item<'s> {
    fn from(item: DisplayItem<'s>) -> Self {
        item.item
    }
}

impl Display for DisplayItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.item.get_item_type() {
            ItemType::Goal(_) => write!(f, "🪧 {}", self.item.get_summary()),
            ItemType::Motivation => write!(f, "🎯 {}", self.item.get_summary()),
            ItemType::Action => write!(f, "🪜 {}", self.item.get_summary()),
            ItemType::Simple => write!(f, "📌 {}", self.item.get_summary()),
            ItemType::Undeclared => write!(f, "❓ {}", self.item.get_summary()),
            ItemType::PersonOrGroup => write!(f, "👤 {}", self.item.get_summary()),
            ItemType::IdeaOrThought => write!(f, "💡 {}", self.item.get_summary()),
        }
    }
}

impl<'s> DisplayItem<'s> {
    pub(crate) fn new(item: &'s Item<'s>) -> Self {
        DisplayItem { item }
    }

    pub(crate) fn make_list(items: &'s [&'s Item<'s>]) -> Vec<Self> {
        items.iter().map(|x| DisplayItem::new(x)).collect()
    }

    pub(crate) fn get_surreal_record_id(&'s self) -> &'s RecordId {
        self.item.get_surreal_record_id()
    }
}
