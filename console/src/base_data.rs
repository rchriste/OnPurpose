// use crate::surrealdb_layer::surreal_tables::SurrealTables;

// use self::{item::Item, covering_until_date_time::CoveringUntilDateTime, covering::Covering};

pub(crate) mod circumstance;
pub(crate) mod covering;
pub(crate) mod covering_until_date_time;
pub(crate) mod hope;
pub(crate) mod item;
pub(crate) mod life_area;
pub(crate) mod motivation;
pub(crate) mod motivation_or_responsive_item;
pub(crate) mod person_or_group;
pub(crate) mod responsive_item;
pub(crate) mod routine;
pub(crate) mod simple;
pub(crate) mod to_do;
pub(crate) mod undeclared;

// pub(crate) struct BaseData<'s> {
//     items: Vec<Item<'s>>,
//     coverings: Vec<Covering<'s>>,
//     coverings_until_date_time: Vec<CoveringUntilDateTime<'s>>,
//     surreal_tables: &'s SurrealTables
// }

// impl<'s> BaseData<'s> {
//     pub(crate) fn new(surreal_tables: &'s SurrealTables) -> Self {
//         //note that I would prefer to move SurrealTables into this struct,
//         let mut base_data = Self {
//             items: Vec::default(),
//             coverings: Vec::default(),
//             coverings_until_date_time: Vec::default(),
//             surreal_tables,
//         };
//         todo!()
//         // base_data.create_items();
//         // base_data.coverings = surreal_tables.make_coverings(&base_data.items);
//         // base_data.coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&base_data.items);

//         // base_data
//     }

//     fn create_items(&'static mut self) {
//         self.items = self.surreal_tables.make_items();
//     }
// }
