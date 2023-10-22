use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};
use surrealdb_extra::table::Table;

/// The purpose of this struct is to record Items that should be covered for a certain amount of time or until
/// an exact date_time
#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "coverings_until_datetime")]
pub(crate) struct SurrealCoveringUntilDatetime {
    pub(crate) id: Option<Thing>,
    pub(crate) cover_this: RecordId,
    pub(crate) until: Datetime,
}
