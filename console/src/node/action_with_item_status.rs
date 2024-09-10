use surrealdb::opt::RecordId;

use crate::{
    base_data::{in_the_moment_priority::InTheMomentPriorityWithItemAction, FindRecordId},
    data_storage::surrealdb_layer::{
        surreal_in_the_moment_priority::{SurrealAction, SurrealPriorityKind},
        surreal_item::SurrealUrgency,
    },
};

use super::item_status::{ActionWithItemNode, ItemStatus};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ActionWithItemStatus<'e> {
    SetReadyAndUrgency(&'e ItemStatus<'e>),
    ParentBackToAMotivation(&'e ItemStatus<'e>),
    ReviewItem(&'e ItemStatus<'e>),
    PickItemReviewFrequency(&'e ItemStatus<'e>),
    PickWhatShouldBeDoneFirst(Vec<ActionWithItemStatus<'e>>),
    MakeProgress(&'e ItemStatus<'e>),
}

impl<'e> ActionWithItemStatus<'e> {
    pub(crate) fn new(action: &ActionWithItemNode<'_>, items_status: &'e [ItemStatus<'e>]) -> Self {
        match action {
            ActionWithItemNode::SetReadyAndUrgency(action) => {
                let item_status = items_status
                    .find_record_id(action.get_surreal_record_id())
                    .expect("All items are there");
                ActionWithItemStatus::SetReadyAndUrgency(item_status)
            }
            ActionWithItemNode::ParentBackToAMotivation(action) => {
                let item_status = items_status
                    .find_record_id(action.get_surreal_record_id())
                    .expect("All items are there");
                ActionWithItemStatus::ParentBackToAMotivation(item_status)
            }
            ActionWithItemNode::ReviewItem(action) => {
                let item_status = items_status
                    .find_record_id(action.get_surreal_record_id())
                    .expect("All items are there");
                ActionWithItemStatus::ReviewItem(item_status)
            }
            ActionWithItemNode::PickItemReviewFrequency(action) => {
                let item_status = items_status
                    .find_record_id(action.get_surreal_record_id())
                    .expect("All items are there");
                ActionWithItemStatus::PickItemReviewFrequency(item_status)
            }
            ActionWithItemNode::MakeProgress(action) => {
                let item_status = items_status
                    .find_record_id(action.get_surreal_record_id())
                    .expect("All items are there");
                ActionWithItemStatus::MakeProgress(item_status)
            }
        }
    }

    pub(crate) fn from_surreal_action(
        action: &SurrealAction,
        items_status: &'e [ItemStatus<'e>],
    ) -> Self {
        match action {
            SurrealAction::SetReadyAndUrgency(record_id) => {
                let item_status = items_status
                    .find_record_id(record_id)
                    .expect("All items are there");
                ActionWithItemStatus::SetReadyAndUrgency(item_status)
            }
            SurrealAction::ParentBackToAMotivation(record_id) => {
                let item_status = items_status
                    .find_record_id(record_id)
                    .expect("All items are there");
                ActionWithItemStatus::ParentBackToAMotivation(item_status)
            }
            SurrealAction::ReviewItem(record_id) => {
                let item_status = items_status
                    .find_record_id(record_id)
                    .expect("All items are there");
                ActionWithItemStatus::ReviewItem(item_status)
            }
            SurrealAction::PickItemReviewFrequency(record_id) => {
                let item_status = items_status
                    .find_record_id(record_id)
                    .expect("All items are there");
                ActionWithItemStatus::PickItemReviewFrequency(item_status)
            }
            SurrealAction::MakeProgress(record_id) => {
                let item_status = items_status
                    .find_record_id(record_id)
                    .expect("All items are there");
                ActionWithItemStatus::MakeProgress(item_status)
            }
        }
    }

    pub(crate) fn clone_to_surreal_action(&self) -> SurrealAction {
        match self {
            ActionWithItemStatus::MakeProgress(item) => SurrealAction::MakeProgress(item.get_surreal_record_id().clone()),
            ActionWithItemStatus::ParentBackToAMotivation(item) => SurrealAction::ParentBackToAMotivation(item.get_surreal_record_id().clone()),
            ActionWithItemStatus::PickItemReviewFrequency(item) => SurrealAction::PickItemReviewFrequency(item.get_surreal_record_id().clone()),
            ActionWithItemStatus::PickWhatShouldBeDoneFirst(_) => todo!("It is not valid to call get_surreal_action in this scenario. If that is desired then we need to work out what to do."),
            ActionWithItemStatus::ReviewItem(item) => SurrealAction::ReviewItem(item.get_surreal_record_id().clone()),
            ActionWithItemStatus::SetReadyAndUrgency(item) => SurrealAction::SetReadyAndUrgency(item.get_surreal_record_id().clone()),
        }
    }

    pub(crate) fn get_surreal_record_id(&self) -> &RecordId {
        match self {
            ActionWithItemStatus::SetReadyAndUrgency(item)
            | ActionWithItemStatus::ParentBackToAMotivation(item)
            | ActionWithItemStatus::ReviewItem(item)
            | ActionWithItemStatus::PickItemReviewFrequency(item)
            | ActionWithItemStatus::MakeProgress(item) => item.get_surreal_record_id(),
            ActionWithItemStatus::PickWhatShouldBeDoneFirst(_) => {
                todo!("It is not valid to call this function on this item")
            }
        }
    }

    pub(crate) fn get_urgency_now(&self) -> SurrealUrgency {
        match self {
            ActionWithItemStatus::MakeProgress(item_status, ..) => item_status
                .get_urgency_now()
                .unwrap_or(&SurrealUrgency::InTheModeByImportance)
                .clone(),
            ActionWithItemStatus::ParentBackToAMotivation(..) => SurrealUrgency::MoreUrgentThanMode,
            ActionWithItemStatus::PickItemReviewFrequency(..) => {
                SurrealUrgency::InTheModeMaybeUrgent
            }
            ActionWithItemStatus::PickWhatShouldBeDoneFirst(choices) => {
                let urgency = choices
                    .iter()
                    .map(|x| x.get_urgency_now())
                    .max()
                    .expect("It is not valid for choices to be empty");
                urgency
            }
            ActionWithItemStatus::ReviewItem(..) => SurrealUrgency::InTheModeMaybeUrgent,
            ActionWithItemStatus::SetReadyAndUrgency(..) => {
                SurrealUrgency::InTheModeDefinitelyUrgent
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct ActionListsByUrgency<'s> {
    pub(crate) more_urgent_than_anything_including_scheduled: Vec<ActionWithItemStatus<'s>>,
    pub(crate) scheduled_any_mode: Vec<ActionWithItemStatus<'s>>,
    pub(crate) more_urgent_than_mode: Vec<ActionWithItemStatus<'s>>,
    pub(crate) in_the_mode_scheduled: Vec<ActionWithItemStatus<'s>>,
    pub(crate) in_the_mode_definitely_urgent: Vec<ActionWithItemStatus<'s>>,
    pub(crate) in_the_mode_maybe_urgent_and_by_importance: Vec<ActionWithItemStatus<'s>>,
}

impl<'s> ActionListsByUrgency<'s> {
    pub(crate) fn apply_in_the_moment_priorities(
        self,
        all_priorities: &'s [InTheMomentPriorityWithItemAction<'s>],
    ) -> Vec<ActionWithItemStatus<'s>> {
        let mut ordered_bullet_list = Vec::new();

        if let Some(more_urgent_than_anything_including_scheduled) = self
            .more_urgent_than_anything_including_scheduled
            .apply_in_the_moment_priorities(all_priorities)
        {
            ordered_bullet_list.push(more_urgent_than_anything_including_scheduled);
        }

        if let Some(scheduled_any_mode) = self
            .scheduled_any_mode
            .apply_in_the_moment_priorities(all_priorities)
        {
            ordered_bullet_list.push(scheduled_any_mode);
        }

        if let Some(more_urgent_than_mode) = self
            .more_urgent_than_mode
            .apply_in_the_moment_priorities(all_priorities)
        {
            ordered_bullet_list.push(more_urgent_than_mode);
        }

        if let Some(in_the_mode_scheduled) = self
            .in_the_mode_scheduled
            .apply_in_the_moment_priorities(all_priorities)
        {
            ordered_bullet_list.push(in_the_mode_scheduled);
        }

        if let Some(in_the_mode_definitely_urgent) = self
            .in_the_mode_definitely_urgent
            .apply_in_the_moment_priorities(all_priorities)
        {
            ordered_bullet_list.push(in_the_mode_definitely_urgent);
        }

        if let Some(in_the_mode_maybe_urgent_and_by_importance) = self
            .in_the_mode_maybe_urgent_and_by_importance
            .apply_in_the_moment_priorities(all_priorities)
        {
            ordered_bullet_list.push(in_the_mode_maybe_urgent_and_by_importance);
        }

        ordered_bullet_list
    }
}

trait ApplyInTheMomentPriorities<'s> {
    fn apply_in_the_moment_priorities(
        self,
        all_priorities: &'s [InTheMomentPriorityWithItemAction<'s>],
    ) -> Option<ActionWithItemStatus<'s>>;
}

impl<'s> ApplyInTheMomentPriorities<'s> for Vec<ActionWithItemStatus<'s>> {
    fn apply_in_the_moment_priorities(
        self,
        all_priorities: &'s [InTheMomentPriorityWithItemAction<'s>],
    ) -> Option<ActionWithItemStatus<'s>> {
        //Go through all ItemActions and look for that item in all_priorities and if found then if it is the highest priority then remove from the list all other items found and if it is the lowest priority then remove it if there are any higher priorities in the list.
        let all = self.iter().collect::<Vec<_>>();
        let lowest_priority_removed = self.iter().filter(|item_action| {
            let mut checked_something = false;
            //I need to go through everything that removes myself as the lowest priority and then go through the list a second time looking for the highest priority and removing all other items so this needs to be a two pass thing right now it is written as one pass
            let result = all_priorities
                .iter()
                .filter(|x| {
                    x.is_active()
                        && x.get_kind() == &SurrealPriorityKind::LowestPriority
                        && x.get_choice() == *item_action
                })
                .any(|lowest_priority| {
                    checked_something = true;
                    !lowest_priority.in_not_chosen_any(&all)
                });
            if checked_something {
                result
            } else {
                true
            }
        });

        let highest_priority_removed = lowest_priority_removed.filter(|item_action| {
            let mut checked_something = false;
            let result = all_priorities
                .iter()
                .filter(|x| {
                    x.is_active()
                        && x.get_kind() == &SurrealPriorityKind::HighestPriority
                        && x.in_not_chosen(item_action)
                })
                .all(|highest_priority_choice| {
                    checked_something = true;
                    !all.iter()
                        .any(|other| highest_priority_choice.get_choice() == *other)
                });
            if checked_something {
                result
            } else {
                true
            }
        });
        let choices = highest_priority_removed.cloned().collect::<Vec<_>>();
        if self.len() > 1 {
            assert!(!choices.is_empty(), "I am not expecting that it will ever be possible to remove all choices if so then this should be debugged");
        }

        if choices.is_empty() {
            None
        } else if choices.len() == 1 {
            Some(
                choices
                    .into_iter()
                    .next()
                    .expect("Size is checked to be at least 1"),
            )
        } else {
            Some(ActionWithItemStatus::PickWhatShouldBeDoneFirst(choices))
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::{
        base_data::BaseData,
        calculated_data,
        data_storage::surrealdb_layer::{
            surreal_in_the_moment_priority::{
                SurrealAction, SurrealInTheMomentPriorityBuilder, SurrealPriorityKind,
            },
            surreal_item::SurrealItemBuilder,
            surreal_tables::SurrealTablesBuilder,
            SurrealTrigger,
        },
        node::action_with_item_status::{ActionWithItemStatus, ApplyInTheMomentPriorities},
    };

    #[test]
    fn apply_in_the_moment_priorities_when_only_one_item_is_given_with_no_in_the_moment_priorities_then_that_one_item_is_returned(
    ) {
        let only_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("Only item")
            .build()
            .unwrap();

        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![only_item.clone()])
            .build()
            .unwrap();

        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let calculated_data = calculated_data::CalculatedData::new_from_base_data(base_data);

        let item_status = calculated_data.get_items_status().first().unwrap();
        let item_action = ActionWithItemStatus::MakeProgress(item_status);

        let blank_in_the_moment_priorities = calculated_data.get_in_the_moment_priorities();

        let dut = vec![item_action];
        let result = dut.apply_in_the_moment_priorities(blank_in_the_moment_priorities);

        assert!(result.is_some());
        let result = result.expect("assert.is_some() should have passed");
        assert_eq!(
            result.clone_to_surreal_action().get_record_id(),
            &only_item.id.unwrap()
        );
        assert_eq!(result, ActionWithItemStatus::MakeProgress(item_status));
    }

    #[test]
    fn apply_in_the_moment_priorities_when_only_two_items_are_given_pick_between_them_is_returned()
    {
        let first_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("First item")
            .build()
            .unwrap();
        let second_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Second item")
            .build()
            .unwrap();

        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![first_item.clone(), second_item.clone()])
            .build()
            .unwrap();

        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let calculated_data = calculated_data::CalculatedData::new_from_base_data(base_data);
        let items_status = calculated_data.get_items_status();

        let first_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == first_item.id.as_ref().unwrap())
            .expect("First item status not found");
        let second_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == second_item.id.as_ref().unwrap())
            .expect("Second item status not found");

        let first_item_action = ActionWithItemStatus::MakeProgress(first_item_status);
        let second_item_action = ActionWithItemStatus::MakeProgress(second_item_status);

        let blank_in_the_moment_priorities = calculated_data.get_in_the_moment_priorities();

        let dut = vec![first_item_action.clone(), second_item_action.clone()];
        let result = dut.apply_in_the_moment_priorities(blank_in_the_moment_priorities);

        assert!(result.is_some());
        let result = result.expect("assert.is_some() should have passed");
        assert_eq!(
            result,
            ActionWithItemStatus::PickWhatShouldBeDoneFirst(vec![
                first_item_action,
                second_item_action,
            ])
        );
    }

    #[test]
    fn apply_in_the_moment_priorities_when_two_items_are_given_and_one_is_the_highest_in_the_moment_priority_that_one_is_returned(
    ) {
        let first_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("First item")
            .build()
            .unwrap();
        let second_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Second item")
            .build()
            .unwrap();

        let in_an_hour = Utc::now() + chrono::Duration::hours(1);
        let in_the_moment_priority = SurrealInTheMomentPriorityBuilder::default()
            .id(Some(("surreal_in_the_moment_priority", "1").into()))
            .kind(SurrealPriorityKind::HighestPriority)
            .choice(SurrealAction::MakeProgress(
                first_item.id.clone().expect("hard coded to a value"),
            ))
            .not_chosen(vec![SurrealAction::MakeProgress(
                second_item.id.clone().expect("hard coded to a value"),
            )])
            .in_effect_until(vec![SurrealTrigger::WallClockDateTime(in_an_hour.into())])
            .build()
            .unwrap();

        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![first_item.clone(), second_item.clone()])
            .surreal_in_the_moment_priorities(vec![in_the_moment_priority])
            .build()
            .unwrap();

        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let calculated_data = calculated_data::CalculatedData::new_from_base_data(base_data);
        let items_status = calculated_data.get_items_status();

        let first_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == first_item.id.as_ref().unwrap())
            .expect("First item status not found");
        let second_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == second_item.id.as_ref().unwrap())
            .expect("Second item status not found");

        let first_item_action = ActionWithItemStatus::MakeProgress(first_item_status);
        let second_item_action = ActionWithItemStatus::MakeProgress(second_item_status);

        let blank_in_the_moment_priorities = calculated_data.get_in_the_moment_priorities();

        let dut = vec![first_item_action.clone(), second_item_action.clone()];
        let result = dut.apply_in_the_moment_priorities(blank_in_the_moment_priorities);

        assert!(result.is_some());
        let result = result.expect("assert.is_some() should have passed");
        assert_eq!(result, first_item_action,);
    }

    #[test]
    fn apply_in_the_moment_priorities_when_two_items_are_given_and_one_is_the_lowest_in_the_moment_priority_the_other_one_is_returned(
    ) {
        let first_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("First item")
            .build()
            .unwrap();
        let second_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Second item")
            .build()
            .unwrap();

        let in_an_hour = Utc::now() + chrono::Duration::hours(1);
        let in_the_moment_priority = SurrealInTheMomentPriorityBuilder::default()
            .id(Some(("surreal_in_the_moment_priority", "1").into()))
            .kind(SurrealPriorityKind::LowestPriority)
            .choice(SurrealAction::MakeProgress(
                first_item.id.clone().expect("hard coded to a value"),
            ))
            .not_chosen(vec![SurrealAction::MakeProgress(
                second_item.id.clone().expect("hard coded to a value"),
            )])
            .in_effect_until(vec![SurrealTrigger::WallClockDateTime(in_an_hour.into())])
            .build()
            .unwrap();

        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![first_item.clone(), second_item.clone()])
            .surreal_in_the_moment_priorities(vec![in_the_moment_priority])
            .build()
            .unwrap();

        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let calculated_data = calculated_data::CalculatedData::new_from_base_data(base_data);
        let items_status = calculated_data.get_items_status();

        let first_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == first_item.id.as_ref().unwrap())
            .expect("First item status not found");
        let second_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == second_item.id.as_ref().unwrap())
            .expect("Second item status not found");

        let first_item_action = ActionWithItemStatus::MakeProgress(first_item_status);
        let second_item_action = ActionWithItemStatus::MakeProgress(second_item_status);

        let blank_in_the_moment_priorities = calculated_data.get_in_the_moment_priorities();

        let dut = vec![first_item_action.clone(), second_item_action.clone()];
        let result = dut.apply_in_the_moment_priorities(blank_in_the_moment_priorities);

        assert!(result.is_some());
        let result = result.expect("assert.is_some() should have passed");
        assert_eq!(result, second_item_action,);
    }

    #[test]
    fn apply_in_the_moment_priorities_when_three_items_are_given_and_one_is_the_highest_in_the_moment_priority_over_one_other_item_then_the_other_two_are_returned_to_pick_between(
    ) {
        let first_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("First item")
            .build()
            .unwrap();
        let second_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Second item")
            .build()
            .unwrap();
        let third_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "3").into()))
            .summary("Third item")
            .build()
            .unwrap();

        let in_an_hour = Utc::now() + chrono::Duration::hours(1);
        let in_the_moment_priority = SurrealInTheMomentPriorityBuilder::default()
            .id(Some(("surreal_in_the_moment_priority", "1").into()))
            .kind(SurrealPriorityKind::HighestPriority)
            .choice(SurrealAction::MakeProgress(
                first_item.id.clone().expect("hard coded to a value"),
            ))
            .not_chosen(vec![SurrealAction::MakeProgress(
                second_item.id.clone().expect("hard coded to a value"),
            )])
            .in_effect_until(vec![SurrealTrigger::WallClockDateTime(in_an_hour.into())])
            .build()
            .unwrap();

        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![
                first_item.clone(),
                second_item.clone(),
                third_item.clone(),
            ])
            .surreal_in_the_moment_priorities(vec![in_the_moment_priority])
            .build()
            .unwrap();

        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let calculated_data = calculated_data::CalculatedData::new_from_base_data(base_data);
        let items_status = calculated_data.get_items_status();

        let first_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == first_item.id.as_ref().unwrap())
            .expect("First item status not found");
        let second_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == second_item.id.as_ref().unwrap())
            .expect("Second item status not found");
        let third_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == third_item.id.as_ref().unwrap())
            .expect("Third item status not found");

        let first_item_action = ActionWithItemStatus::MakeProgress(first_item_status);
        let second_item_action = ActionWithItemStatus::MakeProgress(second_item_status);
        let third_item_action = ActionWithItemStatus::MakeProgress(third_item_status);

        let in_the_moment_priorities = calculated_data.get_in_the_moment_priorities();

        let dut = vec![
            first_item_action.clone(),
            second_item_action.clone(),
            third_item_action.clone(),
        ];
        let result = dut.apply_in_the_moment_priorities(in_the_moment_priorities);

        assert!(result.is_some());
        let result = result.expect("assert.is_some() should have passed");
        assert_eq!(
            result,
            ActionWithItemStatus::PickWhatShouldBeDoneFirst(vec![
                first_item_action.clone(),
                third_item_action.clone()
            ]),
        );
    }

    #[test]
    fn apply_in_the_moment_priorities_when_three_items_are_given_and_one_is_the_lowest_in_the_moment_priority_over_one_other_item_then_the_other_two_are_returned_to_pick_between(
    ) {
        let first_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("First item")
            .build()
            .unwrap();
        let second_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Second item")
            .build()
            .unwrap();
        let third_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "3").into()))
            .summary("Third item")
            .build()
            .unwrap();

        let in_an_hour = Utc::now() + chrono::Duration::hours(1);
        let in_the_moment_priority = SurrealInTheMomentPriorityBuilder::default()
            .id(Some(("surreal_in_the_moment_priority", "1").into()))
            .kind(SurrealPriorityKind::LowestPriority)
            .choice(SurrealAction::MakeProgress(
                first_item.id.clone().expect("hard coded to a value"),
            ))
            .not_chosen(vec![SurrealAction::MakeProgress(
                second_item.id.clone().expect("hard coded to a value"),
            )])
            .in_effect_until(vec![SurrealTrigger::WallClockDateTime(in_an_hour.into())])
            .build()
            .unwrap();

        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![
                first_item.clone(),
                second_item.clone(),
                third_item.clone(),
            ])
            .surreal_in_the_moment_priorities(vec![in_the_moment_priority])
            .build()
            .unwrap();

        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let calculated_data = calculated_data::CalculatedData::new_from_base_data(base_data);
        let items_status = calculated_data.get_items_status();

        let first_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == first_item.id.as_ref().unwrap())
            .expect("First item status not found");
        let second_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == second_item.id.as_ref().unwrap())
            .expect("Second item status not found");
        let third_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == third_item.id.as_ref().unwrap())
            .expect("Third item status not found");

        let first_item_action = ActionWithItemStatus::MakeProgress(first_item_status);
        let second_item_action = ActionWithItemStatus::MakeProgress(second_item_status);
        let third_item_action = ActionWithItemStatus::MakeProgress(third_item_status);

        let in_the_moment_priorities = calculated_data.get_in_the_moment_priorities();

        let dut = vec![
            first_item_action.clone(),
            second_item_action.clone(),
            third_item_action.clone(),
        ];
        let result = dut.apply_in_the_moment_priorities(in_the_moment_priorities);

        assert!(result.is_some());
        let result = result.expect("assert.is_some() should have passed");
        assert_eq!(
            result,
            ActionWithItemStatus::PickWhatShouldBeDoneFirst(vec![
                second_item_action.clone(),
                third_item_action.clone()
            ]),
        );
    }

    #[test]
    fn apply_in_the_moment_priorities_when_three_items_are_given_and_one_is_the_highest_priority_over_one_other_and_the_third_is_the_lowest_priority_over_the_one_other_then_the_highest_priority_is_returned(
    ) {
        let first_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("First item")
            .build()
            .unwrap();
        let second_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Second item")
            .build()
            .unwrap();
        let third_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "3").into()))
            .summary("Third item")
            .build()
            .unwrap();

        let in_an_hour = Utc::now() + chrono::Duration::hours(1);
        let highest_in_the_moment_priority = SurrealInTheMomentPriorityBuilder::default()
            .id(Some(("surreal_in_the_moment_priority", "1").into()))
            .kind(SurrealPriorityKind::HighestPriority)
            .choice(SurrealAction::MakeProgress(
                first_item.id.clone().expect("hard coded to a value"),
            ))
            .not_chosen(vec![SurrealAction::MakeProgress(
                second_item.id.clone().expect("hard coded to a value"),
            )])
            .in_effect_until(vec![SurrealTrigger::WallClockDateTime(in_an_hour.into())])
            .build()
            .unwrap();
        let lowest_in_the_moment_priority = SurrealInTheMomentPriorityBuilder::default()
            .id(Some(("surreal_in_the_moment_priority", "3").into()))
            .kind(SurrealPriorityKind::LowestPriority)
            .choice(SurrealAction::MakeProgress(
                third_item.id.clone().expect("hard coded to a value"),
            ))
            .not_chosen(vec![SurrealAction::MakeProgress(
                first_item.id.clone().expect("hard coded to a value"),
            )])
            .in_effect_until(vec![SurrealTrigger::WallClockDateTime(in_an_hour.into())])
            .build()
            .unwrap();

        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![
                first_item.clone(),
                second_item.clone(),
                third_item.clone(),
            ])
            .surreal_in_the_moment_priorities(vec![
                highest_in_the_moment_priority,
                lowest_in_the_moment_priority,
            ])
            .build()
            .unwrap();

        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let calculated_data = calculated_data::CalculatedData::new_from_base_data(base_data);
        let items_status = calculated_data.get_items_status();

        let first_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == first_item.id.as_ref().unwrap())
            .expect("First item status not found");
        let second_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == second_item.id.as_ref().unwrap())
            .expect("Second item status not found");
        let third_item_status = items_status
            .iter()
            .find(|x| x.get_item().get_surreal_record_id() == third_item.id.as_ref().unwrap())
            .expect("Third item status not found");

        let first_item_action = ActionWithItemStatus::MakeProgress(first_item_status);
        let second_item_action = ActionWithItemStatus::MakeProgress(second_item_status);
        let third_item_action = ActionWithItemStatus::MakeProgress(third_item_status);

        let in_the_moment_priorities = calculated_data.get_in_the_moment_priorities();

        let dut = vec![
            first_item_action.clone(),
            second_item_action.clone(),
            third_item_action.clone(),
        ];
        let result = dut.apply_in_the_moment_priorities(in_the_moment_priorities);

        assert!(result.is_some());
        let result = result.expect("assert.is_some() should have passed");
        assert_eq!(result, first_item_action);
    }
}
