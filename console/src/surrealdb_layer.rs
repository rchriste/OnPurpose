pub mod surreal_covering;
pub mod surreal_covering_until_date_time;
pub mod surreal_item;
pub mod surreal_requirement;
pub mod surreal_specific_to_hope;

use chrono::{DateTime, Local, Utc};
use surrealdb::{
    engine::any::{connect, Any, IntoEndpoint},
    sql::Thing,
    Surreal,
};
use surrealdb_extra::table::Table;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot::{self, error::RecvError},
};

use crate::base_data::{ItemType, ProcessedText};

use self::{
    surreal_covering::SurrealCovering,
    surreal_covering_until_date_time::SurrealCoveringUntilDatetime,
    surreal_item::SurrealItem,
    surreal_requirement::{RequirementType, SurrealRequirement},
    surreal_specific_to_hope::{Permanence, SurrealSpecificToHope},
};

#[derive(Debug)]
pub struct SurrealTables {
    pub surreal_items: Vec<SurrealItem>,
    pub surreal_specific_to_hopes: Vec<SurrealSpecificToHope>,
    pub surreal_coverings: Vec<SurrealCovering>,
    pub surreal_requirements: Vec<SurrealRequirement>,
    pub surreal_coverings_until_date_time: Vec<SurrealCoveringUntilDatetime>,
}

pub enum DataLayerCommands {
    SendRawData(oneshot::Sender<SurrealTables>),
    AddProcessedText(String, SurrealItem),
    GetProcessedText(SurrealItem, oneshot::Sender<Vec<ProcessedText>>),
    FinishItem(SurrealItem),
    NewToDo(String),
    NewHope(String),
    NewMotivation(String),
    CoverItemWithANewToDo(SurrealItem, String),
    CoverItemWithANewQuestion(SurrealItem, String),
    CoverItemWithANewMilestone(SurrealItem, String),
    CoverItemUntilAnExactDateTime(SurrealItem, DateTime<Utc>),
    AddRequirementNotSunday(SurrealItem),
    UpdateHopePermanence(SurrealSpecificToHope, Permanence),
    UpdateItemSummary(SurrealItem, String),
}

impl DataLayerCommands {
    pub async fn get_raw_data(
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
    pub async fn get_processed_text(
        sender: &Sender<DataLayerCommands>,
        for_item: SurrealItem,
    ) -> Result<Vec<ProcessedText>, RecvError> {
        let (processed_text_tx, processed_text_rx) = oneshot::channel();
        sender
            .send(DataLayerCommands::GetProcessedText(
                for_item,
                processed_text_tx,
            ))
            .await
            .unwrap();
        processed_text_rx.await
    }
}

pub async fn data_storage_start_and_run(
    mut data_storage_layer_receive_rx: Receiver<DataLayerCommands>,
    endpoint: impl IntoEndpoint,
) {
    let db = connect(endpoint).await.unwrap();
    db.use_ns("OnPurpose").use_db("Russ").await.unwrap();

    loop {
        let received = data_storage_layer_receive_rx.recv().await;
        match received {
            Some(DataLayerCommands::SendRawData(oneshot)) => {
                let surreal_tables = load_data_from_surrealdb(&db).await;
                oneshot.send(surreal_tables).unwrap();
            }
            Some(DataLayerCommands::AddProcessedText(processed_text, for_item)) => {
                add_processed_text(processed_text, for_item, &db).await
            }
            Some(DataLayerCommands::GetProcessedText(for_item, send_response_here)) => {
                get_processed_text(for_item, send_response_here, &db).await
            }
            Some(DataLayerCommands::FinishItem(item)) => finish_item(item, &db).await,
            Some(DataLayerCommands::NewToDo(to_do_text)) => new_to_do(to_do_text, &db).await,
            Some(DataLayerCommands::NewHope(hope_text)) => new_hope(hope_text, &db).await,
            Some(DataLayerCommands::NewMotivation(summary_text)) => {
                new_motivation(summary_text, &db).await
            }
            Some(DataLayerCommands::CoverItemWithANewToDo(item_to_cover, new_to_do_text)) => {
                cover_item_with_a_new_next_step(item_to_cover, new_to_do_text, &db).await
            }
            Some(DataLayerCommands::CoverItemWithANewQuestion(item, question)) => {
                cover_item_with_a_new_question(item, question, &db).await
            }
            Some(DataLayerCommands::CoverItemWithANewMilestone(
                item_to_cover,
                new_milestone_text,
            )) => cover_item_with_a_new_milestone(item_to_cover, new_milestone_text, &db).await,
            Some(DataLayerCommands::CoverItemUntilAnExactDateTime(item_to_cover, cover_until)) => {
                cover_item_until_an_exact_date_time(item_to_cover, cover_until, &db).await
            }
            Some(DataLayerCommands::AddRequirementNotSunday(add_requirement_to_this)) => {
                add_requirement_not_sunday(add_requirement_to_this, &db).await
            }
            Some(DataLayerCommands::UpdateHopePermanence(specific_to_hope, permanence)) => {
                update_hope_permanence(specific_to_hope, permanence, &db).await
            }
            Some(DataLayerCommands::UpdateItemSummary(item, new_summary)) => {
                update_item_summary(item, new_summary, &db).await
            }
            None => return, //Channel closed, time to shutdown down, exit
        }
    }
}

pub async fn load_data_from_surrealdb(db: &Surreal<Any>) -> SurrealTables {
    let all_specific_to_hopes = SurrealSpecificToHope::get_all(db);
    let all_items = SurrealItem::get_all(db);
    let all_coverings = SurrealCovering::get_all(db);
    let all_requirements = SurrealRequirement::get_all(db);
    let all_coverings_until_date_time = SurrealCoveringUntilDatetime::get_all(db);
    let all_items = all_items.await.unwrap();
    let all_specific_to_hopes = all_specific_to_hopes.await.unwrap();
    let all_specific_to_hopes = all_items
        .iter()
        .map(|x| {
            match all_specific_to_hopes
                .iter()
                .find(|y| x.id.as_ref().expect("In DB") == &y.for_item)
            {
                Some(s) => s.clone(),
                None => SurrealSpecificToHope::new_defaults(x.id.as_ref().expect("In DB").clone()),
            }
        })
        .collect();
    SurrealTables {
        surreal_items: all_items,
        surreal_coverings: all_coverings.await.unwrap(),
        surreal_requirements: all_requirements.await.unwrap(),
        surreal_coverings_until_date_time: all_coverings_until_date_time.await.unwrap(),
        surreal_specific_to_hopes: all_specific_to_hopes,
    }
}

pub async fn add_processed_text(processed_text: String, for_item: SurrealItem, db: &Surreal<Any>) {
    let for_item: Option<Thing> = for_item.into();
    let data = ProcessedText {
        id: None,
        text: processed_text,
        when_written: Local::now().naive_utc().and_utc().into(),
        for_item: for_item.unwrap(),
    };
    data.create(db).await.unwrap();
}

pub async fn get_processed_text(
    for_item: SurrealItem,
    send_response_here: oneshot::Sender<Vec<ProcessedText>>,
    db: &Surreal<Any>,
) {
    let mut query_result = db
        .query("SELECT * FROM processed_text WHERE for_item = $for_item")
        .bind(("for_item", for_item.id))
        .await
        .unwrap();

    let processed_text: Vec<ProcessedText> = query_result.take(0).unwrap();

    send_response_here.send(processed_text).unwrap();
}

pub async fn finish_item(mut finish_this: SurrealItem, db: &Surreal<Any>) {
    finish_this.finished = Some(Local::now().naive_utc().and_utc().into());
    finish_this.update(db).await.unwrap();
}

async fn new_to_do(to_do_text: String, db: &Surreal<Any>) {
    SurrealItem {
        id: None,
        summary: to_do_text,
        finished: None,
        item_type: ItemType::ToDo,
    }
    .create(db)
    .await
    .unwrap();
}

async fn new_hope(hope_text: String, db: &Surreal<Any>) {
    SurrealItem {
        id: None,
        summary: hope_text,
        finished: None,
        item_type: ItemType::Hope,
    }
    .create(db)
    .await
    .unwrap();
}

async fn new_motivation(motivation_text: String, db: &Surreal<Any>) {
    SurrealItem {
        id: None,
        summary: motivation_text,
        finished: None,
        item_type: ItemType::Motivation,
    }
    .create(db)
    .await
    .unwrap();
}

async fn cover_item_with_a_new_question(item: SurrealItem, question: String, db: &Surreal<Any>) {
    //For now covering an item with a question is the same implementation as just covering with a next step so just call into that
    cover_item_with_a_new_next_step(item, question, db).await
}

async fn cover_item_with_a_new_next_step(
    item_to_cover: SurrealItem,
    new_to_do_text: String,
    db: &Surreal<Any>,
) {
    //Note that both of these things should really be happening inside of a single transaction but I don't know how to do that
    //easily so just do this for now.

    let new_to_do = SurrealItem {
        id: None,
        summary: new_to_do_text,
        finished: None,
        item_type: ItemType::ToDo,
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
    //This would be best done as a single transaction but I am not quite sure how to do that so do it separate for now

    let new_milestone = SurrealItem {
        id: None,
        summary: milestone_text,
        finished: None,
        item_type: ItemType::Hope,
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

async fn add_requirement_not_sunday(item: SurrealItem, db: &Surreal<Any>) {
    SurrealRequirement {
        id: None,
        requirement_for: item.id.unwrap(),
        requirement_type: RequirementType::NotSunday,
    }
    .create(db)
    .await
    .unwrap();
}

async fn update_hope_permanence(
    mut surreal_specific_to_hope: SurrealSpecificToHope,
    new_permanence: Permanence,
    db: &Surreal<Any>,
) {
    surreal_specific_to_hope.permanence = new_permanence;

    if surreal_specific_to_hope.id.is_some() {
        //Update
        surreal_specific_to_hope.update(db).await.unwrap();
    } else {
        //Create record
        surreal_specific_to_hope.create(db).await.unwrap();
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

    use crate::surrealdb_layer::surreal_item::SurrealItemVecExtensions;

    use super::*;

    #[tokio::test]
    async fn data_starts_empty() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert!(surreal_tables.surreal_items.is_empty());
        assert!(surreal_tables.surreal_coverings.is_empty());
        assert!(surreal_tables.surreal_requirements.is_empty());
        assert!(surreal_tables.surreal_coverings_until_date_time.is_empty());

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn add_new_todo() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        sender
            .send(DataLayerCommands::NewToDo("New next step".into()))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(surreal_tables.surreal_items.len(), 1);
        assert_eq!(
            ItemType::ToDo,
            surreal_tables.surreal_items.first().unwrap().item_type
        );
        assert_eq!(surreal_tables.surreal_coverings.len(), 0);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn add_new_hope() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        sender
            .send(DataLayerCommands::NewHope("New hope".into()))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(surreal_tables.surreal_items.len(), 1);
        assert_eq!(
            ItemType::Hope,
            surreal_tables.surreal_items.first().unwrap().item_type
        );
        assert_eq!(surreal_tables.surreal_coverings.len(), 0);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn add_new_motivation() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        sender
            .send(DataLayerCommands::NewMotivation("New motivation".into()))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(surreal_tables.surreal_items.len(), 1);
        assert_eq!(
            ItemType::Motivation,
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

        sender
            .send(DataLayerCommands::NewToDo("New next step".into()))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

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
            .send(DataLayerCommands::GetProcessedText(
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

        sender
            .send(DataLayerCommands::NewToDo("New next step".into()))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();
        let items = surreal_tables
            .surreal_items
            .make_items(&surreal_tables.surreal_requirements);

        assert_eq!(items.len(), 1);
        let next_step_item = items.first().unwrap();
        assert_eq!(next_step_item.is_finished(), false);
        assert_eq!(surreal_tables.surreal_coverings.len(), 0);

        sender
            .send(DataLayerCommands::FinishItem(next_step_item.clone().into()))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();
        let items = surreal_tables
            .surreal_items
            .make_items(&surreal_tables.surreal_requirements);

        assert_eq!(items.len(), 1);
        let next_step_item = items.first().unwrap();
        assert_eq!(next_step_item.is_finished(), true);
        assert_eq!(surreal_tables.surreal_coverings.len(), 0);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn cover_item_with_a_new_next_step() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        sender
            .send(DataLayerCommands::NewToDo("Item to be covered".into()))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert_eq!(0, surreal_tables.surreal_coverings.len()); //length of zero means nothing is covered
        let item_to_cover = surreal_tables.surreal_items.first().unwrap();

        sender
            .send(DataLayerCommands::CoverItemWithANewToDo(
                item_to_cover.clone(),
                "Covering item".into(),
            ))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

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
    async fn cover_item_with_a_question() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        sender
            .send(DataLayerCommands::NewToDo("Item to be covered".into()))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert_eq!(0, surreal_tables.surreal_coverings.len()); //length of zero means nothing is covered
        let item_to_cover = surreal_tables.surreal_items.first().unwrap();

        sender
            .send(DataLayerCommands::CoverItemWithANewQuestion(
                item_to_cover.clone(),
                "Covering item".into(),
            ))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

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

        sender
            .send(DataLayerCommands::NewHope("Hope to be covered".into()))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

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

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

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

        sender
            .send(DataLayerCommands::NewToDo("Item to get covered".into()))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

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

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

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

        sender
            .send(DataLayerCommands::NewToDo("Item to get requirement".into()))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert!(surreal_tables.surreal_requirements.is_empty());

        sender
            .send(DataLayerCommands::AddRequirementNotSunday(
                surreal_tables.surreal_items.into_iter().next().unwrap(),
            ))
            .await
            .unwrap();

        let surreal_tables = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(1, surreal_tables.surreal_items.len());
        assert_eq!(1, surreal_tables.surreal_requirements.len());
        assert_eq!(
            RequirementType::NotSunday,
            surreal_tables
                .surreal_requirements
                .first()
                .unwrap()
                .requirement_type
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
                .surreal_requirements
                .first()
                .unwrap()
                .requirement_for
        );

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }
}
