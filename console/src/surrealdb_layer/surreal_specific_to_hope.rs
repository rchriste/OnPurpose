use serde::{Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Thing};
use surrealdb_extra::table::Table;

use super::surreal_item::{Permanence, Staging};

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "specific_to_hopes")]
pub(crate) struct SurrealSpecificToHope {
    pub(crate) id: Option<Thing>,
    pub(crate) for_item: RecordId,
    pub(crate) permanence: Permanence,
    pub(crate) staging: Staging,
}

pub(crate) trait SurrealSpecificToHopes<'a> {
    fn get_by_id(&'a self, id: &RecordId) -> Option<&'a SurrealSpecificToHope>;
}

impl<'a> SurrealSpecificToHopes<'a> for &'a [SurrealSpecificToHope] {
    fn get_by_id(&'a self, id: &RecordId) -> Option<&'a SurrealSpecificToHope> {
        self.iter().find(|x| &x.for_item == id)
    }
}
