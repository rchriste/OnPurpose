use std::fmt::{Display, Formatter};

use crate::{
    display::display_item_status::DisplayItemStatus,
    node::{action_with_item_status::ActionWithItemStatus, Filter},
};

use super::display_item_node::DisplayFormat;

#[derive(Clone)]
pub(crate) struct DisplayActionWithItemStatus<'s> {
    item: &'s ActionWithItemStatus<'s>,
    filter: Filter,
    display_format: DisplayFormat,
}

impl Display for DisplayActionWithItemStatus<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.item {
            ActionWithItemStatus::MakeProgress(item_status) => {
                let display = DisplayItemStatus::new(item_status, self.filter, self.display_format);
                write!(f, "[🏃] {}", display)
            }
            ActionWithItemStatus::ParentBackToAMotivation(item_status) => {
                let display = DisplayItemStatus::new(item_status, self.filter, self.display_format);
                write!(f, "[🌟 Needs a reason] {}", display)
            }
            ActionWithItemStatus::PickItemReviewFrequency(item_status) => {
                let display = DisplayItemStatus::new(item_status, self.filter, self.display_format);
                write!(f, "[🔁 State review frequency] {}", display)
            }
            ActionWithItemStatus::ItemNeedsAClassification(item_status) => {
                let display = DisplayItemStatus::new(item_status, self.filter, self.display_format);
                write!(f, "[🗂️ Needs classification] {}", display)
            }
            ActionWithItemStatus::ReviewItem(item_status) => {
                let display = DisplayItemStatus::new(item_status, self.filter, self.display_format);
                write!(f, "[🔍 Review] {}", display)
            }
            ActionWithItemStatus::SetReadyAndUrgency(item_status) => {
                let display = DisplayItemStatus::new(item_status, self.filter, self.display_format);
                write!(f, "[🚦 Set readiness and urgency] {}", display)
            }
        }
    }
}

impl<'s> DisplayActionWithItemStatus<'s> {
    pub(crate) fn new(
        item: &'s ActionWithItemStatus<'s>,
        filter: Filter,
        display_format: DisplayFormat,
    ) -> Self {
        Self {
            item,
            filter,
            display_format,
        }
    }
}
