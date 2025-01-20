use surrealdb::sql::Thing;

use crate::{data_storage::surrealdb_layer::{surreal_item::SurrealUrgency, surreal_mode::SurrealMode}, node::{item_node::ItemNode, item_status::ItemStatus}};

pub(crate) struct Mode<'s> {
    surreal_mode: &'s SurrealMode,
}

pub(crate) enum ModeCategory {
    Core,
    NonCore,
    OutOfScope,
    NotDeclared
}

impl<'s> Mode<'s> {
    pub(crate) fn new(surreal_mode: &'s SurrealMode) -> Self {
        Self { surreal_mode }
    }

    pub(crate) fn get_name(&self) -> &'s str {
        &self.surreal_mode.summary
    }

    pub(crate) fn get_parent(&self) -> &'s Option<Thing> {
        &self.surreal_mode.parent_mode
    }

    pub(crate) fn get_surreal_id(&self) -> &'s Thing {
        self.surreal_mode
            .id
            .as_ref()
            .expect("Comes from the database so this is always present")
    }

    pub(crate) fn get_category_by_importance<'a>(&self, item: &'a ItemNode<'a>) -> ModeCategory {
        self.get_category_and_reason_by_importance(item).0
    }

    pub(crate) fn get_category_by_urgency<'a>(&self, item: &'a ItemNode<'a>) -> ModeCategory {
        self.get_category_and_reason_by_urgency(item).0
    }

    pub(crate) fn get_category_and_reason_by_urgency<'a>(&self, item: &'a ItemStatus<'a>, minimum_urgency: Option<SurrealUrgency>) -> (ModeCategory, &'a ItemStatus<'a>) {
        item.get_self_and_parents(Filter::Active).iter().find_map(|x| x.get_urgency_now()
        todo!()
    }
}
