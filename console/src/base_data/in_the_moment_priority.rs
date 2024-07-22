use surrealdb::sql::Datetime;

use crate::{
    node::{
        item_action::ActionWithItemStatus,
        item_node::{ItemNode, TriggerWithItem},
        item_status::{ItemStatus, TriggerWithItemNode},
        IsTriggered,
    },
    surrealdb_layer::surreal_in_the_moment_priority::{
        SurrealInTheMomentPriority, SurrealPriorityKind,
    },
};

use super::{item::Item, time_spent::TimeSpent};

pub(crate) struct InTheMomentPriorityWithItemAction<'s> {
    surreal_in_the_moment_priority: &'s SurrealInTheMomentPriority,
    in_effect_until: Vec<TriggerWithItemNode<'s>>,
    choice: ActionWithItemStatus<'s>,
    not_chosen: Vec<ActionWithItemStatus<'s>>,
}

impl<'s> InTheMomentPriorityWithItemAction<'s> {
    pub(crate) fn new(
        surreal_in_the_moment_priority: &'s SurrealInTheMomentPriority,
        now_sql: &Datetime,
        all_items: &'s [Item<'s>],
        all_nodes: &'s [ItemNode<'s>],
        items_status: &'s [ItemStatus<'s>],
        time_spent_log: &[TimeSpent<'_>],
    ) -> InTheMomentPriorityWithItemAction<'s> {
        let in_effect_until = surreal_in_the_moment_priority
            .in_effect_until
            .iter()
            .map(|trigger| {
                let trigger = TriggerWithItem::new(trigger, now_sql, all_items, time_spent_log);
                TriggerWithItemNode::new(&trigger, all_nodes)
            })
            .collect();
        let choice = ActionWithItemStatus::from_surreal_action(
            &surreal_in_the_moment_priority.choice,
            items_status,
        );
        let not_chosen = surreal_in_the_moment_priority
            .not_chosen
            .iter()
            .map(|action| ActionWithItemStatus::from_surreal_action(action, items_status))
            .collect();
        InTheMomentPriorityWithItemAction {
            surreal_in_the_moment_priority,
            in_effect_until,
            choice,
            not_chosen,
        }
    }

    pub(crate) fn get_choice(&self) -> &ActionWithItemStatus {
        &self.choice
    }

    pub(crate) fn get_kind(&self) -> &SurrealPriorityKind {
        &self.surreal_in_the_moment_priority.kind
    }

    pub(crate) fn in_not_chosen(&self, search_for: &ActionWithItemStatus) -> bool {
        self.not_chosen.contains(search_for)
    }

    pub(crate) fn in_not_chosen_any(&self, search_for: &[&ActionWithItemStatus]) -> bool {
        search_for
            .iter()
            .any(|item_action| self.in_not_chosen(item_action))
    }

    pub(crate) fn is_active(&self) -> bool {
        !self.in_effect_until.iter().any(|x| x.is_triggered())
    }
}
