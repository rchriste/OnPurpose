pub(crate) mod surreal_covering;
pub(crate) mod surreal_covering_until_date_time;
pub(crate) mod surreal_item;
pub(crate) mod surreal_life_area;
pub(crate) mod surreal_processed_text;
pub(crate) mod surreal_required_circumstance;
pub(crate) mod surreal_routine;
pub(crate) mod surreal_tables;

use chrono::{DateTime, Local, Utc};
use surrealdb::{
    engine::any::{connect, Any, IntoEndpoint},
    error::Api,
    opt::RecordId,
    sql::Thing,
    Error, Surreal,
};
use surrealdb_extra::table::{Table, TableError};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot::{self, error::RecvError},
};

use crate::new_item::NewItem;

use self::{
    surreal_covering::SurrealCovering,
    surreal_covering_until_date_time::SurrealCoveringUntilDatetime,
    surreal_item::{
        Facing, ItemType, Permanence, Responsibility, Staging, SurrealItem, SurrealItemOldVersion,
        SurrealOrderedSubItem,
    },
    surreal_life_area::SurrealLifeArea,
    surreal_processed_text::SurrealProcessedText,
    surreal_required_circumstance::SurrealRequiredCircumstance,
    surreal_routine::SurrealRoutine,
    surreal_tables::SurrealTables,
};

pub(crate) enum DataLayerCommands {
    SendRawData(oneshot::Sender<SurrealTables>),
    SendProcessedText(RecordId, oneshot::Sender<Vec<SurrealProcessedText>>),
    AddProcessedText(String, RecordId),
    FinishItem(RecordId),
    NewItem(NewItem),
    CoverItemWithANewItem {
        cover_this: RecordId,
        cover_with: NewItem,
    },
    CoverItemWithAnExistingItem {
        item_to_be_covered: RecordId,
        item_that_should_do_the_covering: RecordId,
    },
    #[allow(dead_code)]
    //This was initially added for data migration that is now removed but I expect to want it again in the future
    RemoveCoveringItem(SurrealCovering),
    CoverItemUntilAnExactDateTime(RecordId, DateTime<Utc>),
    ParentItemWithExistingItem {
        child: RecordId,
        parent: RecordId,
        higher_priority_than_this: Option<RecordId>,
    },
    ParentItemWithANewChildItem {
        child: NewItem,
        parent: RecordId,
        higher_priority_than_this: Option<RecordId>,
    },
    ParentNewItemWithAnExistingChildItem {
        child: RecordId,
        parent_new_item: NewItem,
    },
    UpdateResponsibilityAndItemType(RecordId, Responsibility, ItemType),
    UpdateItemResponsibility(RecordId, Responsibility),
    UpdateItemPermanence(RecordId, Permanence),
    UpdateItemStaging(RecordId, Staging),
    UpdateItemSummary(RecordId, String),
    UpdateFacing(RecordId, Vec<Facing>),
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
    db.use_ns("OnPurpose").use_db("Russ").await.unwrap();

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
            Some(DataLayerCommands::FinishItem(item)) => finish_item(item, &db).await,
            Some(DataLayerCommands::NewItem(new_item)) => {
                super::surrealdb_layer::new_item(new_item, &db).await;
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
            Some(DataLayerCommands::RemoveCoveringItem(surreal_covering)) => {
                SurrealCovering::delete(surreal_covering.id.unwrap().id.to_raw(), &db)
                    .await
                    .unwrap()
                    .unwrap(); //2nd unwrap ensures the delete actually happened
            }
            Some(DataLayerCommands::CoverItemUntilAnExactDateTime(item_to_cover, cover_until)) => {
                cover_item_until_an_exact_date_time(item_to_cover, cover_until, &db).await
            }
            Some(DataLayerCommands::ParentItemWithExistingItem {
                child,
                parent,
                higher_priority_than_this,
            }) => {
                parent_item_with_existing_item(child, parent, higher_priority_than_this, &db).await
            }
            Some(DataLayerCommands::ParentItemWithANewChildItem {
                child,
                parent,
                higher_priority_than_this,
            }) => parent_item_with_a_new_child(child, parent, higher_priority_than_this, &db).await,
            Some(DataLayerCommands::ParentNewItemWithAnExistingChildItem {
                child,
                parent_new_item,
            }) => parent_new_item_with_an_existing_child_item(child, parent_new_item, &db).await,
            Some(DataLayerCommands::UpdateItemPermanence(item, new_permanence)) => {
                update_hope_permanence(item, new_permanence, &db).await
            }
            Some(DataLayerCommands::UpdateItemStaging(record_id, new_staging)) => {
                update_hope_staging(record_id, new_staging, &db).await
            }
            Some(DataLayerCommands::UpdateItemSummary(item, new_summary)) => {
                update_item_summary(item, new_summary, &db).await
            }
            Some(DataLayerCommands::UpdateResponsibilityAndItemType(
                item,
                new_responsibility,
                new_item_type,
            )) => {
                let mut item = SurrealItem::get_by_id(item.id.to_raw(), &db)
                    .await
                    .unwrap()
                    .unwrap();
                item.responsibility = new_responsibility;
                item.item_type = new_item_type;
                let new = db
                    .update((
                        SurrealItem::TABLE_NAME,
                        item.get_id()
                            .clone()
                            .expect("Came from the DB")
                            .id
                            .clone()
                            .to_raw(),
                    ))
                    .content(&item)
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(item, new);
            }
            Some(DataLayerCommands::UpdateItemResponsibility(record_id, new_responsibility)) => {
                let mut item = SurrealItem::get_by_id(record_id.id.to_raw(), &db)
                    .await
                    .unwrap()
                    .unwrap();
                item.responsibility = new_responsibility;
                item.update(&db).await.unwrap();
            }
            Some(DataLayerCommands::UpdateFacing(record_id, new_facing)) => {
                let mut item = SurrealItem::get_by_id(record_id.id.to_raw(), &db)
                    .await
                    .unwrap()
                    .unwrap();
                item.facing = new_facing;
                let updated = item.clone().update(&db).await.unwrap().unwrap();
                assert_eq!(item, updated);
            }
            None => return, //Channel closed, time to shutdown down, exit
        }
    }
}

pub(crate) async fn load_from_surrealdb_upgrade_if_needed(db: &Surreal<Any>) -> SurrealTables {
    //TODO: I should do some timings to see if starting all of these get_all requests and then doing awaits on them later really is faster in Rust. Or if they just for sure don't start until the await. For example I could call this function as many times as possible in 10 sec and time that and then see how many times I can call that function written like this and then again with the get_all being right with the await to make sure that code like this is worth it perf wise.
    let all_items = SurrealItem::get_all(db);
    let all_coverings = SurrealCovering::get_all(db);
    let all_required_circumstances = SurrealRequiredCircumstance::get_all(db);
    let all_coverings_until_date_time = SurrealCoveringUntilDatetime::get_all(db);
    let all_life_areas = SurrealLifeArea::get_all(db);
    let all_routines = SurrealRoutine::get_all(db);

    let all_items = match all_items.await {
        Ok(all_items) => all_items,
        Err(TableError::Db(Error::Api(Api::FromValue { value: _, error }))) => {
            println!("Upgrading items table because of issue: {}", error);
            upgrade_items_table(db).await;
            SurrealItem::get_all(db).await.unwrap()
        }
        _ => todo!(),
    };

    SurrealTables {
        surreal_items: all_items,
        surreal_coverings: all_coverings.await.unwrap(),
        surreal_required_circumstances: all_required_circumstances.await.unwrap(),
        surreal_coverings_until_date_time: all_coverings_until_date_time.await.unwrap(),
        surreal_life_areas: all_life_areas.await.unwrap(),
        surreal_routines: all_routines.await.unwrap(),
    }
}

async fn upgrade_items_table(db: &Surreal<Any>) {
    for item_old_version in SurrealItemOldVersion::get_all(db)
        .await
        .unwrap()
        .into_iter()
    {
        let item: SurrealItem = item_old_version.into();
        let _: SurrealItem = db
            .update((
                SurrealItem::TABLE_NAME,
                item.get_id()
                    .clone()
                    .ok_or(TableError::IdEmpty)
                    .unwrap()
                    .id
                    .clone()
                    .to_raw(),
            ))
            .content(item)
            .await
            .unwrap()
            .unwrap();
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
        text: processed_text,
        when_written: Local::now().naive_utc().and_utc().into(),
        for_item: for_item.unwrap(),
    };
    data.create(db).await.unwrap();
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

pub(crate) async fn finish_item(finish_this: RecordId, db: &Surreal<Any>) {
    let mut finish_this = SurrealItem::get_by_id(finish_this.id.to_raw(), db)
        .await
        .unwrap()
        .unwrap();
    finish_this.finished = Some(Local::now().naive_utc().and_utc().into());
    finish_this.update(db).await.unwrap();
}

async fn new_item(new_item: NewItem, db: &Surreal<Any>) -> SurrealItem {
    let surreal_item: SurrealItem = SurrealItem::new(new_item, vec![]);
    surreal_item
        .create(db)
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("I just created one item it should be there")
}

async fn cover_with_a_new_item(cover_this: RecordId, cover_with: NewItem, db: &Surreal<Any>) {
    let cover_with = SurrealItem::new(cover_with, vec![]);
    let cover_with = cover_with
        .create(db)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    let cover_with: Option<Thing> = cover_with.into();
    let cover_this: Option<Thing> = cover_this.into();
    SurrealCovering {
        id: None,
        smaller: cover_with.expect("Should already be in the database"),
        parent: cover_this.expect("Should already be in the database"),
    }
    .create(db)
    .await
    .unwrap();
}

async fn cover_item_with_an_existing_item(
    existing_item_to_be_covered: RecordId,
    existing_item_that_is_doing_the_covering: RecordId,
    db: &Surreal<Any>,
) {
    let smaller_option: Option<Thing> = existing_item_that_is_doing_the_covering.into();
    let parent_option: Option<Thing> = existing_item_to_be_covered.into();
    SurrealCovering {
        id: None,
        smaller: smaller_option.expect("Should already be in the database"),
        parent: parent_option.expect("Should already be in the database"),
    }
    .create(db)
    .await
    .unwrap();
}

async fn cover_item_until_an_exact_date_time(
    item_to_cover: RecordId,
    cover_until: DateTime<Utc>,
    db: &Surreal<Any>,
) {
    SurrealCoveringUntilDatetime {
        id: None,
        cover_this: item_to_cover,
        until: cover_until.into(),
    }
    .create(db)
    .await
    .unwrap();
}

async fn parent_item_with_existing_item(
    child: RecordId,
    parent: RecordId,
    higher_priority_than_this: Option<RecordId>,
    db: &Surreal<Any>,
) {
    let mut parent = SurrealItem::get_by_id(parent.id.to_raw(), db)
        .await
        .unwrap()
        .unwrap();
    if let Some(higher_priority_than_this) = higher_priority_than_this {
        let index_of_higher_priority = parent.smaller_items_in_priority_order
            .iter()
            .position(|x| match x {
                //Note that position() is short-circuiting. If there are multiple matches it could be argued that I should panic or assert but
                //I am just matching the first one and then I just keep going. Because I am still figuring out the design and this is 
                //more in the vein of hardening work I think this is fine but feel free to revisit this.
                SurrealOrderedSubItem::SubItem { surreal_item_id } => {
                    surreal_item_id == &higher_priority_than_this
                }
                SurrealOrderedSubItem::Split { .. } => todo!("I need to understand more about how split will be used before I can implement this"),
            })
            .expect("Should already be in the list");
        parent.smaller_items_in_priority_order.insert(
            index_of_higher_priority,
            SurrealOrderedSubItem::SubItem {
                surreal_item_id: child,
            },
        );
    } else {
        parent
            .smaller_items_in_priority_order
            .push(SurrealOrderedSubItem::SubItem {
                surreal_item_id: child,
            });
    }
    let saved = parent.clone().update(db).await.unwrap().unwrap();
    assert_eq!(parent, saved);
}

async fn parent_item_with_a_new_child(
    child: NewItem,
    parent: RecordId,
    higher_priority_than_this: Option<RecordId>,
    db: &Surreal<Any>,
) {
    let child = new_item(child, db).await;
    parent_item_with_existing_item(
        child.id.expect("In DB"),
        parent,
        higher_priority_than_this,
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

    let parent_surreal_item = SurrealItem::new(parent_new_item, smaller_items_in_priority_order);
    parent_surreal_item.create(db).await.unwrap();
}

async fn update_hope_permanence(
    surreal_item: RecordId,
    new_permanence: Permanence,
    db: &Surreal<Any>,
) {
    let mut surreal_item = SurrealItem::get_by_id(surreal_item.id.to_raw(), db)
        .await
        .unwrap()
        .unwrap();
    surreal_item.permanence = new_permanence;

    if surreal_item.id.is_some() {
        //Update
        surreal_item.update(db).await.unwrap();
    } else {
        //Create record
        surreal_item.create(db).await.unwrap();
    }
}

async fn update_hope_staging(record_id: RecordId, new_staging: Staging, db: &Surreal<Any>) {
    let mut surreal_item = SurrealItem::get_by_id(record_id.id.to_raw(), db)
        .await
        .unwrap()
        .unwrap();
    surreal_item.staging = new_staging;

    if surreal_item.id.is_some() {
        let _: SurrealItem = db
            .update((
                SurrealItem::TABLE_NAME,
                surreal_item.get_id().clone().unwrap().id.clone().to_raw(),
            ))
            //I am doing this directly rather than using the update method on the surreal_item type because I need to call content rather than update
            //because I changed the type of Staging::OnDeck to include two parameters and update will silently not update and content will properly
            //do this update. Although in theory content is creating a new record so that might cause more churn if it is not required. I might consider
            //just migrating all records all at once and one time to prevent this need to use content for ever more.
            .content(surreal_item)
            .await
            .unwrap()
            .unwrap();
    } else {
        //Create record
        surreal_item.create(db).await.unwrap();
    }
}

async fn update_item_summary(item_to_update: RecordId, new_summary: String, db: &Surreal<Any>) {
    let mut item_to_update = SurrealItem::get_by_id(item_to_update.id.to_raw(), db)
        .await
        .unwrap()
        .unwrap();
    item_to_update.summary = new_summary;

    item_to_update.update(db).await.unwrap();
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    use crate::{new_item::NewItemBuilder, surrealdb_layer::surreal_item::HowMuchIsInMyControl};

    #[tokio::test]
    async fn data_starts_empty() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert!(surreal_tables.surreal_items.is_empty());
        assert!(surreal_tables.surreal_coverings.is_empty());
        assert!(surreal_tables.surreal_required_circumstances.is_empty());
        assert!(surreal_tables.surreal_coverings_until_date_time.is_empty());

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
            ItemType::Undeclared,
            surreal_tables.surreal_items.first().unwrap().item_type
        );
        assert_eq!(surreal_tables.surreal_coverings.len(), 0);

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
            item.get_id().as_ref().expect("Item exists in DB").clone(),
        )
        .await
        .unwrap();

        assert!(processed_text.is_empty());

        sender
            .send(DataLayerCommands::AddProcessedText(
                "Some user processed text".into(),
                item.get_id().as_ref().expect("Already in DB").clone(),
            ))
            .await
            .unwrap();

        let (processed_text_tx, processed_text_rx) = oneshot::channel();
        let next_step = surreal_tables.surreal_items.first().unwrap();
        sender
            .send(DataLayerCommands::SendProcessedText(
                next_step
                    .get_id()
                    .as_ref()
                    .expect("Item exists in DB")
                    .clone(),
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
            .item_type(ItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(new_next_step))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let items = surreal_tables.make_items();

        assert_eq!(items.len(), 1);
        let next_step_item = items.first().unwrap();
        assert_eq!(next_step_item.is_finished(), false);
        assert_eq!(surreal_tables.surreal_coverings.len(), 0);

        sender
            .send(DataLayerCommands::FinishItem(
                next_step_item.get_id().clone().into(),
            ))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let items = surreal_tables.make_items();

        assert_eq!(items.len(), 1);
        let next_step_item = items.first().unwrap();
        assert_eq!(next_step_item.is_finished(), true);
        assert_eq!(surreal_tables.surreal_coverings.len(), 0);

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
            .item_type(ItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(new_action))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert_eq!(0, surreal_tables.surreal_coverings.len()); //length of zero means nothing is covered
        let item_to_cover = surreal_tables.surreal_items.first().unwrap();

        let new_item = NewItemBuilder::default()
            .summary("Covering item")
            .responsibility(Responsibility::ProactiveActionToTake)
            .item_type(ItemType::Action)
            .build()
            .unwrap();

        sender
            .send(DataLayerCommands::CoverItemWithANewItem {
                cover_this: item_to_cover.get_id().as_ref().expect("In DB").clone(),
                cover_with: new_item,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(2, surreal_tables.surreal_items.len());
        assert_eq!(1, surreal_tables.surreal_coverings.len()); //expect one item to be is covered
        let covering = surreal_tables.surreal_coverings.first().unwrap();
        let item_that_should_be_covered = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "Item to be covered")
            .unwrap();
        let item_that_should_cover = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "Covering item")
            .unwrap();
        assert_eq!(
            item_that_should_be_covered.id.as_ref().unwrap(),
            &covering.parent
        );
        assert_eq!(
            item_that_should_cover.id.as_ref().unwrap(),
            &covering.smaller
        );
        //TODO: Check Order & Responsibility that they are properly set

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
            .item_type(ItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(new_action))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert!(surreal_tables.surreal_coverings_until_date_time.is_empty());

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
        assert_eq!(1, surreal_tables.surreal_coverings_until_date_time.len());
        assert_eq!(
            surreal_tables
                .surreal_items
                .first()
                .unwrap()
                .id
                .as_ref()
                .unwrap(),
            &surreal_tables
                .surreal_coverings_until_date_time
                .first()
                .unwrap()
                .cover_this
        );
        assert_eq!(
            cover_until,
            surreal_tables
                .surreal_coverings_until_date_time
                .first()
                .as_ref()
                .unwrap()
                .until
                .clone()
                .into()
        );

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
            .item_type(ItemType::Action)
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
                    .item_type(ItemType::Goal(HowMuchIsInMyControl::default()))
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
            .item_type(ItemType::Action)
            .build()
            .expect("Filled out required fields");
        sender
            .send(DataLayerCommands::NewItem(item_that_needs_a_parent))
            .await
            .unwrap();

        let parent_item = NewItemBuilder::default()
            .summary("Parent Item")
            .item_type(ItemType::Goal(HowMuchIsInMyControl::default()))
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
                    .get_id()
                    .as_ref()
                    .expect("In DB")
                    .clone(),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
                    .get_id()
                    .as_ref()
                    .expect("In DB")
                    .clone(),
                higher_priority_than_this: None,
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
}
