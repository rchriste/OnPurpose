pub(crate) mod circumstance;
pub(crate) mod covering;
pub(crate) mod covering_until_date_time;
pub(crate) mod hope;
pub(crate) mod item;
pub(crate) mod life_area;
pub(crate) mod motivation;
pub(crate) mod motivation_or_responsive_item;
pub(crate) mod responsive_item;
pub(crate) mod routine;
pub(crate) mod to_do;

use ouroboros::self_referencing;

use crate::surrealdb_layer::surreal_tables::SurrealTables;

use self::{
    covering::Covering,
    covering_until_date_time::CoveringUntilDateTime,
    hope::Hope,
    item::{Item, ItemVecExtensions},
    life_area::LifeArea,
    routine::Routine,
};

#[self_referencing]
pub(crate) struct BaseData {
    surreal_tables: SurrealTables,

    #[borrows(surreal_tables)]
    #[covariant]
    items: Vec<Item<'this>>,

    #[borrows(items)]
    #[covariant]
    active_items: Vec<&'this Item<'this>>,

    #[borrows(items, surreal_tables)]
    #[covariant]
    just_hopes: Vec<Hope<'this>>,

    #[borrows(items, surreal_tables)]
    #[covariant]
    coverings: Vec<Covering<'this>>,

    #[borrows(items, surreal_tables)]
    #[covariant]
    coverings_until_date_time: Vec<CoveringUntilDateTime<'this>>,

    #[borrows(surreal_tables)]
    #[covariant]
    life_areas: Vec<LifeArea<'this>>,

    #[borrows(surreal_tables)]
    #[covariant]
    routines: Vec<Routine<'this>>,
}

impl BaseData {
    pub(crate) fn new_from_surreal_tables(surreal_tables: SurrealTables) -> Self {
        BaseDataBuilder {
            surreal_tables,
            items_builder: |surreal_tables| surreal_tables.make_items(),
            active_items_builder: |items| items.filter_active_items(),
            just_hopes_builder: |items, surreal_tables| {
                items.filter_just_hopes(&surreal_tables.surreal_specific_to_hopes)
            },
            coverings_builder: |items, surreal_tables| surreal_tables.make_coverings(items),
            coverings_until_date_time_builder: |items, surreal_tables| {
                surreal_tables.make_coverings_until_date_time(items)
            },
            life_areas_builder: |surreal_tables| surreal_tables.make_life_areas(),
            routines_builder: |surreal_tables| surreal_tables.make_routines(),
        }
        .build()
    }

    pub(crate) fn get_items(&self) -> &[Item] {
        self.borrow_items()
    }

    pub(crate) fn get_active_items(&self) -> &[&Item] {
        self.borrow_active_items()
    }

    pub(crate) fn get_just_hopes(&self) -> &[Hope] {
        self.borrow_just_hopes()
    }

    pub(crate) fn get_coverings(&self) -> &[Covering] {
        self.borrow_coverings()
    }

    pub(crate) fn get_coverings_until_date_time(&self) -> &[CoveringUntilDateTime] {
        self.borrow_coverings_until_date_time()
    }

    pub(crate) fn get_life_areas(&self) -> &[LifeArea] {
        self.borrow_life_areas()
    }

    pub(crate) fn get_routines(&self) -> &[Routine] {
        self.borrow_routines()
    }
}
