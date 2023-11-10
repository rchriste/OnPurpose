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
        ItemType, NotesLocation, Permanence, Responsibility, Staging, SurrealItem,
        SurrealItemOldVersion, SurrealOrderedSubItem,
    },
    surreal_life_area::SurrealLifeArea,
    surreal_processed_text::SurrealProcessedText,
    surreal_required_circumstance::{CircumstanceType, SurrealRequiredCircumstance},
    surreal_routine::SurrealRoutine,
    surreal_tables::SurrealTables,
};

pub(crate) enum DataLayerCommands {
    SendRawData(oneshot::Sender<SurrealTables>),
    SendProcessedText(SurrealItem, oneshot::Sender<Vec<SurrealProcessedText>>),
    AddProcessedText(String, SurrealItem),
    FinishItem(SurrealItem),
    NewItem(NewItem),
    CoverItemWithANewItem {
        cover_this: SurrealItem,
        cover_with: NewItem,
    },
    CoverItemWithANewWaitingForQuestion(SurrealItem, String),
    CoverItemWithANewMilestone(SurrealItem, String),
    CoverItemWithAnExistingItem {
        item_to_be_covered: SurrealItem,
        item_that_should_do_the_covering: SurrealItem,
    },
    #[allow(dead_code)]
    //This was initially added for data migration that is now removed but I expect to want it again in the future
    RemoveCoveringItem(SurrealCovering),
    CoverItemUntilAnExactDateTime(SurrealItem, DateTime<Utc>),
    ParentItemWithExistingItem {
        child: SurrealItem,
        parent: SurrealItem,
        higher_priority_than_this: Option<SurrealItem>,
    },
    ParentItemWithANewChildItem {
        child: NewItem,
        parent: SurrealItem,
        higher_priority_than_this: Option<SurrealItem>,
    },
    ParentNewItemWithAnExistingChildItem {
        child: SurrealItem,
        parent_new_item: NewItem,
    },
    AddCircumstanceNotSunday(SurrealItem),
    AddCircumstanceDuringFocusTime(SurrealItem),
    UpdateResponsibilityAndItemType(SurrealItem, Responsibility, ItemType),
    UpdateItemResponsibility(SurrealItem, Responsibility),
    UpdateItemPermanence(SurrealItem, Permanence),
    UpdateItemStaging(SurrealItem, Staging),
    UpdateItemSummary(SurrealItem, String),
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
        for_item: SurrealItem,
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
            Some(DataLayerCommands::CoverItemWithANewWaitingForQuestion(item, question)) => {
                cover_item_with_a_new_waiting_for_question(item, question, &db).await
            }
            Some(DataLayerCommands::CoverItemWithANewMilestone(
                item_to_cover,
                new_milestone_text,
            )) => cover_item_with_a_new_milestone(item_to_cover, new_milestone_text, &db).await,
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
            Some(DataLayerCommands::AddCircumstanceNotSunday(add_circumstance_to_this)) => {
                add_circumstance_not_sunday(add_circumstance_to_this, &db).await
            }
            Some(DataLayerCommands::AddCircumstanceDuringFocusTime(add_circumstance_to_this)) => {
                add_circumstance_during_focus_time(add_circumstance_to_this, &db).await
            }
            Some(DataLayerCommands::UpdateItemPermanence(item, new_permanence)) => {
                update_hope_permanence(item, new_permanence, &db).await
            }
            Some(DataLayerCommands::UpdateItemStaging(item, new_staging)) => {
                update_hope_staging(item, new_staging, &db).await
            }
            Some(DataLayerCommands::UpdateItemSummary(item, new_summary)) => {
                update_item_summary(item, new_summary, &db).await
            }
            Some(DataLayerCommands::UpdateResponsibilityAndItemType(
                item,
                new_responsibility,
                new_item_type,
            )) => {
                let mut item = item;
                item.responsibility = new_responsibility;
                item.item_type = new_item_type;
                item.update(&db).await.unwrap();
            }
            Some(DataLayerCommands::UpdateItemResponsibility(item, new_responsibility)) => {
                let mut item = item;
                item.responsibility = new_responsibility;
                item.update(&db).await.unwrap();
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
    for_item: SurrealItem,
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
    for_item: SurrealItem,
    send_response_here: oneshot::Sender<Vec<SurrealProcessedText>>,
    db: &Surreal<Any>,
) {
    let mut query_result = db
        .query("SELECT * FROM processed_text WHERE for_item = $for_item")
        .bind(("for_item", for_item.id))
        .await
        .unwrap();

    let processed_text: Vec<SurrealProcessedText> = query_result.take(0).unwrap();

    send_response_here.send(processed_text).unwrap();
}

pub(crate) async fn finish_item(mut finish_this: SurrealItem, db: &Surreal<Any>) {
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

async fn cover_item_with_a_new_waiting_for_question(
    item: SurrealItem,
    question: String,
    db: &Surreal<Any>,
) {
    //TODO: Cause this to be a Waiting For Responsibility o
    //For now covering an item with a question is the same implementation as just covering with a next step so just call into that
    cover_item_with_a_new_next_step(item, question, Responsibility::WaitingFor, db).await
}

async fn cover_with_a_new_item(cover_this: SurrealItem, cover_with: NewItem, db: &Surreal<Any>) {
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

async fn cover_item_with_a_new_next_step(
    item_to_cover: SurrealItem,
    new_to_do_text: String,
    responsibility: Responsibility,
    db: &Surreal<Any>,
) {
    //Note that both of these things should really be happening inside of a single transaction but I don't know how to do that
    //easily so just do this for now.

    let new_to_do = SurrealItem {
        id: None,
        summary: new_to_do_text,
        finished: None,
        item_type: ItemType::ToDo,
        smaller_items_in_priority_order: Vec::default(),
        responsibility: responsibility.clone(),
        notes_location: NotesLocation::default(),
        permanence: Permanence::default(),
        staging: Staging::default(),
    }
    .create(db)
    .await
    .unwrap()
    .into_iter()
    .next()
    .unwrap();

    let smaller_option: Option<Thing> = new_to_do.into();
    let parent_option: Option<Thing> = item_to_cover.into();
    SurrealCovering {
        id: None,
        smaller: smaller_option.expect("Should already be in the database"),
        parent: parent_option.expect("Should already be in the database"),
    }
    .create(db)
    .await
    .unwrap();
}

async fn cover_item_with_a_new_milestone(
    item_to_cover: SurrealItem,
    milestone_text: String,
    db: &Surreal<Any>,
) {
    //TODO: This should be removed. I am not ever doing anything to encode the idea of milestone into the saved data.
    //This would be best done as a single transaction but I am not quite sure how to do that so do it separate for now

    let new_milestone = SurrealItem {
        id: None,
        summary: milestone_text,
        finished: None,
        item_type: ItemType::Hope,
        smaller_items_in_priority_order: Vec::default(),
        responsibility: Responsibility::default(),
        notes_location: NotesLocation::default(),
        permanence: Permanence::default(),
        staging: Staging::default(),
    }
    .create(db)
    .await
    .unwrap()
    .into_iter()
    .next()
    .unwrap();

    let smaller_option: Option<Thing> = new_milestone.into();
    let parent_option: Option<Thing> = item_to_cover.into();
    SurrealCovering {
        id: None,
        smaller: smaller_option.expect("Should already be in the database"),
        parent: parent_option.expect("Should already be in the database"),
    }
    .create(db)
    .await
    .unwrap();
}

async fn cover_item_with_an_existing_item(
    existing_item_to_be_covered: SurrealItem,
    existing_item_that_is_doing_the_covering: SurrealItem,
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
    item_to_cover: SurrealItem,
    cover_until: DateTime<Utc>,
    db: &Surreal<Any>,
) {
    SurrealCoveringUntilDatetime {
        id: None,
        cover_this: item_to_cover.id.expect("Should already be in database"),
        until: cover_until.into(),
    }
    .create(db)
    .await
    .unwrap();
}

async fn add_circumstance_not_sunday(item: SurrealItem, db: &Surreal<Any>) {
    add_circumstance(item, CircumstanceType::NotSunday, db).await
}

async fn add_circumstance_during_focus_time(item: SurrealItem, db: &Surreal<Any>) {
    add_circumstance(item, CircumstanceType::DuringFocusTime, db).await
}

async fn add_circumstance(item: SurrealItem, circumstance: CircumstanceType, db: &Surreal<Any>) {
    SurrealRequiredCircumstance {
        id: None,
        required_for: item.id.unwrap(),
        circumstance_type: circumstance,
    }
    .create(db)
    .await
    .unwrap();
}

async fn parent_item_with_existing_item(
    child: SurrealItem,
    mut parent: SurrealItem,
    higher_priority_than_this: Option<SurrealItem>,
    db: &Surreal<Any>,
) {
    if let Some(higher_priority_than_this) = higher_priority_than_this {
        let higher_priority_than_this = higher_priority_than_this.id.expect("Already in DB");
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
                surreal_item_id: child.id.expect("Already in DB"),
            },
        );
    } else {
        parent
            .smaller_items_in_priority_order
            .push(SurrealOrderedSubItem::SubItem {
                surreal_item_id: child.id.expect("Already in DB"),
            });
    }
    parent.update(db).await.unwrap();
}

async fn parent_item_with_a_new_child(
    child: NewItem,
    parent: SurrealItem,
    higher_priority_than_this: Option<SurrealItem>,
    db: &Surreal<Any>,
) {
    let child = new_item(child, db).await;
    parent_item_with_existing_item(child, parent, higher_priority_than_this, db).await
}

async fn parent_new_item_with_an_existing_child_item(
    child: SurrealItem,
    parent_new_item: NewItem,
    db: &Surreal<Any>,
) {
    //TODO: Write a Unit Test for this
    let smaller_items_in_priority_order = vec![SurrealOrderedSubItem::SubItem {
        surreal_item_id: child.id.expect("Already in DB"),
    }];

    let parent_surreal_item = SurrealItem::new(parent_new_item, smaller_items_in_priority_order);
    parent_surreal_item.create(db).await.unwrap();
}

async fn update_hope_permanence(
    mut surreal_item: SurrealItem,
    new_permanence: Permanence,
    db: &Surreal<Any>,
) {
    surreal_item.permanence = new_permanence;

    if surreal_item.id.is_some() {
        //Update
        surreal_item.update(db).await.unwrap();
    } else {
        //Create record
        surreal_item.create(db).await.unwrap();
    }
}

async fn update_hope_staging(
    mut surreal_item: SurrealItem,
    new_staging: Staging,
    db: &Surreal<Any>,
) {
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

async fn update_item_summary(
    mut item_to_update: SurrealItem,
    new_summary: String,
    db: &Surreal<Any>,
) {
    item_to_update.summary = new_summary;

    item_to_update.update(db).await.unwrap();
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    use crate::new_item::NewItemBuilder;

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

        let new_item = NewItem::new("New item".into());
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

        let new_action = NewItem::new_action("New next step".into());
        sender
            .send(DataLayerCommands::NewItem(new_action))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        let item = surreal_tables.surreal_items.first().unwrap();
        let processed_text = DataLayerCommands::get_processed_text(&sender, item.clone())
            .await
            .unwrap();

        assert!(processed_text.is_empty());

        sender
            .send(DataLayerCommands::AddProcessedText(
                "Some user processed text".into(),
                item.clone(),
            ))
            .await
            .unwrap();

        let (processed_text_tx, processed_text_rx) = oneshot::channel();
        let next_step = surreal_tables.surreal_items.first().unwrap();
        sender
            .send(DataLayerCommands::SendProcessedText(
                next_step.clone(),
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

        let new_next_step = NewItem::new_action("New next step".into());
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
            .send(DataLayerCommands::FinishItem(next_step_item.clone().into()))
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

        let new_action = NewItem::new_action("Item to be covered".into());
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
            .item_type(ItemType::ToDo)
            .build()
            .unwrap();

        sender
            .send(DataLayerCommands::CoverItemWithANewItem {
                cover_this: item_to_cover.clone(),
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
    async fn cover_item_with_a_question() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_action = NewItem::new_action("Item to be covered".into());
        sender
            .send(DataLayerCommands::NewItem(new_action))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert_eq!(0, surreal_tables.surreal_coverings.len()); //length of zero means nothing is covered
        let item_to_cover = surreal_tables.surreal_items.first().unwrap();

        sender
            .send(DataLayerCommands::CoverItemWithANewWaitingForQuestion(
                item_to_cover.clone(),
                "Covering item".into(),
            ))
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

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn cover_item_with_a_new_milestone() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_goal = NewItem::new_goal("Hope to be covered".into());
        sender
            .send(DataLayerCommands::NewItem(new_goal))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert_eq!(0, surreal_tables.surreal_coverings.len()); //length of zero means nothing is covered
        let item_to_cover = surreal_tables.surreal_items.first().unwrap();

        sender
            .send(DataLayerCommands::CoverItemWithANewMilestone(
                item_to_cover.clone(),
                "Covering milestone".into(),
            ))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(2, surreal_tables.surreal_items.len());
        assert_eq!(1, surreal_tables.surreal_coverings.len()); //expect one item to be is covered
        let covering = surreal_tables.surreal_coverings.first().unwrap();
        let item_that_should_be_covered = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "Hope to be covered")
            .unwrap();
        let item_that_should_cover = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "Covering milestone")
            .unwrap();
        assert_eq!(
            item_that_should_be_covered.id.as_ref().unwrap(),
            &covering.parent
        );
        assert_eq!(
            item_that_should_cover.id.as_ref().unwrap(),
            &covering.smaller
        );

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn cover_item_until_an_exact_date_time() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_action = NewItem::new_action("Item to be covered".into());
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
                surreal_tables.surreal_items.into_iter().next().unwrap(),
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
    async fn cover_item_with_the_requirement_not_sunday() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new_action("Item to get requirement".into());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert!(surreal_tables.surreal_required_circumstances.is_empty());

        sender
            .send(DataLayerCommands::AddCircumstanceNotSunday(
                surreal_tables.surreal_items.into_iter().next().unwrap(),
            ))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert_eq!(1, surreal_tables.surreal_required_circumstances.len());
        assert_eq!(
            CircumstanceType::NotSunday,
            surreal_tables
                .surreal_required_circumstances
                .first()
                .unwrap()
                .circumstance_type
        );
        assert_eq!(
            surreal_tables
                .surreal_items
                .first()
                .unwrap()
                .id
                .as_ref()
                .unwrap(),
            &surreal_tables
                .surreal_required_circumstances
                .first()
                .unwrap()
                .required_for
        );

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_item_with_a_new_item() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_action = NewItem::new_action("Item that needs a parent".into());
        sender
            .send(DataLayerCommands::NewItem(new_action))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());

        sender
            .send(DataLayerCommands::ParentNewItemWithAnExistingChildItem {
                child: surreal_tables.surreal_items.into_iter().next().unwrap(),
                parent_new_item: NewItemBuilder::default()
                    .summary("Parent Item")
                    .item_type(ItemType::Hope)
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

        let item_that_needs_a_parent = NewItem::new_action("Item that needs a parent".into());
        sender
            .send(DataLayerCommands::NewItem(item_that_needs_a_parent))
            .await
            .unwrap();

        let parent_item = NewItem::new_action("Parent Item".into());
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
                    .clone(),
                parent: surreal_tables
                    .surreal_items
                    .iter()
                    .find(|x| x.summary == "Parent Item")
                    .unwrap()
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
