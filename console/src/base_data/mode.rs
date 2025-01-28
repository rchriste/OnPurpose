use ahash::HashMap;
use surrealdb::{opt::RecordId, sql::Thing};

use crate::{
    data_storage::surrealdb_layer::{
        surreal_item::{SurrealModeScope, SurrealUrgency},
        surreal_mode::SurrealMode,
    },
    node::{item_node::ItemNode, item_status::ItemStatus, Filter},
};

pub(crate) struct Mode<'s> {
    surreal_mode: &'s SurrealMode,
}

#[derive(PartialEq, Eq)]
pub(crate) enum ModeCategory<'e> {
    Core,
    NonCore,
    OutOfScope,
    NotDeclared { item_to_specify: &'e RecordId },
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

    pub(crate) fn get_surreal(&self) -> &'s SurrealMode {
        self.surreal_mode
    }

    pub(crate) fn get_category_by_importance<'a>(&self, item: &'a ItemNode<'a>) -> ModeCategory {
        todo!()
    }

    pub(crate) fn get_category_by_urgency<'a>(
        &self,
        item: &'a ItemStatus<'a>,
        minimum_urgency: Option<SurrealUrgency>,
        all_item_nodes: &'a HashMap<&RecordId, ItemNode>,
    ) -> ModeCategory<'a> {
        let matches = item.get_self_and_parents_flattened(Filter::Active).iter().filter_map(|x| {
            let item_node = all_item_nodes.get(x.get_surreal_record_id()).expect("Item status must exist");
            match item_node.get_urgency_now() {
                None | Some(None) => None,
                Some(Some(urgency)) => {
                    let scope = urgency.get_scope();
                    match scope {
                        SurrealModeScope::AllModes => Some(ModeCategory::NonCore),
                        SurrealModeScope::DefaultModesWithChanges { extra_modes_included } => todo!("Need to check default modes, and extra modes, and if extra_modes_included causes it to get pulled in"),
                    }
                }
            }}).collect::<Vec<_>>();
        if matches.iter().any(|x| x == &ModeCategory::Core) {
            ModeCategory::Core
        } else if matches.iter().any(|x| x == &ModeCategory::NonCore) {
            ModeCategory::NonCore
        } else if matches.iter().any(|x| x == &ModeCategory::OutOfScope) {
            ModeCategory::OutOfScope
        } else if let Some(item_to_specify) = matches.into_iter().find_map(|x| match x {
            ModeCategory::NotDeclared { item_to_specify } => Some(item_to_specify),
            _ => None,
        }) {
            ModeCategory::NotDeclared {
                item_to_specify: item.get_surreal_record_id(),
            }
        } else {
            panic!("This should not happen, because we are getting self and parents so there should always be a ModeCategory match")
        }
    }
}
