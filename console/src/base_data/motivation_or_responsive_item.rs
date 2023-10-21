use crate::surrealdb_layer::surreal_item::SurrealItem;

use super::{motivation::Motivation, responsive_item::ResponsiveItem};

//TODO: I can probably remove this file and type. I believe this is dead code
pub enum MotivationOrResponsiveItem<'e> {
    Motivation(Motivation<'e>),
    ResponsiveItem(ResponsiveItem<'e>),
}

impl<'e> MotivationOrResponsiveItem<'e> {
    pub fn get_surreal_item(&self) -> &'e SurrealItem {
        match self {
            MotivationOrResponsiveItem::Motivation(motivation) => motivation.get_surreal_item(),
            MotivationOrResponsiveItem::ResponsiveItem(responsive_item) => {
                responsive_item.get_surreal_item()
            }
        }
    }
}
