use crate::surrealdb_layer::surreal_covering::SurrealCovering;

use super::item::Item;

pub(crate) struct Covering<'a> {
    pub(crate) smaller: &'a Item<'a>,
    pub(crate) parent: &'a Item<'a>,
    pub(crate) surreal_covering: &'a SurrealCovering,
}

impl<'a> Covering<'a> {
    pub(crate) fn get_surreal_covering(&'a self) -> &'a SurrealCovering {
        self.surreal_covering
    }
}
