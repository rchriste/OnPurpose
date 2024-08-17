use chrono::{DateTime, Utc};
use surrealdb::{
    engine::any::{connect, Any, IntoEndpoint},
    opt::{PatchOp, RecordId},
    sql::{Datetime, Thing},
    Surreal,
};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot::{self, error::RecvError},
};

use crate::{new_item::NewItem, new_time_spent::NewTimeSpent};

use super::{
    surreal_in_the_moment_priority::{
        SurrealAction, SurrealInTheMomentPriority, SurrealPriorityKind,
    },
    surreal_item::{
        Responsibility, SurrealDependency, SurrealFacing, SurrealFrequency, SurrealItem,
        SurrealItemOldVersion, SurrealItemReview, SurrealItemType, SurrealOrderedSubItem,
        SurrealReviewGuidance, SurrealUrgencyPlan,
    },
    surreal_life_area::SurrealLifeArea,
    surreal_processed_text::SurrealProcessedText,
    surreal_routine::SurrealRoutine,
    surreal_tables::SurrealTables,
    surreal_time_spent::{SurrealTimeSpent, SurrealTimeSpentOldVersion},
    SurrealTrigger,
};

pub(crate) enum DataLayerCommands {
    SendRawData(oneshot::Sender<SurrealTables>),
    SendProcessedText(RecordId, oneshot::Sender<Vec<SurrealProcessedText>>),
    SendTimeSpentLog(oneshot::Sender<Vec<SurrealTimeSpent>>),
    RecordTimeSpent(NewTimeSpent),
    AddProcessedText(String, RecordId),
    FinishItem {
        item: RecordId,
        when_finished: Datetime,
    },
    NewItem(NewItem),
    CoverItemWithANewItem {
        cover_this: RecordId,
        cover_with: NewItem,
    },
    CoverItemWithAnExistingItem {
        item_to_be_covered: RecordId,
        item_that_should_do_the_covering: RecordId,
    },
    CoverItemUntilAnExactDateTime(RecordId, DateTime<Utc>),
    UpdateRelativeImportance {
        parent: RecordId,
        update_this_child: RecordId,
        higher_importance_than_this_child: Option<RecordId>,
    },
    ParentItemWithExistingItem {
        child: RecordId,
        parent: RecordId,
        higher_importance_than_this: Option<RecordId>,
    },
    ParentItemWithANewChildItem {
        child: NewItem,
        parent: RecordId,
        higher_importance_than_this: Option<RecordId>,
    },
    ParentNewItemWithAnExistingChildItem {
        child: RecordId,
        parent_new_item: NewItem,
    },
    ParentItemRemoveParent {
        child: RecordId,
        parent_to_remove: RecordId,
    },
    UpdateResponsibilityAndItemType(RecordId, Responsibility, SurrealItemType),
    AddItemDependency(RecordId, SurrealDependency),
    RemoveItemDependency(RecordId, SurrealDependency),
    UpdateSummary(RecordId, String),
    UpdateFacing(RecordId, Vec<SurrealFacing>),
    UpdateUrgencyPlan(RecordId, Option<SurrealUrgencyPlan>),
    UpdateItemReviewFrequency(RecordId, SurrealFrequency, SurrealReviewGuidance),
    UpdateItemLastReviewedDate(RecordId, Datetime),
    DeclareInTheMomentPriority {
        choice: SurrealAction,
        kind: SurrealPriorityKind,
        not_chosen: Vec<SurrealAction>,
        in_effect_until: Vec<SurrealTrigger>,
    },
}

impl DataLayerCommands {
    pub(crate) async fn get_raw_data(
        sender: &Sender<DataLayerCommands>,
    ) -> Result<SurrealTables, RecvError> {
        let (raw_data_sender, raw_data_receiver) = oneshot::channel();
        sender
            .send(DataLayerCommands::SendRawData(raw_data_sender))
            .await
            .unwrap();
        raw_data_receiver.await
    }

    #[allow(dead_code)] //Remove after this is used beyond the unit tests
    pub(crate) async fn get_processed_text(
        sender: &Sender<DataLayerCommands>,
        for_item: RecordId,
    ) -> Result<Vec<SurrealProcessedText>, RecvError> {
        let (processed_text_tx, processed_text_rx) = oneshot::channel();
        sender
            .send(DataLayerCommands::SendProcessedText(
                for_item,
                processed_text_tx,
            ))
            .await
            .unwrap();
        processed_text_rx.await
    }
}

pub(crate) async fn data_storage_start_and_run(
    mut data_storage_layer_receive_rx: Receiver<DataLayerCommands>,
    endpoint: impl IntoEndpoint,
) {
    let db = connect(endpoint).await.unwrap();
    db.use_ns("OnPurpose").use_db("Russ").await.unwrap(); //TODO: "Russ" should be a parameter, maybe the username or something

    // let updated: Option<SurrealItem> = db.update((SurrealItem::TABLE_NAME, "5i5mkemqn0f1716v3ycw"))
    //     .patch(PatchOp::replace("/urgency_plan", None::<Option<SurrealUrgencyPlan>>)).await.unwrap();
    // assert!(updated.is_some());
    // panic!("Finished");
    loop {
        let received = data_storage_layer_receive_rx.recv().await;
        match received {
            Some(DataLayerCommands::SendRawData(oneshot)) => {
                let surreal_tables = load_from_surrealdb_upgrade_if_needed(&db).await;
                oneshot.send(surreal_tables).unwrap();
            }
            Some(DataLayerCommands::AddProcessedText(processed_text, for_item)) => {
                add_processed_text(processed_text, for_item, &db).await
            }
            Some(DataLayerCommands::SendProcessedText(for_item, send_response_here)) => {
                send_processed_text(for_item, send_response_here, &db).await
            }
            Some(DataLayerCommands::SendTimeSpentLog(sender)) => send_time_spent(sender, &db).await,
            Some(DataLayerCommands::RecordTimeSpent(new_time_spent)) => {
                record_time_spent(new_time_spent, &db).await
            }
            Some(DataLayerCommands::FinishItem {
                item,
                when_finished,
            }) => finish_item(item, when_finished, &db).await,
            Some(DataLayerCommands::NewItem(new_item)) => {
                create_new_item(new_item, &db).await;
            }
            Some(DataLayerCommands::CoverItemWithANewItem {
                cover_this,
                cover_with,
            }) => cover_with_a_new_item(cover_this, cover_with, &db).await,
            Some(DataLayerCommands::CoverItemWithAnExistingItem {
                item_to_be_covered,
                item_that_should_do_the_covering,
            }) => {
                cover_item_with_an_existing_item(
                    item_to_be_covered,
                    item_that_should_do_the_covering,
                    &db,
                )
                .await
            }
            Some(DataLayerCommands::CoverItemUntilAnExactDateTime(item_to_cover, cover_until)) => {
                cover_item_until_an_exact_date_time(item_to_cover, cover_until, &db).await
            }
            Some(DataLayerCommands::ParentItemWithExistingItem {
                child,
                parent,
                higher_importance_than_this,
            }) => {
                parent_item_with_existing_item(child, parent, higher_importance_than_this, &db)
                    .await
            }
            Some(DataLayerCommands::ParentItemWithANewChildItem {
                child,
                parent,
                higher_importance_than_this,
            }) => {
                parent_item_with_a_new_child(child, parent, higher_importance_than_this, &db).await
            }
            Some(DataLayerCommands::ParentNewItemWithAnExistingChildItem {
                child,
                parent_new_item,
            }) => parent_new_item_with_an_existing_child_item(child, parent_new_item, &db).await,
            Some(DataLayerCommands::ParentItemRemoveParent {
                child,
                parent_to_remove,
            }) => {
                let mut parent: SurrealItem =
                    db.select(parent_to_remove.clone()).await.unwrap().unwrap();

                parent.smaller_items_in_priority_order = parent
                    .smaller_items_in_priority_order
                    .into_iter()
                    .filter(|x| match x {
                        SurrealOrderedSubItem::SubItem { surreal_item_id } => {
                            surreal_item_id != &child
                        }
                    })
                    .collect::<Vec<_>>();
                let saved = db
                    .update(parent_to_remove)
                    .content(parent.clone())
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(parent, saved);
            }
            Some(DataLayerCommands::AddItemDependency(record_id, new_ready)) => {
                add_dependency(record_id, new_ready, &db).await
            }
            Some(DataLayerCommands::RemoveItemDependency(record_id, to_remove)) => {
                remove_dependency(record_id, to_remove, &db).await
            }
            Some(DataLayerCommands::UpdateRelativeImportance {
                parent,
                update_this_child,
                higher_importance_than_this_child,
            }) => {
                parent_item_with_existing_item(
                    update_this_child,
                    parent,
                    higher_importance_than_this_child,
                    &db,
                )
                .await
            }
            Some(DataLayerCommands::UpdateItemLastReviewedDate(record_id, new_last_reviewed)) => {
                //TODO: I should probably fix this so it does the update all as one transaction rather than reading in the data and then changing it and writing it out again. That could cause issues if there are multiple writers. The reason why I didn't do it yet is because I only want to update part of the SurrealItemReview type and I need to experiment with the PatchOp::replace to see if and how to make it work with the nested type. Otherwise I might consider just making review_frequency and last_reviewed separate fields and then I can just update the review_frequency and not have to worry about the last_reviewed field.
                let mut item: SurrealItem = db.select(record_id.clone()).await.unwrap().unwrap();
                let mut item_review = item
                    .item_review
                    .expect("You should only review items that have a review frequency");
                item_review.last_reviewed = Some(new_last_reviewed);

                item.item_review = Some(item_review);
                let updated = db
                    .update(record_id)
                    .content(item.clone())
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(item, updated);
            }
            Some(DataLayerCommands::UpdateItemReviewFrequency(
                record_id,
                surreal_frequency,
                surreal_review_guidance,
            )) => {
                //TODO: I should probably fix this so it does the update all as one transaction rather than reading in the data and then changing it and writing it out again. That could cause issues if there are multiple writers. The reason why I didn't do it yet is because I only want to update part of the SurrealItemReview type and I need to experiment with the PatchOp::replace to see if and how to make it work with the nested type. Otherwise I might consider just making review_frequency and last_reviewed separate fields and then I can just update the review_frequency and not have to worry about the last_reviewed field.
                let previous_value: SurrealItem =
                    db.select(record_id.clone()).await.unwrap().unwrap();
                let mut item = previous_value.clone();
                item.item_review = Some(SurrealItemReview {
                    last_reviewed: match previous_value.item_review {
                        Some(SurrealItemReview { last_reviewed, .. }) => last_reviewed,
                        None => None,
                    },
                    review_frequency: surreal_frequency,
                });
                item.review_guidance = Some(surreal_review_guidance);
                let updated = db
                    .update(record_id)
                    .content(item.clone())
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(item, updated);
            }
            Some(DataLayerCommands::UpdateSummary(item, new_summary)) => {
                update_item_summary(item, new_summary, &db).await
            }
            Some(DataLayerCommands::UpdateResponsibilityAndItemType(
                item,
                new_responsibility,
                new_item_type,
            )) => {
                let updated: SurrealItem = db
                    .update(item.clone())
                    .patch(PatchOp::replace(
                        "/responsibility",
                        new_responsibility.clone(),
                    ))
                    .patch(PatchOp::replace("/item_type", new_item_type.clone()))
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(updated.responsibility, new_responsibility);
                assert_eq!(updated.item_type, new_item_type);
            }
            Some(DataLayerCommands::UpdateFacing(record_id, new_facing)) => {
                let updated: SurrealItem = db
                    .update(record_id)
                    .patch(PatchOp::replace("/facing", new_facing.clone()))
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(updated.facing, new_facing);
            }
            Some(DataLayerCommands::UpdateUrgencyPlan(record_id, new_urgency_plan)) => {
                let updated: SurrealItem = db
                    .update(record_id)
                    .patch(PatchOp::replace("/urgency_plan", new_urgency_plan.clone()))
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(updated.urgency_plan, new_urgency_plan);
            }
            Some(DataLayerCommands::DeclareInTheMomentPriority {
                choice,
                kind,
                not_chosen,
                in_effect_until,
            }) => {
                let mut priority = SurrealInTheMomentPriority {
                    id: None,
                    not_chosen,
                    in_effect_until,
                    created: Utc::now().into(),
                    choice,
                    kind,
                };
                let updated = db
                    .create(SurrealInTheMomentPriority::TABLE_NAME)
                    .content(priority.clone())
                    .await
                    .unwrap();
                assert_eq!(1, updated.len());

                let updated: SurrealInTheMomentPriority = updated.into_iter().next().unwrap();
                priority.id = updated.id.clone();
                assert_eq!(priority, updated);
            }
            None => return, //Channel closed, time to shutdown down, exit
        }
    }
}

pub(crate) async fn load_from_surrealdb_upgrade_if_needed(db: &Surreal<Any>) -> SurrealTables {
    //TODO: I should do some timings to see if starting all of these get_all requests and then doing awaits on them later really is faster in Rust. Or if they just for sure don't start until the await. For example I could call this function as many times as possible in 10 sec and time that and then see how many times I can call that function written like this and then again with the get_all being right with the await to make sure that code like this is worth it perf wise.
    let all_items = db.select(SurrealItem::TABLE_NAME);
    let all_life_areas = db.select(SurrealLifeArea::TABLE_NAME);
    let all_routines = db.select(SurrealRoutine::TABLE_NAME);
    let time_spent_log = db.select(SurrealTimeSpent::TABLE_NAME);
    let surreal_in_the_moment_priorities = db.select(SurrealInTheMomentPriority::TABLE_NAME);

    let all_items: Vec<SurrealItem> = match all_items.await {
        Ok(all_items) => all_items,
        Err(err) => {
            println!("Upgrading items table because of issue: {}", err);
            upgrade_items_table(db).await;
            db.select(SurrealItem::TABLE_NAME).await.unwrap()
        }
    };

    let time_spent_log = match time_spent_log.await {
        Ok(time_spent_log) => time_spent_log,
        Err(err) => {
            println!("Time spent log is missing because of issue: {}", err);
            upgrade_time_spent_log(db).await;
            db.select(SurrealTimeSpent::TABLE_NAME).await.unwrap()
        }
    };

    SurrealTables {
        surreal_items: all_items,
        surreal_life_areas: all_life_areas.await.unwrap(),
        surreal_routines: all_routines.await.unwrap(),
        surreal_time_spent_log: time_spent_log,
        surreal_in_the_moment_priorities: surreal_in_the_moment_priorities.await.unwrap(),
    }
}

async fn upgrade_items_table(db: &Surreal<Any>) {
    let a: Vec<SurrealItemOldVersion> = db.select(SurrealItemOldVersion::TABLE_NAME).await.unwrap();
    for item_old_version in a.into_iter() {
        let item: SurrealItem = item_old_version.into();
        let updated: SurrealItem = db
            .update(item.id.clone().unwrap())
            .content(item.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(item, updated);
    }
}

async fn upgrade_time_spent_log(db: &Surreal<Any>) {
    let a: Vec<SurrealTimeSpentOldVersion> = db
        .select(SurrealTimeSpentOldVersion::TABLE_NAME)
        .await
        .unwrap();
    for time_spent_old in a.into_iter() {
        let time_spent: SurrealTimeSpent = time_spent_old.into();
        let updated: SurrealTimeSpent = db
            .update(time_spent.id.clone().expect("In DB"))
            .content(time_spent.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(time_spent, updated);
    }
}

pub(crate) async fn add_processed_text(
    processed_text: String,
    for_item: RecordId,
    db: &Surreal<Any>,
) {
    let for_item: Option<Thing> = for_item.into();
    let data = SurrealProcessedText {
        id: None,
        text: processed_text.clone(),
        when_written: Utc::now().into(),
        for_item: for_item.unwrap(),
    };
    let saved: Vec<SurrealProcessedText> = db
        .create(SurrealProcessedText::TABLE_NAME)
        .content(data)
        .await
        .unwrap();
    assert_eq!(1, saved.len());
    assert_eq!(processed_text, saved.first().unwrap().text);
}

pub(crate) async fn send_processed_text(
    for_item: RecordId,
    send_response_here: oneshot::Sender<Vec<SurrealProcessedText>>,
    db: &Surreal<Any>,
) {
    let mut query_result = db
        .query("SELECT * FROM processed_text WHERE for_item = $for_item")
        .bind(("for_item", for_item))
        .await
        .unwrap();

    let processed_text: Vec<SurrealProcessedText> = query_result.take(0).unwrap();

    send_response_here.send(processed_text).unwrap();
}

async fn send_time_spent(sender: oneshot::Sender<Vec<SurrealTimeSpent>>, db: &Surreal<Any>) {
    let time_spent = db.select(SurrealTimeSpent::TABLE_NAME).await.unwrap();
    sender.send(time_spent).unwrap();
}

async fn record_time_spent(new_time_spent: NewTimeSpent, db: &Surreal<Any>) {
    let mut new_time_spent: SurrealTimeSpent = new_time_spent.into();
    let saved: Vec<SurrealTimeSpent> = db
        .create(SurrealTimeSpent::TABLE_NAME)
        .content(new_time_spent.clone())
        .await
        .unwrap();
    assert_eq!(1, saved.len());
    let saved = saved.into_iter().next().unwrap();
    new_time_spent.id = saved.id.clone();
    assert_eq!(new_time_spent, saved);
}

pub(crate) async fn finish_item(finish_this: RecordId, when_finished: Datetime, db: &Surreal<Any>) {
    let updated: SurrealItem = db
        .update(finish_this)
        .patch(PatchOp::replace("/finished", Some(when_finished.clone())))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.finished, Some(when_finished));
}

async fn create_new_item(new_item: NewItem, db: &Surreal<Any>) -> SurrealItem {
    let mut surreal_item: SurrealItem = SurrealItem::new(new_item, vec![]);
    let created: Vec<SurrealItem> = db
        .create(SurrealItem::TABLE_NAME)
        .content(surreal_item.clone())
        .await
        .unwrap();
    assert!(created.len() == 1);
    let created = created.into_iter().next().unwrap();
    surreal_item.id = created.id.clone();
    assert_eq!(surreal_item, created);

    created
}

async fn cover_with_a_new_item(cover_this: RecordId, cover_with: NewItem, db: &Surreal<Any>) {
    let cover_with = create_new_item(cover_with, db).await;

    let cover_with: Option<Thing> = cover_with.into();
    let cover_with = cover_with.expect("always exists the .into() wraps it in an option");
    let new_dependency = SurrealDependency::AfterItem(cover_with);
    add_dependency(cover_this, new_dependency, db).await;
}

async fn cover_item_with_an_existing_item(
    existing_item_to_be_covered: RecordId,
    existing_item_that_is_doing_the_covering: RecordId,
    db: &Surreal<Any>,
) {
    let new_dependency = SurrealDependency::AfterItem(existing_item_that_is_doing_the_covering);
    add_dependency(existing_item_to_be_covered, new_dependency, db).await;
}

async fn cover_item_until_an_exact_date_time(
    item_to_cover: RecordId,
    cover_until: DateTime<Utc>,
    db: &Surreal<Any>,
) {
    let new_dependency = SurrealDependency::AfterDateTime(cover_until.into());
    add_dependency(item_to_cover, new_dependency, db).await;
}

async fn parent_item_with_existing_item(
    child_record_id: RecordId,
    parent_record_id: RecordId,
    higher_importance_than_this: Option<RecordId>,
    db: &Surreal<Any>,
) {
    //TODO: This should be refactored so it happens inside of a transaction and ideally as one query because if the data is modified between the time that the data is read and the time that the data is written back out then the data could be lost. I haven't done this yet because I need to figure out how to do this inside of a SurrealDB query and I haven't done that yet.
    let mut parent: SurrealItem = db.select(parent_record_id.clone()).await.unwrap().unwrap();
    parent.smaller_items_in_priority_order = parent
        .smaller_items_in_priority_order
        .into_iter()
        .filter(|x| match x {
            SurrealOrderedSubItem::SubItem { surreal_item_id } => {
                surreal_item_id != &child_record_id
            }
        })
        .collect::<Vec<_>>();
    if let Some(higher_priority_than_this) = higher_importance_than_this {
        let index_of_higher_priority = parent
            .smaller_items_in_priority_order
            .iter()
            .position(|x| match x {
                //Note that position() is short-circuiting. If there are multiple matches it could be argued that I should panic or assert but
                //I am just matching the first one and then I just keep going. Because I am still figuring out the design and this is
                //more in the vein of hardening work I think this is fine but feel free to revisit this.
                SurrealOrderedSubItem::SubItem { surreal_item_id } => {
                    surreal_item_id == &higher_priority_than_this
                }
            })
            .expect("Should already be in the list");
        parent.smaller_items_in_priority_order.insert(
            index_of_higher_priority,
            SurrealOrderedSubItem::SubItem {
                surreal_item_id: child_record_id,
            },
        );
    } else {
        parent
            .smaller_items_in_priority_order
            .push(SurrealOrderedSubItem::SubItem {
                surreal_item_id: child_record_id,
            });
    }
    let saved = db
        .update(parent_record_id)
        .content(parent.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(parent, saved);
}

async fn parent_item_with_a_new_child(
    child: NewItem,
    parent: RecordId,
    higher_importance_than_this: Option<RecordId>,
    db: &Surreal<Any>,
) {
    let child = create_new_item(child, db).await;
    parent_item_with_existing_item(
        child.id.expect("In DB"),
        parent,
        higher_importance_than_this,
        db,
    )
    .await
}

async fn parent_new_item_with_an_existing_child_item(
    child: RecordId,
    parent_new_item: NewItem,
    db: &Surreal<Any>,
) {
    //TODO: Write a Unit Test for this
    let smaller_items_in_priority_order = vec![SurrealOrderedSubItem::SubItem {
        surreal_item_id: child,
    }];

    let mut parent_surreal_item =
        SurrealItem::new(parent_new_item, smaller_items_in_priority_order);
    let created = db
        .create(SurrealItem::TABLE_NAME)
        .content(parent_surreal_item.clone())
        .await
        .unwrap();
    assert_eq!(1, created.len());
    let created: SurrealItem = created.into_iter().next().unwrap();
    parent_surreal_item.id = created.id.clone();
    assert_eq!(parent_surreal_item, created);
}

async fn add_dependency(record_id: RecordId, new_dependency: SurrealDependency, db: &Surreal<Any>) {
    //TODO: This should be refactored so it happens inside of a transaction and ideally as one query because if the data is modified between the time that the data is read and the time that the data is written back out then the data could be lost. I haven't done this yet because I need to figure out how to do this inside of a SurrealDB query and I haven't done that yet.

    let mut surreal_item: SurrealItem = db
        .select(record_id.clone())
        .await
        .unwrap()
        .expect("Record exists");
    if surreal_item.dependencies.contains(&new_dependency) {
        //Is already there, nothing to do
    } else {
        surreal_item.dependencies.push(new_dependency);

        let updated: SurrealItem = db
            .update(record_id)
            .content(surreal_item.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(surreal_item, updated);
    }
}

async fn remove_dependency(record_id: RecordId, to_remove: SurrealDependency, db: &Surreal<Any>) {
    let mut surreal_item: SurrealItem = db.select(record_id.clone()).await.unwrap().unwrap();
    surreal_item.dependencies.retain(|x| x != &to_remove);

    let update = db
        .update(record_id)
        .content(surreal_item.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(surreal_item, update);
}

async fn update_item_summary(item_to_update: RecordId, new_summary: String, db: &Surreal<Any>) {
    let updated: SurrealItem = db
        .update(item_to_update.clone())
        .patch(PatchOp::replace("/summary", new_summary.clone()))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.summary, new_summary);
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    use crate::{
        new_item::NewItemBuilder, surrealdb_layer::surreal_item::SurrealHowMuchIsInMyControl,
    };

    #[tokio::test]
    async fn data_starts_empty() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert!(surreal_tables.surreal_items.is_empty());
        assert!(surreal_tables.surreal_life_areas.is_empty());
        assert!(surreal_tables.surreal_routines.is_empty());
        assert!(surreal_tables.surreal_time_spent_log.is_empty());

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn add_new_item() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(surreal_tables.surreal_items.len(), 1);
        assert_eq!(
            SurrealItemType::Undeclared,
            surreal_tables.surreal_items.first().unwrap().item_type
        );

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn add_user_processed_text() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_action = NewItemBuilder::default()
            .summary("New next step")
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(new_action))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        let item = surreal_tables.surreal_items.first().unwrap();
        let processed_text = DataLayerCommands::get_processed_text(
            &sender,
            item.id.clone().expect("Item exists in DB"),
        )
        .await
        .unwrap();

        assert!(processed_text.is_empty());

        sender
            .send(DataLayerCommands::AddProcessedText(
                "Some user processed text".into(),
                item.id.clone().expect("Already in DB"),
            ))
            .await
            .unwrap();

        let (processed_text_tx, processed_text_rx) = oneshot::channel();
        let next_step = surreal_tables.surreal_items.first().unwrap();
        sender
            .send(DataLayerCommands::SendProcessedText(
                next_step.id.clone().expect("Item exists in DB"),
                processed_text_tx,
            ))
            .await
            .unwrap();

        let processed_text = processed_text_rx.await.unwrap();
        assert!(!processed_text.is_empty());
        assert_eq!(
            "Some user processed text",
            processed_text.first().unwrap().text
        );

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn finish_item() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_next_step = NewItemBuilder::default()
            .summary("New next step")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(new_next_step))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let items = surreal_tables.make_items(&now);

        assert_eq!(items.len(), 1);
        let next_step_item = items.first().unwrap();
        assert_eq!(next_step_item.is_finished(), false);

        let when_finished = Utc::now();
        sender
            .send(DataLayerCommands::FinishItem {
                item: next_step_item.get_id().clone().into(),
                when_finished: when_finished.into(),
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let items = surreal_tables.make_items(&now);

        assert_eq!(items.len(), 1);
        let next_step_item = items.first().unwrap();
        assert_eq!(next_step_item.is_finished(), true);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn cover_item_with_a_new_proactive_next_step() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_action = NewItemBuilder::default()
            .summary("Item to be covered")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(new_action))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert_eq!(
            0,
            surreal_tables
                .surreal_items
                .first()
                .unwrap()
                .dependencies
                .len()
        ); //length of zero means nothing is covered
        let item_to_cover = surreal_tables.surreal_items.first().unwrap();

        let new_item = NewItemBuilder::default()
            .summary("Covering item")
            .responsibility(Responsibility::ProactiveActionToTake)
            .item_type(SurrealItemType::Action)
            .build()
            .unwrap();

        sender
            .send(DataLayerCommands::CoverItemWithANewItem {
                cover_this: item_to_cover.id.clone().expect("In DB"),
                cover_with: new_item,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        let item_that_should_be_covered = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "Item to be covered")
            .unwrap();
        assert_eq!(2, surreal_tables.surreal_items.len());
        assert_eq!(1, item_that_should_be_covered.dependencies.len()); //expect one item to be is covered
        let item_that_should_cover = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "Covering item")
            .unwrap();
        let id = match &item_that_should_be_covered.dependencies.first().unwrap() {
            SurrealDependency::AfterItem(id) => id,
            _ => panic!("Should be an item"),
        };
        assert_eq!(item_that_should_cover.id.as_ref().unwrap(), id,);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn cover_item_until_an_exact_date_time() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_action = NewItemBuilder::default()
            .summary("Item to be covered")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(new_action))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert!(surreal_tables
            .surreal_items
            .first()
            .unwrap()
            .dependencies
            .is_empty());

        let cover_until: chrono::DateTime<Utc> = Utc::now();
        sender
            .send(DataLayerCommands::CoverItemUntilAnExactDateTime(
                surreal_tables
                    .surreal_items
                    .into_iter()
                    .next()
                    .unwrap()
                    .id
                    .expect("In DB"),
                cover_until.clone().into(),
            ))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert_eq!(
            1,
            surreal_tables
                .surreal_items
                .first()
                .unwrap()
                .dependencies
                .len()
        );
        let until = match surreal_tables
            .surreal_items
            .first()
            .unwrap()
            .dependencies
            .first()
            .unwrap()
        {
            SurrealDependency::AfterDateTime(until) => until,
            _ => panic!("Should be an item"),
        };

        let cover_until: Datetime = cover_until.into();
        assert_eq!(cover_until, *until);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_item_with_a_new_item() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_action = NewItemBuilder::default()
            .summary("Item that needs a parent")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(new_action))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());

        sender
            .send(DataLayerCommands::ParentNewItemWithAnExistingChildItem {
                child: surreal_tables
                    .surreal_items
                    .into_iter()
                    .next()
                    .unwrap()
                    .id
                    .expect("In Db"),
                parent_new_item: NewItemBuilder::default()
                    .summary("Parent Item")
                    .item_type(SurrealItemType::Goal(SurrealHowMuchIsInMyControl::default()))
                    .build()
                    .unwrap(),
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(2, surreal_tables.surreal_items.len());
        assert_eq!(
            1,
            surreal_tables
                .surreal_items
                .iter()
                .find(|x| x.summary == "Parent Item")
                .unwrap()
                .smaller_items_in_priority_order
                .len()
        );
        assert_eq!(
            &SurrealOrderedSubItem::SubItem {
                surreal_item_id: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Item that needs a parent")
                    .unwrap()
                    .id
                    .as_ref()
                    .unwrap()
                    .clone()
            },
            surreal_tables
                .surreal_items
                .iter()
                .find(|x| x.summary == "Parent Item")
                .unwrap()
                .smaller_items_in_priority_order
                .first()
                .unwrap()
        );

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_item_with_an_existing_item_that_has_no_children() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let item_that_needs_a_parent = NewItemBuilder::default()
            .summary("Item that needs a parent")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(item_that_needs_a_parent))
            .await
            .unwrap();

        let parent_item = NewItemBuilder::default()
            .summary("Parent Item")
            .item_type(SurrealItemType::Goal(SurrealHowMuchIsInMyControl::default()))
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(parent_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(2, surreal_tables.surreal_items.len());

        sender
            .send(DataLayerCommands::ParentItemWithExistingItem {
                child: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Item that needs a parent")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                higher_importance_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(2, surreal_tables.surreal_items.len());
        assert_eq!(
            1,
            surreal_tables
                .surreal_items
                .iter()
                .find(|x| x.summary == "Parent Item")
                .unwrap()
                .smaller_items_in_priority_order
                .len()
        );
        assert_eq!(
            &SurrealOrderedSubItem::SubItem {
                surreal_item_id: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Item that needs a parent")
                    .unwrap()
                    .id
                    .as_ref()
                    .unwrap()
                    .clone()
            },
            surreal_tables
                .surreal_items
                .iter()
                .find(|x| x.summary == "Parent Item")
                .unwrap()
                .smaller_items_in_priority_order
                .first()
                .unwrap()
        );

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_item_with_an_existing_item_that_has_children() {
        // SETUP
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let child_item = NewItemBuilder::default()
            .summary("Child Item at the top of the list")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(child_item))
            .await
            .unwrap();

        let child_item = NewItemBuilder::default()
            .summary("Child Item 2nd position")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(child_item))
            .await
            .unwrap();

        let child_item = NewItemBuilder::default()
            .summary("Child Item 3rd position")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(child_item))
            .await
            .unwrap();

        let child_item = NewItemBuilder::default()
            .summary("Child Item bottom position")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(child_item))
            .await
            .unwrap();

        let parent_item = NewItemBuilder::default()
            .summary("Parent Item")
            .item_type(SurrealItemType::Goal(SurrealHowMuchIsInMyControl::default()))
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(parent_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(5, surreal_tables.surreal_items.len());

        sender
            .send(DataLayerCommands::ParentItemWithExistingItem {
                child: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Child Item at the top of the list")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                higher_importance_than_this: None,
            })
            .await
            .unwrap();

        // TEST - The order of adding the items is meant to cause the higher_priority_than_this to be used

        sender
            .send(DataLayerCommands::ParentItemWithExistingItem {
                child: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Child Item bottom position")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                higher_importance_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithExistingItem {
                child: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Child Item 2nd position")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                higher_importance_than_this: Some(
                    surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| x.summary == "Child Item bottom position")
                        .unwrap()
                        .id
                        .clone()
                        .expect("In DB"),
                ),
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithExistingItem {
                child: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Child Item 3rd position")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                higher_importance_than_this: Some(
                    surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| x.summary == "Child Item bottom position")
                        .unwrap()
                        .id
                        .clone()
                        .expect("In DB"),
                ),
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(
            vec![
                SurrealOrderedSubItem::SubItem {
                    surreal_item_id: surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| x.summary == "Child Item at the top of the list")
                        .unwrap()
                        .id
                        .as_ref()
                        .unwrap()
                        .clone()
                },
                SurrealOrderedSubItem::SubItem {
                    surreal_item_id: surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| x.summary == "Child Item 2nd position")
                        .unwrap()
                        .id
                        .as_ref()
                        .unwrap()
                        .clone()
                },
                SurrealOrderedSubItem::SubItem {
                    surreal_item_id: surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| x.summary == "Child Item 3rd position")
                        .unwrap()
                        .id
                        .as_ref()
                        .unwrap()
                        .clone()
                },
                SurrealOrderedSubItem::SubItem {
                    surreal_item_id: surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| x.summary == "Child Item bottom position")
                        .unwrap()
                        .id
                        .as_ref()
                        .unwrap()
                        .clone()
                },
            ],
            surreal_tables
                .surreal_items
                .iter()
                .find(|x| x.summary == "Parent Item")
                .unwrap()
                .smaller_items_in_priority_order
        );

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn change_order_of_children() {
        // SETUP
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let child_item = NewItemBuilder::default()
            .summary("Child Item at the top of the list")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(child_item))
            .await
            .unwrap();

        let child_item = NewItemBuilder::default()
            .summary("Child Item 2nd position")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(child_item))
            .await
            .unwrap();

        let child_item = NewItemBuilder::default()
            .summary("Child Item 3rd position")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(child_item))
            .await
            .unwrap();

        let child_item = NewItemBuilder::default()
            .summary("Child Item bottom position, then moved to above 2nd position")
            .item_type(SurrealItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(child_item))
            .await
            .unwrap();

        let parent_item = NewItemBuilder::default()
            .summary("Parent Item")
            .item_type(SurrealItemType::Goal(SurrealHowMuchIsInMyControl::default()))
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(parent_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(5, surreal_tables.surreal_items.len());

        sender
            .send(DataLayerCommands::ParentItemWithExistingItem {
                child: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Child Item at the top of the list")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                higher_importance_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithExistingItem {
                child: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| {
                        x.summary == "Child Item bottom position, then moved to above 2nd position"
                    })
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                higher_importance_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithExistingItem {
                child: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Child Item 2nd position")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                higher_importance_than_this: Some(
                    surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| {
                            x.summary
                                == "Child Item bottom position, then moved to above 2nd position"
                        })
                        .unwrap()
                        .id
                        .clone()
                        .expect("In DB"),
                ),
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithExistingItem {
                child: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Child Item 3rd position")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                higher_importance_than_this: Some(
                    surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| {
                            x.summary
                                == "Child Item bottom position, then moved to above 2nd position"
                        })
                        .unwrap()
                        .id
                        .clone()
                        .expect("In DB"),
                ),
            })
            .await
            .unwrap();

        // TEST - Move the bottom item to the 2nd position
        sender
            .send(DataLayerCommands::ParentItemWithExistingItem {
                child: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| {
                        x.summary == "Child Item bottom position, then moved to above 2nd position"
                    })
                    .unwrap()
                    .id
                    .clone()
                    .expect("In DB"),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
                    .id
                    .as_ref()
                    .expect("In DB")
                    .clone(),
                higher_importance_than_this: Some(
                    surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| x.summary == "Child Item 2nd position")
                        .unwrap()
                        .id
                        .clone()
                        .expect("In DB"),
                ),
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(
            vec![
                SurrealOrderedSubItem::SubItem {
                    surreal_item_id: surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| x.summary == "Child Item at the top of the list")
                        .unwrap()
                        .id
                        .as_ref()
                        .unwrap()
                        .clone()
                },
                SurrealOrderedSubItem::SubItem {
                    surreal_item_id: surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| x.summary
                            == "Child Item bottom position, then moved to above 2nd position")
                        .unwrap()
                        .id
                        .as_ref()
                        .unwrap()
                        .clone()
                },
                SurrealOrderedSubItem::SubItem {
                    surreal_item_id: surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| x.summary == "Child Item 2nd position")
                        .unwrap()
                        .id
                        .as_ref()
                        .unwrap()
                        .clone()
                },
                SurrealOrderedSubItem::SubItem {
                    surreal_item_id: surreal_tables
                        .surreal_items
                        .iter()
                        .find(|x| x.summary == "Child Item 3rd position")
                        .unwrap()
                        .id
                        .as_ref()
                        .unwrap()
                        .clone()
                },
            ],
            surreal_tables
                .surreal_items
                .iter()
                .find(|x| x.summary == "Parent Item")
                .unwrap()
                .smaller_items_in_priority_order
        );

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }
}
