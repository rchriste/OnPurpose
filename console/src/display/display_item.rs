use std::fmt::Display;

use surrealdb::opt::RecordId;

use crate::{
    base_data::item::Item,
    surrealdb_layer::surreal_item::{SurrealItemType, SurrealMotivationKind},
};

/// DisplayItem was created to make it centralize all of the different ways of displaying or printing an Item without
/// putting that onto the core Item type that should not be tied to specific display or printing logic for a console
/// application.
#[derive(Debug)]
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

impl From<DisplayItem<'_>> for RecordId {
    fn from(display_item: DisplayItem<'_>) -> Self {
        display_item.item.get_surreal_record_id().clone()
    }
}

impl Display for DisplayItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.item.get_item_type() {
            SurrealItemType::Goal(_) => write!(f, "🪧 {}", self.item.get_summary()),
            SurrealItemType::Motivation(kind) => {
                write!(f, "🎯")?;
                match kind {
                    SurrealMotivationKind::NotSet => {}
                    SurrealMotivationKind::CoreWork => write!(f, "🏢")?,
                    SurrealMotivationKind::NonCoreWork => write!(f, "🏞")?,
                }
                write!(f, " {}", self.item.get_summary())
            }
            SurrealItemType::Action => write!(f, "🪜 {}", self.item.get_summary()),
            SurrealItemType::Undeclared => write!(f, "❓ {}", self.item.get_summary()),
            SurrealItemType::PersonOrGroup => write!(f, "👤 {}", self.item.get_summary()),
            SurrealItemType::IdeaOrThought => write!(f, "💡 {}", self.item.get_summary()),
        }
    }
}

impl<'s> DisplayItem<'s> {
    pub(crate) fn new(item: &'s Item<'s>) -> Self {
        DisplayItem { item }
    }

    pub(crate) fn get_surreal_record_id(&'s self) -> &'s RecordId {
        self.item.get_surreal_record_id()
    }
}
