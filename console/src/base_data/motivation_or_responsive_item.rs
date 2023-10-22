use super::{motivation::Motivation, responsive_item::ResponsiveItem};

//TODO: I can probably remove this file and type. I believe this is dead code
pub(crate) enum MotivationOrResponsiveItem<'e> {
    Motivation(Motivation<'e>),
    ResponsiveItem(ResponsiveItem<'e>),
}