use serde::{Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Thing};
use surrealdb_extra::table::Table;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "specific_to_hopes")]
pub(crate) struct SurrealSpecificToHope {
    pub(crate) id: Option<Thing>,
    pub(crate) for_item: RecordId,
    pub(crate) permanence: Permanence,
    pub(crate) staging: Staging,
}

impl SurrealSpecificToHope {
    pub(crate) fn new_defaults(for_item: RecordId) -> Self {
        Self {
            id: Option::<Thing>::default(),
            for_item,
            permanence: Permanence::default(),
            staging: Staging::default(),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum Permanence {
    Maintenance,
    #[default]
    Project,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum Staging {
    #[default]
    MentallyResident,
    OnDeck,
    Intension,
    Released,
}

pub(crate) trait SurrealSpecificToHopes<'a> {
    fn get_by_id(&'a self, id: &RecordId) -> Option<&'a SurrealSpecificToHope>;
}

impl<'a> SurrealSpecificToHopes<'a> for &'a [SurrealSpecificToHope] {
    fn get_by_id(&'a self, id: &RecordId) -> Option<&'a SurrealSpecificToHope> {
        self.iter().find(|x| &x.for_item == id)
    }
}
