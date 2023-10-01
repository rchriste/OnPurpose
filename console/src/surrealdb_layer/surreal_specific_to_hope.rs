use serde::{Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Thing};
use surrealdb_extra::table::Table;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "specific_to_hopes")]
pub struct SurrealSpecificToHope {
    pub id: Option<Thing>,
    pub for_item: RecordId,
    pub permanence: Permanence,
    pub staging: Staging,
}

impl SurrealSpecificToHope {
    pub fn new_defaults(for_item: RecordId) -> Self {
        Self {
            id: Option::<Thing>::default(),
            for_item,
            permanence: Permanence::default(),
            staging: Staging::default(),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub enum Permanence {
    Maintenance,
    #[default]
    Project,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub enum Staging {
    #[default]
    MentallyResident,
    OnDeck,
    Released,
}

pub trait SurrealSpecificToHopes<'a> {
    fn get_by_id(&'a self, id: &RecordId) -> Option<&'a SurrealSpecificToHope>;
}

impl<'a> SurrealSpecificToHopes<'a> for &'a [SurrealSpecificToHope] {
    fn get_by_id(&'a self, id: &RecordId) -> Option<&'a SurrealSpecificToHope> {
        self.iter().find(|x| &x.for_item == id)
    }
}
