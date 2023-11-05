use crate::surrealdb_layer::surreal_covering::SurrealCovering;

use super::item::Item;

pub(crate) struct Covering<'a> {
    pub(crate) smaller: &'a Item<'a>,
    pub(crate) parent: &'a Item<'a>,
    pub(crate) _surreal_covering: &'a SurrealCovering,
}
