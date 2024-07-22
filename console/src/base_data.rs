pub(crate) mod circumstance;
pub(crate) mod in_the_moment_priority;
pub(crate) mod item;
pub(crate) mod life_area;
pub(crate) mod routine;
pub(crate) mod time_spent;

use chrono::{DateTime, Utc};
use ouroboros::self_referencing;
use surrealdb::opt::RecordId;

use crate::surrealdb_layer::{
    surreal_in_the_moment_priority::SurrealInTheMomentPriority, surreal_tables::SurrealTables,
};

use self::{
    item::{Item, ItemVecExtensions},
    life_area::LifeArea,
    routine::Routine,
    time_spent::TimeSpent,
};

#[self_referencing]
pub(crate) struct BaseData {
    surreal_tables: SurrealTables,
    now: DateTime<Utc>,

    #[borrows(surreal_tables, now)]
    #[covariant]
    items: Vec<Item<'this>>,

    #[borrows(items)]
    #[covariant]
    active_items: Vec<&'this Item<'this>>,

    #[borrows(surreal_tables)]
    #[covariant]
    life_areas: Vec<LifeArea<'this>>,

    #[borrows(surreal_tables)]
    #[covariant]
    routines: Vec<Routine<'this>>,

    #[borrows(surreal_tables)]
    #[covariant]
    time_spent_log: Vec<TimeSpent<'this>>,
}

impl BaseData {
    pub(crate) fn new_from_surreal_tables(
        surreal_tables: SurrealTables,
        now: DateTime<Utc>,
    ) -> Self {
        BaseDataBuilder {
            surreal_tables,
            items_builder: |surreal_tables, now| surreal_tables.make_items(now),
            active_items_builder: |items| items.filter_active_items(),
            now,
            life_areas_builder: |surreal_tables| surreal_tables.make_life_areas(),
            routines_builder: |surreal_tables| surreal_tables.make_routines(),
            time_spent_log_builder: |surreal_tables| surreal_tables.make_time_spent_log().collect(),
        }
        .build()
    }

    pub(crate) fn get_now(&self) -> &DateTime<Utc> {
        self.borrow_now()
    }

    pub(crate) fn get_items(&self) -> &[Item] {
        self.borrow_items()
    }

    pub(crate) fn get_active_items(&self) -> &[&Item] {
        self.borrow_active_items()
    }

    pub(crate) fn get_life_areas(&self) -> &[LifeArea] {
        self.borrow_life_areas()
    }

    pub(crate) fn get_routines(&self) -> &[Routine] {
        self.borrow_routines()
    }

    pub(crate) fn get_time_spent_log(&self) -> &[TimeSpent] {
        self.borrow_time_spent_log()
    }

    pub(crate) fn get_surreal_in_the_moment_priorities(&self) -> &[SurrealInTheMomentPriority] {
        self.borrow_surreal_tables()
            .get_surreal_in_the_moment_priorities()
    }
}

pub(crate) trait FindRecordId<'t, T> {
    fn find_record_id(&self, record_id: &RecordId) -> Option<&'t T>;
}
