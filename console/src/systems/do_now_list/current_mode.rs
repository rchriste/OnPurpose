use crate::{
    data_storage::surrealdb_layer::surreal_current_mode::{
        SurrealCurrentMode, SurrealSelectedSingleMode,
    },
    node::{item_node::ItemNode, Filter},
};

pub(crate) struct CurrentMode {
    urgency_in_scope: Vec<SelectedSingleMode>,
    importance_in_scope: Vec<SelectedSingleMode>,
}

#[derive(PartialEq, Eq)]
pub(crate) enum SelectedSingleMode {
    AllCoreMotivationalPurposes,
    AllNonCoreMotivationalPurposes,
}

impl Default for CurrentMode {
    fn default() -> Self {
        //By default everything should be selected
        CurrentMode {
            urgency_in_scope: vec![
                SelectedSingleMode::AllCoreMotivationalPurposes,
                SelectedSingleMode::AllNonCoreMotivationalPurposes,
            ],
            importance_in_scope: vec![
                SelectedSingleMode::AllCoreMotivationalPurposes,
                SelectedSingleMode::AllNonCoreMotivationalPurposes,
            ],
        }
    }
}

impl SurrealSelectedSingleMode {
    pub(crate) fn copy_to_items_in_scope_with_item_nodes(&self) -> SelectedSingleMode {
        match self {
            SurrealSelectedSingleMode::AllCoreMotivationalPurposes => {
                SelectedSingleMode::AllCoreMotivationalPurposes
            }
            SurrealSelectedSingleMode::AllNonCoreMotivationalPurposes => {
                SelectedSingleMode::AllNonCoreMotivationalPurposes
            }
        }
    }
}

impl CurrentMode {
    pub(crate) fn new(surreal_current_mode: &SurrealCurrentMode) -> CurrentMode {
        let urgency_in_scope = surreal_current_mode
            .urgency_in_scope
            .iter()
            .map(|urgency| urgency.copy_to_items_in_scope_with_item_nodes())
            .collect::<Vec<_>>();
        let importance_in_scope = surreal_current_mode
            .importance_in_scope
            .iter()
            .map(|importance| importance.copy_to_items_in_scope_with_item_nodes())
            .collect::<Vec<_>>();

        CurrentMode {
            urgency_in_scope,
            importance_in_scope,
        }
    }

    pub(crate) fn is_urgency_in_the_mode(&self, item_node: &ItemNode) -> bool {
        is_in_scope(self.get_urgency_in_scope(), item_node)
    }

    pub(crate) fn is_importance_in_the_mode(&self, item_node: &ItemNode) -> bool {
        is_in_scope(self.get_importance_in_scope(), item_node)
    }

    pub(crate) fn get_urgency_in_scope(&self) -> &Vec<SelectedSingleMode> {
        &self.urgency_in_scope
    }

    pub(crate) fn get_importance_in_scope(&self) -> &Vec<SelectedSingleMode> {
        &self.importance_in_scope
    }
}

fn is_in_scope(in_scope: &[SelectedSingleMode], item_node: &ItemNode) -> bool {
    in_scope.iter().any(|x| match x {
        SelectedSingleMode::AllCoreMotivationalPurposes => item_node.is_core_work(Filter::Active),
        SelectedSingleMode::AllNonCoreMotivationalPurposes => {
            item_node.is_non_core_work(Filter::Active)
        }
    })
}
