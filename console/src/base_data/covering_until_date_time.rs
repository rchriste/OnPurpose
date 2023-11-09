use chrono::{DateTime, Local};

use super::item::Item;

#[derive(Debug)]
pub(crate) struct CoveringUntilDateTime<'a> {
    pub(crate) cover_this: &'a Item<'a>,
    pub(crate) until: DateTime<Local>,
}
