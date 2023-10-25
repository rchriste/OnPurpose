use std::fmt::Display;

use crate::base_data::responsive_item::ResponsiveItem;

pub(crate) struct DisplayResponsiveItem<'e> {
    item: &'e ResponsiveItem<'e>,
}

impl Display for DisplayResponsiveItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.item.get_item().get_summary())
    }
}

impl<'e> DisplayResponsiveItem<'e> {
    pub(crate) fn new(item: &'e ResponsiveItem<'e>) -> Self {
        Self { item }
    }
}
