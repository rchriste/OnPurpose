use crate::surrealdb_layer::{
    surreal_item::SurrealItem,
    surreal_required_circumstance::{CircumstanceType, SurrealRequiredCircumstance},
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct Circumstance<'a> {
    pub(crate) circumstance_for: &'a SurrealItem,
    pub(crate) circumstance_type: &'a CircumstanceType,
    surreal_required_circumstance: &'a SurrealRequiredCircumstance,
}

impl<'a> From<&Circumstance<'a>> for &'a SurrealRequiredCircumstance {
    fn from(value: &Circumstance<'a>) -> Self {
        value.surreal_required_circumstance
    }
}

impl<'a> From<Circumstance<'a>> for &'a SurrealRequiredCircumstance {
    fn from(value: Circumstance<'a>) -> Self {
        value.surreal_required_circumstance
    }
}
