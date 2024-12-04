use derive_builder::Builder;
use surrealdb::sql::Thing;

#[derive(Builder)]
#[builder(setter(into))]
pub(crate) struct NewMode {
    pub(crate) name: String,

    #[builder(default)]
    pub(crate) parent: Option<Thing>,
}
