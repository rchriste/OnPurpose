use surrealdb::sql::Datetime;

use crate::surrealdb_layer::surreal_item::{ItemType, Responsibility};

pub(crate) struct NewItem {
    pub(crate) summary: String,
    pub(crate) finished: Option<Datetime>,
    pub(crate) responsibility: Responsibility,
    pub(crate) item_type: ItemType,
}

impl NewItem {
    pub(crate) fn new(summary: String) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::default(),
            item_type: ItemType::Undeclared,
        }
    }

    pub(crate) fn new_action(summary: String) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::default(),
            item_type: ItemType::ToDo,
        }
    }

    pub(crate) fn new_goal(summary: String) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::default(),
            item_type: ItemType::Hope,
        }
    }

    pub(crate) fn new_motivation(summary: String) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::default(),
            item_type: ItemType::Motivation,
        }
    }

    pub(crate) fn new_person_or_group(summary: String) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::ReactiveBeAvailableToAct,
            item_type: ItemType::PersonOrGroup,
        }
    }
}
