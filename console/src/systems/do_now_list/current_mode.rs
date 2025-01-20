use crate::{
    data_storage::surrealdb_layer::surreal_current_mode::SurrealCurrentMode,
    node::{item_node::ItemNode, mode_node::ModeNode, Filter},
};

pub(crate) struct CurrentMode<'s> {
    mode: &'s ModeNode<'s>,
}

impl<'s> CurrentMode<'s> {
    pub(crate) fn new(
        surreal_current_mode: &SurrealCurrentMode,
        mode_nodes: &'s [ModeNode<'s>],
    ) -> Self {
        let mode = mode_nodes
            .iter()
            .find(|mode| {
                mode.get_surreal_id()
                    == surreal_current_mode.mode.as_ref().expect("Mode must exist")
            })
            .expect("Mode must exist");

        CurrentMode { mode }
    }

    pub(crate) fn get_mode(&self) -> &ModeNode {
        self.mode
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
        SelectedSingleMode::AllCoreMotivationalPurposes => {
            item_node.is_core_work_or_neither(Filter::Active)
        }
        SelectedSingleMode::AllNonCoreMotivationalPurposes => {
            item_node.is_non_core_work_or_neither(Filter::Active)
        }
    })
}
