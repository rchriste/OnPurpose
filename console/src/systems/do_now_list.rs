use chrono::{DateTime, Utc};
use ouroboros::self_referencing;

use crate::{
    calculated_data::CalculatedData,
    data_storage::surrealdb_layer::surreal_item::SurrealUrgency,
    node::{
        action_with_item_status::{ActionListsByUrgency, ActionWithItemStatus},
        item_status::ItemStatus,
        Filter,
    },
    systems::upcoming::Upcoming,
};

#[self_referencing]
pub(crate) struct DoNowList {
    calculated_data: CalculatedData,

    #[borrows(calculated_data)]
    #[covariant]
    ordered_do_now_list: Vec<ActionWithItemStatus<'this>>,

    #[borrows(calculated_data)]
    #[covariant]
    upcoming: Upcoming<'this>,
}

impl DoNowList {
    pub(crate) fn new_do_now_list(
        calculated_data: CalculatedData,
        current_time: &DateTime<Utc>,
    ) -> Self {
        DoNowListBuilder {
            calculated_data,
            ordered_do_now_list_builder: |calculated_data| {
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
                        .push_if_new(item);
                }

                for item in urgent_items.into_iter() {
                    match item.get_urgency_now() {
                        SurrealUrgency::MoreUrgentThanAnythingIncludingScheduled => {
                            bullet_lists_by_urgency
                                .more_urgent_than_anything_including_scheduled
                                .push_if_new(item);
                        }
                        SurrealUrgency::ScheduledAnyMode(_) => {
                            bullet_lists_by_urgency.scheduled_any_mode.push_if_new(item);
                        }
                        SurrealUrgency::MoreUrgentThanMode => {
                            bullet_lists_by_urgency
                                .more_urgent_than_mode
                                .push_if_new(item);
                        }
                        SurrealUrgency::InTheModeScheduled(_) => {
                            bullet_lists_by_urgency
                                .in_the_mode_scheduled
                                .push_if_new(item);
                        }
                        SurrealUrgency::InTheModeDefinitelyUrgent => {
                            bullet_lists_by_urgency
                                .in_the_mode_definitely_urgent
                                .push_if_new(item);
                        }
                        SurrealUrgency::InTheModeMaybeUrgent
                        | SurrealUrgency::InTheModeByImportance => {
                            bullet_lists_by_urgency
                                .in_the_mode_maybe_urgent_and_by_importance
                                .push_if_new(item);
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

    pub(crate) fn get_ordered_do_now_list(&self) -> &[ActionWithItemStatus<'_>] {
        self.borrow_ordered_do_now_list()
    }

    pub(crate) fn get_all_items_status(&self) -> &[ItemStatus<'_>] {
        self.borrow_calculated_data().get_items_status()
    }

    pub(crate) fn get_upcoming(&self) -> &Upcoming {
        self.borrow_upcoming()
    }

    pub(crate) fn get_now(&self) -> &DateTime<Utc> {
        self.borrow_calculated_data().get_now()
    }
}

trait PushIfNew<'t> {
    fn push_if_new(&mut self, item: ActionWithItemStatus<'t>);
}

impl<'t> PushIfNew<'t> for Vec<ActionWithItemStatus<'t>> {
    fn push_if_new(&mut self, item: ActionWithItemStatus<'t>) {
        match self
            .iter()
            .find(|x| x.get_surreal_record_id() == item.get_surreal_record_id())
        {
            None => {
                self.push(item);
            }
            Some(_) => {
                //Do nothing, Item is already there
            }
        }
    }
}
