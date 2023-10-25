use std::fmt::Display;

use crate::{
    base_data::motivation_or_responsive_item::MotivationOrResponsiveItem,
    display::{display_item::DisplayItem, display_responsive_item::DisplayResponsiveItem},
};

pub(crate) struct DisplayMotivationOrResponsiveItem<'e> {
    item: &'e MotivationOrResponsiveItem<'e>,
}

impl Display for DisplayMotivationOrResponsiveItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.item {
            MotivationOrResponsiveItem::Motivation(motivation) => {
                write!(f, "{}", DisplayItem::new(motivation.get_item()))
            }
            MotivationOrResponsiveItem::ResponsiveItem(responsive_item) => {
                write!(f, "{}", DisplayResponsiveItem::new(responsive_item))
            }
        }
    }
}

impl<'e> From<DisplayMotivationOrResponsiveItem<'e>> for &'e MotivationOrResponsiveItem<'e> {
    fn from(display_motivation_or_responsive_item: DisplayMotivationOrResponsiveItem<'e>) -> Self {
        display_motivation_or_responsive_item.item
    }
}
