use std::fmt::{Display, Formatter};

use surrealdb::opt::RecordId;

use crate::{
    data_storage::surrealdb_layer::{
        surreal_in_the_moment_priority::SurrealAction, surreal_item::SurrealUrgency,
    },
    display::display_item_status::DisplayItemStatus,
    node::action_with_item_status::ActionWithItemStatus,
};

#[derive(Clone)]
pub(crate) struct DisplayActionWithItemStatus<'s> {
    item: &'s ActionWithItemStatus<'s>,
}

impl<'s> Display for DisplayActionWithItemStatus<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let urgency = self.get_urgency_now();
        match urgency {
            SurrealUrgency::MoreUrgentThanAnythingIncludingScheduled => write!(f, "🚨 ")?,
            SurrealUrgency::MoreUrgentThanMode => write!(f, "🔥 ")?,
            SurrealUrgency::InTheModeByImportance => write!(f, "🟢 ")?,
            SurrealUrgency::InTheModeDefinitelyUrgent => write!(f, "🔴 ")?,
            SurrealUrgency::InTheModeMaybeUrgent => write!(f, "🟡 ")?,
            SurrealUrgency::ScheduledAnyMode(..) => write!(f, "❗🗓️ ")?,
            SurrealUrgency::InTheModeScheduled(..) => write!(f, "⭳🗓️ ")?,
        }

        match self.item {
            ActionWithItemStatus::MakeProgress(item_status) => {
                let display = DisplayItemStatus::new(item_status);
                write!(f, "[🚶🏻] {}", display)
            }
            ActionWithItemStatus::ParentBackToAMotivation(item_status) => {
                let display = DisplayItemStatus::new(item_status);
                write!(f, "[👨‍👦 Needs a reason] {}", display)
            }
            ActionWithItemStatus::PickItemReviewFrequency(item_status) => {
                let display = DisplayItemStatus::new(item_status);
                write!(f, "[🔁 State review frequency] {}", display)
            }
            ActionWithItemStatus::PickWhatShouldBeDoneFirst(choices) => {
                write!(
                    f,
                    "[🔝 Pick what is highest priority] {} choices",
                    choices.len()
                )
            }
            ActionWithItemStatus::ReviewItem(item_status) => {
                let display = DisplayItemStatus::new(item_status);
                write!(f, "[🔍 Review] {}", display)
            }
            ActionWithItemStatus::SetReadyAndUrgency(item_status) => {
                let display = DisplayItemStatus::new(item_status);
                write!(f, "[🚦 Set readiness and urgency] {}", display)
            }
        }
    }
}

impl<'s> DisplayActionWithItemStatus<'s> {
    pub(crate) fn new(item: &'s ActionWithItemStatus<'s>) -> Self {
        Self { item }
    }

    pub(crate) fn get_surreal_record_id(&self) -> &RecordId {
        self.item.get_surreal_record_id()
    }

    pub(crate) fn get_urgency_now(&self) -> SurrealUrgency {
        self.item.get_urgency_now()
    }

    pub(crate) fn clone_to_surreal_action(&self) -> SurrealAction {
        self.item.clone_to_surreal_action()
    }

    pub(crate) fn get_action(&self) -> &ActionWithItemStatus<'s> {
        self.item
    }
}
