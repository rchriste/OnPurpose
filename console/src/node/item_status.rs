use std::{ops::Sub, time::Duration};

use chrono::{DateTime, Utc};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};

use crate::{
    base_data::item::Item,
    surrealdb_layer::surreal_item::{EnterListReason, ItemType, Staging},
};

use super::item_node::{GrowingItemNode, ItemNode, ShrinkingItemNode};

#[derive(Clone, Debug)]
pub(crate) struct ItemStatus<'s> {
    item_node: ItemNode<'s>,
    lap_count: f32,
    is_snoozed: bool,
}

impl<'s> ItemStatus<'s> {
    pub(crate) fn new(
        item_node: ItemNode<'s>,
        all_nodes: &[ItemNode<'_>],
        current_date_time: &DateTime<Utc>,
    ) -> Self {
        let lap_count = calculate_lap_count(&item_node, all_nodes, current_date_time);
        let is_snoozed = calculate_is_snoozed(&item_node, all_nodes, current_date_time);
        Self {
            item_node,
            lap_count,
            is_snoozed,
        }
    }

    pub(crate) fn get_lap_count(&self) -> f32 {
        self.lap_count
    }

    pub(crate) fn is_snoozed(&self) -> bool {
        self.is_snoozed
    }

    pub(crate) fn is_first_lap_finished(&self) -> bool {
        self.get_lap_count() > 1.0
    }

    pub(crate) fn get_item_node(&'s self) -> &'s ItemNode<'s> {
        &self.item_node
    }

    pub(crate) fn get_staging(&self) -> &Staging {
        self.item_node.get_staging()
    }

    pub(crate) fn is_staging_not_set(&self) -> bool {
        self.item_node.is_staging_not_set()
    }

    pub(crate) fn get_thing(&self) -> &Thing {
        self.item_node.get_thing()
    }

    pub(crate) fn is_responsibility_reactive(&self) -> bool {
        self.item_node.is_responsibility_reactive()
    }

    pub(crate) fn is_type_undeclared(&self) -> bool {
        self.item_node.is_type_undeclared()
    }

    pub(crate) fn is_staging_mentally_resident(&self) -> bool {
        self.item_node.is_staging_mentally_resident()
    }

    pub(crate) fn is_person_or_group(&self) -> bool {
        self.item_node.is_person_or_group()
    }

    pub(crate) fn get_item(&self) -> &Item<'s> {
        self.item_node.get_item()
    }

    pub(crate) fn has_active_children(&self) -> bool {
        self.item_node.has_active_children()
    }

    pub(crate) fn get_smaller(&'s self) -> &[ShrinkingItemNode<'s>] {
        self.item_node.get_smaller()
    }

    pub(crate) fn get_type(&self) -> &ItemType {
        self.item_node.get_type()
    }

    pub(crate) fn get_surreal_record_id(&self) -> &RecordId {
        self.item_node.get_surreal_record_id()
    }

    pub(crate) fn has_larger(&self) -> bool {
        self.item_node.has_larger()
    }

    pub(crate) fn get_larger(&'s self) -> impl Iterator<Item = &GrowingItemNode<'s>> {
        self.item_node.get_larger()
    }
}

fn calculate_lap_count(
    item_node: &ItemNode<'_>,
    all_nodes: &[ItemNode<'_>],
    current_date_time: &DateTime<Utc>,
) -> f32 {
    match item_node.get_staging() {
        Staging::NotSet => 0.0,
        Staging::OnDeck { enter_list, lap } | Staging::MentallyResident { enter_list, lap } => {
            match enter_list {
                EnterListReason::DateTime(enter_time) => {
                    let enter_time: DateTime<Utc> = enter_time.clone().into();
                    let lap: Duration = (*lap).into();
                    let elapsed = current_date_time.sub(enter_time);
                    let elapsed = elapsed.num_seconds() as f32;
                    let lap = lap.as_secs_f32();
                    elapsed / lap
                }
                EnterListReason::HighestUncovered {
                    earliest,
                    review_after,
                } => {
                    if current_date_time < earliest {
                        return 0.0;
                    }
                    let all_larger = item_node.get_larger();
                    let all_larger = all_larger
                        .map(|x| x.get_node(all_nodes))
                        .collect::<Vec<_>>();
                    let mut all_larger_iter = all_larger.iter();
                    let (highest_uncovered, uncovered_when) = loop {
                        let larger = all_larger_iter.next();
                        match larger {
                            Some(larger) => {
                                let (highest_uncovered, uncovered_when) =
                                    find_highest_uncovered_child_with_when_uncovered(
                                        larger,
                                        current_date_time,
                                    );
                                if let Some(highest_uncovered) = highest_uncovered {
                                    break (Some(highest_uncovered), uncovered_when);
                                }
                            }
                            None => break (None, None),
                        }
                    };
                    match highest_uncovered {
                        Some(highest_uncovered) => {
                            if highest_uncovered == item_node.get_item() {
                                let uncovered_when: DateTime<Utc> = match uncovered_when {
                                    Some(_) => todo!(),
                                    None => earliest.clone().into(),
                                };
                                let elapsed = current_date_time.sub(uncovered_when);
                                let elapsed = elapsed.num_seconds() as f32;
                                let lap = lap.as_secs_f32();
                                elapsed / lap
                            } else {
                                0.0
                            }
                        }
                        None => 0.0,
                    }
                }
            }
        }
        Staging::Planned => 0.0,
        Staging::ThinkingAbout => 0.0,
        Staging::Released => 0.0,
    }
}

/// You can be snoozed if you are covered or if you just haven't reached the starting on staging yet
fn calculate_is_snoozed(
    item_node: &ItemNode<'_>,
    all_nodes: &[ItemNode<'_>],
    now: &DateTime<Utc>,
) -> bool {
    let staging = item_node.get_staging();
    let snoozed_from_staging = match staging {
        Staging::NotSet => false,
        Staging::OnDeck { enter_list, .. } | Staging::MentallyResident { enter_list, .. } => {
            match enter_list {
                EnterListReason::DateTime(enter_list) => {
                    let enter_list: DateTime<Utc> = enter_list.clone().into();
                    &enter_list > now
                }
                EnterListReason::HighestUncovered {
                    earliest,
                    review_after,
                } => {
                    if now < earliest {
                        return true;
                    }
                    let all_larger = item_node.get_larger();
                    let all_larger = all_larger
                        .map(|x| x.get_node(all_nodes))
                        .collect::<Vec<_>>();
                    let mut all_larger_iter = all_larger.iter();
                    let (highest_uncovered, uncovered_when) = loop {
                        let larger = all_larger_iter.next();
                        match larger {
                            Some(larger) => {
                                let (highest_uncovered, uncovered_when) =
                                    find_highest_uncovered_child_with_when_uncovered(larger, now);
                                if let Some(highest_uncovered) = highest_uncovered {
                                    break (Some(highest_uncovered), uncovered_when);
                                }
                            }
                            None => break (None, None),
                        }
                    };
                    match highest_uncovered {
                        Some(highest_uncovered) => {
                            if highest_uncovered == item_node.get_item() {
                                let uncovered_when: DateTime<Utc> = match uncovered_when {
                                    Some(_) => todo!(),
                                    None => earliest.clone().into(),
                                };
                                false
                            } else {
                                true
                            }
                        }
                        None => true,
                    }
                }
            }
        }
        Staging::Planned => false,
        Staging::ThinkingAbout => false,
        Staging::Released => false,
    };

    item_node.get_snoozed_until().iter().any(|x| x > &now) || snoozed_from_staging
}

fn find_highest_uncovered_child_with_when_uncovered<'a>(
    item_node: &'a ItemNode<'a>,
    now: &DateTime<Utc>,
) -> (Option<&'a Item<'a>>, Option<DateTime<Utc>>) {
    let now: Datetime = (*now).into();
    let mut when_uncovered = None;
    for child in item_node.get_smaller().iter() {
        if child.is_finished() {
            match when_uncovered {
                Some(when_uncovered) => todo!(),
                None => {
                    let when_child_finished = child.when_finished().expect("is_finished() is true");
                    match when_uncovered {
                        Some(uncovered) => {
                            if when_child_finished > uncovered {
                                // We want the latest finished child
                                when_uncovered = Some(when_child_finished)
                            }
                        }
                        None => when_uncovered = Some(when_child_finished),
                    }
                }
            }
            continue;
        }
        let staging = child.get_staging();
        match staging {
            Staging::NotSet => todo!(),
            Staging::MentallyResident { enter_list, .. } => {
                match enter_list {
                    EnterListReason::DateTime(datetime) => {
                        if datetime < &now {
                            return (Some(child.get_item()), when_uncovered);
                        } else {
                            todo!("Do I need to update latest_uncovered?");
                            continue;
                        }
                    }
                    EnterListReason::HighestUncovered {
                        earliest,
                        review_after: _review_after,
                    } => {
                        if earliest > &now {
                            todo!("Do I need to update latest_uncovered? I think I can just remove this");
                            continue;
                        }
                        todo!();
                    }
                }
            }
            Staging::OnDeck { enter_list, .. } => match enter_list {
                EnterListReason::DateTime(datetime) => {
                    if datetime < &now {
                        return (Some(child.get_item()), when_uncovered);
                    } else {
                        todo!("Do I need to update latest_uncovered?");
                        continue;
                    }
                }
                EnterListReason::HighestUncovered {
                    earliest,
                    review_after: _review_after,
                } => {
                    if earliest > &now {
                        todo!("Do I need to update latest_uncovered?");
                        continue;
                    } else {
                        return (Some(child.get_item()), when_uncovered);
                    }
                }
            },
            Staging::Planned => todo!(),
            Staging::ThinkingAbout => todo!(),
            Staging::Released => todo!(),
        }
    }
    (None, when_uncovered)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Days, Utc};
    use surrealdb::sql::{Datetime, Duration};
    use tokio::sync::mpsc;

    use crate::{
        base_data::BaseData,
        calculated_data::CalculatedData,
        new_item::{NewItem, NewItemBuilder},
        surrealdb_layer::{
            data_storage_start_and_run,
            surreal_item::{EnterListReason, Staging},
            surreal_tables::SurrealTables,
            DataLayerCommands,
        },
    };

    #[tokio::test]
    async fn parent_node_1_child_configured_for_highest_uncovered_if_before_earliest_time_it_should_remain_snoozed(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("New Child Item")
                    .staging(Staging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(10))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: Duration::from_days(1),
                    })
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        // Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let item_status = calculated_data.get_item_status();
        let child = item_status
            .iter()
            .find(|x| x.get_item().get_summary() == "New Child Item")
            .unwrap();

        // Assert
        assert_eq!(child.is_snoozed(), true);
        assert_eq!(child.get_lap_count(), 0.0);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_1_child_configured_for_highest_uncovered_it_should_be_ready_immediately(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("New Child Item")
                    .staging(Staging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: Duration::from_days(1),
                    })
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        // Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let item_status = calculated_data.get_item_status();
        let child = item_status
            .iter()
            .find(|x| x.get_item().get_summary() == "New Child Item")
            .unwrap();

        // Assert
        assert_eq!(child.is_snoozed(), false);
        assert!(child.get_lap_count() > 0.0);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_2_children_both_configured_for_highest_uncovered_higher_one_becomes_ready_and_lower_one_stays_covered(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Higher Child That Should Become Ready")
                    .staging(Staging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: Duration::from_days(1),
                    })
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Stay Covered")
                    .staging(Staging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: Duration::from_days(1),
                    })
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let item_status = calculated_data.get_item_status();
        let higher_child = item_status
            .iter()
            .find(|x| x.get_item().get_summary() == "Higher Child That Should Become Ready")
            .unwrap();

        let lower_child = item_status
            .iter()
            .find(|x| x.get_item().get_summary() == "Lower Child That Should Stay Covered")
            .unwrap();

        //Assert
        assert_eq!(higher_child.is_snoozed(), false);
        assert!(higher_child.get_lap_count() > 0.0);
        assert_eq!(lower_child.is_snoozed(), true);
        assert_eq!(lower_child.get_lap_count(), 0.0);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_2_children_highest_one_on_deck_lower_child_configured_for_highest_uncovered_it_should_not_be_ready(
    ) {
        todo!("This test is probably not needed")
    }

    #[tokio::test]
    async fn parent_node_with_2_children_highest_one_finished_lower_child_configured_for_highest_uncovered_it_should_be_ready_when_the_other_item_is_finished(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Higher Child That Was On Deck But Becomes Finished")
                    .staging(Staging::OnDeck {
                        enter_list: EnterListReason::DateTime(Datetime(DateTime::from(
                            Utc::now()
                                .checked_sub_days(Days::new(1))
                                .expect("Far from overflowing"),
                        ))),
                        lap: Duration::from_days(1),
                    })
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Uncover When The Higher Child Is Finished")
                    .staging(Staging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: Duration::from_days(1),
                    })
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let lower_child = base_data
            .get_active_items()
            .iter()
            .find(|x| x.get_summary() == "Higher Child That Was On Deck But Becomes Finished")
            .unwrap();
        sender
            .send(DataLayerCommands::FinishItem(
                lower_child.get_surreal_record_id().clone(),
            ))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let a_day_later_so_a_lap_passes = now
            .checked_add_days(Days::new(1))
            .expect("Far from overflowing");
        let calculated_data =
            CalculatedData::new_from_base_data(base_data, &a_day_later_so_a_lap_passes);
        let item_status = calculated_data.get_item_status();
        let lower_child = item_status
            .iter()
            .find(|x| {
                x.get_item().get_summary()
                    == "Lower Child That Should Uncover When The Higher Child Is Finished"
            })
            .unwrap();

        //Assert
        assert_eq!(lower_child.is_snoozed(), false);
        assert!(
            lower_child.get_lap_count() > 0.9,
            "Lap count was {}",
            lower_child.get_lap_count()
        );
        assert!(
            lower_child.get_lap_count() < 1.1,
            "Lap count was {}",
            lower_child.get_lap_count()
        );

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[test]
    fn parent_node_with_2_children_highest_one_covered_and_lower_child_configured_for_highest_uncovered_it_should_be_ready_when_highest_became_covered(
    ) {
        todo!("test case todo")
    }

    #[test]
    fn parent_node_with_2_children_highest_one_mentally_resident_lower_item_after_review_by_date_it_should_show_up_as_needing_review(
    ) {
        todo!("test case todo")
    }
}
