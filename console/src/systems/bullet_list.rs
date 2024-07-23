use chrono::{DateTime, Utc};
use ouroboros::self_referencing;

use crate::{
    calculated_data::CalculatedData,
    node::{
        action_with_item_status::{ActionListsByUrgency, ActionWithItemStatus},
        item_status::ItemStatus,
        Filter,
    },
    surrealdb_layer::surreal_item::SurrealUrgency,
    systems::upcoming::Upcoming,
};

#[self_referencing]
pub(crate) struct BulletList {
    calculated_data: CalculatedData,

    #[borrows(calculated_data)]
    #[covariant]
    ordered_bullet_list: Vec<ActionWithItemStatus<'this>>,

    #[borrows(calculated_data)]
    #[covariant]
    upcoming: Upcoming<'this>,
}

impl BulletList {
    pub(crate) fn new_bullet_list(
        calculated_data: CalculatedData,
        current_time: &DateTime<Utc>,
    ) -> Self {
        BulletListBuilder {
            calculated_data,
            ordered_bullet_list_builder: |calculated_data| {
                //Get all top level items
                let everything_that_has_no_parent = calculated_data
                    .get_items_status()
                    .iter()
                    .filter(|x| !x.has_parents(Filter::Active) && x.is_active())
                    .collect::<Vec<_>>();

                let all_items_status = calculated_data.get_items_status();
                let most_important_items = everything_that_has_no_parent
                    .iter()
                    .filter_map(|x| x.recursive_get_most_important_and_ready(all_items_status))
                    .collect::<Vec<_>>();
                let urgent_items = everything_that_has_no_parent
                    .iter()
                    .flat_map(|x| {
                        x.recursive_get_urgent_bullet_list(all_items_status, Vec::default())
                    })
                    .collect::<Vec<_>>();

                let mut bullet_lists_by_urgency = ActionListsByUrgency::default();

                for item in most_important_items.into_iter() {
                    let item = ActionWithItemStatus::MakeProgress(item);
                    bullet_lists_by_urgency
                        .in_the_mode_maybe_urgent_and_by_importance
                        .push(item);
                }

                for item in urgent_items.into_iter() {
                    match item.get_urgency_now() {
                        SurrealUrgency::MoreUrgentThanAnythingIncludingScheduled => {
                            bullet_lists_by_urgency
                                .more_urgent_than_anything_including_scheduled
                                .push(item);
                        }
                        SurrealUrgency::ScheduledAnyMode(_) => {
                            bullet_lists_by_urgency.scheduled_any_mode.push(item);
                        }
                        SurrealUrgency::MoreUrgentThanMode => {
                            bullet_lists_by_urgency.more_urgent_than_mode.push(item);
                        }
                        SurrealUrgency::InTheModeScheduled(_) => {
                            bullet_lists_by_urgency.in_the_mode_scheduled.push(item);
                        }
                        SurrealUrgency::InTheModeDefinitelyUrgent => {
                            bullet_lists_by_urgency
                                .in_the_mode_definitely_urgent
                                .push(item);
                        }
                        SurrealUrgency::InTheModeMaybeUrgent
                        | SurrealUrgency::InTheModeByImportance => {
                            bullet_lists_by_urgency
                                .in_the_mode_maybe_urgent_and_by_importance
                                .push(item);
                        }
                    }
                }

                let all_priorities = calculated_data.get_in_the_moment_priorities();
                let ordered_bullet_list =
                    bullet_lists_by_urgency.apply_in_the_moment_priorities(all_priorities);

                ordered_bullet_list
            },
            upcoming_builder: |calculated_data| {
                let upcoming = Upcoming::new(calculated_data, current_time);
                upcoming
            },
        }
        .build()
    }

    pub(crate) fn get_ordered_bullet_list(&self) -> &[ActionWithItemStatus<'_>] {
        self.borrow_ordered_bullet_list()
    }

    pub(crate) fn get_all_items_status(&self) -> &[ItemStatus<'_>] {
        self.borrow_calculated_data().get_items_status()
    }

    pub(crate) fn get_upcoming(&self) -> &Upcoming {
        self.borrow_upcoming()
    }
}
