use crate::{
    base_data::mode::ModeCategory,
    data_storage::surrealdb_layer::surreal_current_mode::SurrealCurrentMode,
    node::{item_node::ItemNode, mode_node::ModeNode, Filter},
};

pub(crate) struct CurrentMode<'s> {
    mode: &'s ModeNode<'s>,
}

pub(crate) trait IsInTheMode<'t> {
    fn get_category_by_importance(&self, item_node: &'t ItemNode) -> ModeCategory<'t>;
    fn is_urgency_in_the_mode(&self, item_node: &ItemNode) -> bool;
}

impl<'a> IsInTheMode<'a> for &Option<CurrentMode<'a>> {
    fn get_category_by_importance(&self, item_node: &'a ItemNode) -> ModeCategory<'a> {
        match self {
            Some(current_mode) => current_mode.get_category_by_importance(item_node),
            None => ModeCategory::NonCore,
        }
    }

    fn is_urgency_in_the_mode(&self, item_node: &ItemNode) -> bool {
        match self {
            Some(current_mode) => current_mode.is_urgency_in_the_mode(item_node),
            None => true,
        }
    }
}

impl<'a> IsInTheMode<'a> for CurrentMode<'a> {
    fn get_category_by_importance(&self, item_node: &'a ItemNode<'a>) -> ModeCategory<'a> {
        self.mode.get_category_by_importance(item_node)
    }

    fn is_urgency_in_the_mode(&self, item_node: &ItemNode) -> bool {
        todo!()
    }
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

    pub(crate) fn get_category_by_importance<'a>(
        &self,
        item: &'a ItemNode<'a>,
    ) -> ModeCategory<'a> {
        self.mode.get_category_by_importance(item)
    }
}
