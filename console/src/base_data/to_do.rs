use chrono::{DateTime, Local};
use surrealdb::sql::{Datetime, Thing};

use crate::surrealdb_layer::surreal_item::SurrealItem;

use super::{item::Item, Covering, CoveringUntilDateTime};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ToDo<'a> {
    //TODO: turn these things into get methods and use that derive get thing
    pub id: &'a Thing,
    pub summary: &'a String,
    pub finished: &'a Option<Datetime>,
    item: &'a Item<'a>,
}

impl<'a> From<ToDo<'a>> for SurrealItem {
    fn from(value: ToDo<'a>) -> Self {
        value.item.surreal_item.clone()
    }
}

impl<'a> From<ToDo<'a>> for &'a Thing {
    fn from(value: ToDo<'a>) -> Self {
        value.id
    }
}

impl<'a> From<&ToDo<'a>> for &'a SurrealItem {
    fn from(value: &ToDo<'a>) -> Self {
        value.item.into()
    }
}

impl<'a> From<&ToDo<'a>> for &'a Item<'a> {
    fn from(value: &ToDo<'a>) -> Self {
        value.item
    }
}

impl<'a> From<ToDo<'a>> for Item<'a> {
    fn from(value: ToDo<'a>) -> Self {
        value.item.clone()
    }
}

impl<'a> PartialEq<Item<'a>> for ToDo<'a> {
    fn eq(&self, other: &Item<'a>) -> bool {
        self.item == other
    }
}

impl<'a> ToDo<'a> {
    pub(crate) fn new(item: &'a Item) -> Self {
        ToDo {
            id: item.id,
            summary: item.summary,
            finished: item.finished,
            item,
        }
    }
    pub fn is_covered_by_another_item(&self, coverings: &[Covering<'_>]) -> bool {
        self.item.is_covered_by_another_item(coverings)
    }

    pub fn is_covered_by_date_time(
        &self,
        coverings_until_date_time: &[CoveringUntilDateTime<'_>],
        now: &DateTime<Local>,
    ) -> bool {
        self.item
            .is_covered_by_date_time(coverings_until_date_time, now)
    }

    pub fn is_covered(
        &self,
        coverings: &[Covering<'_>],
        coverings_until_date_time: &[CoveringUntilDateTime<'_>],
        now: &DateTime<Local>,
    ) -> bool {
        self.item
            .is_covered(coverings, coverings_until_date_time, now)
    }

    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }

    pub fn is_circumstances_met(&self, date: &DateTime<Local>, are_we_in_focus_time: bool) -> bool {
        self.item.is_circumstances_met(date, are_we_in_focus_time)
    }

    pub fn get_item(&self) -> &'a Item<'a> {
        self.item
    }

    pub fn get_surreal_item(&self) -> &'a SurrealItem {
        self.item.surreal_item
    }

    pub fn get_summary(&self) -> &'a str {
        self.item.get_summary()
    }
}
