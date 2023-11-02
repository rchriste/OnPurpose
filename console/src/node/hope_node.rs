use crate::{
    base_data::{hope::Hope, item::Item},
    surrealdb_layer::surreal_item::SurrealItem,
};

pub(crate) struct HopeNode<'a> {
    pub(crate) hope: &'a Hope<'a>,
    pub(crate) next_steps: Vec<&'a Item<'a>>,
    pub(crate) towards_motivation_chain: Vec<&'a Item<'a>>,
}

impl<'a> From<&'a HopeNode<'a>> for &'a Hope<'a> {
    fn from(value: &HopeNode<'a>) -> Self {
        value.hope
    }
}

impl<'a> From<&'a HopeNode<'a>> for &'a SurrealItem {
    fn from(value: &'a HopeNode<'a>) -> Self {
        value.hope.into()
    }
}

impl<'a> HopeNode<'a> {
    pub(crate) fn is_maintenance(&self) -> bool {
        self.hope.is_maintenance()
    }

    pub(crate) fn is_project(&self) -> bool {
        self.hope.is_project()
    }
}
