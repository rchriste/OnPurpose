use surrealdb::{Surreal, engine::any::{Any, connect}, sql::Thing};
use surrealdb_extra::table::Table;
use tokio::sync::{mpsc::Receiver, oneshot};
use chrono::Local;

use crate::{test_data::TestData, base_data::{LinkageWithRecordIds, ReviewItem, ReasonItem, NextStepItem, ProcessedText, ItemOwning}};

pub enum DataLayerCommands {
    SendRawData(oneshot::Sender<(TestData, Vec<LinkageWithRecordIds>)>),
    AddUserProcessedText(String, NextStepItem),
    FinishNextStepItem(NextStepItem),
    NewNextStep(String),
    CoverItemWithQuestion(ItemOwning, String),
}

pub async fn data_storage_start_and_run(mut data_storage_layer_receive_rx: Receiver<DataLayerCommands>) {
    let db = connect("file:://~/.on_purpose.db").await.unwrap();
    db.use_ns("OnPurpose").use_db("Russ").await.unwrap();

    loop {
        let received = data_storage_layer_receive_rx.recv().await;
        match received {
            Some(DataLayerCommands::SendRawData(oneshot)) => {
                let (test_data, linkage) = load_data_from_surrealdb(&db).await;
                oneshot.send((test_data, linkage)).unwrap();
            },
            Some(DataLayerCommands::AddUserProcessedText(processed_text, for_item)) => add_user_processed_text(processed_text, for_item, &db).await,
            Some(DataLayerCommands::FinishNextStepItem(next_step_item)) => finish_text_step_item(next_step_item, &db).await,
            Some(DataLayerCommands::NewNextStep(next_step_text)) => new_next_step(next_step_text, &db).await,
            Some(DataLayerCommands::CoverItemWithQuestion(item, question)) => cover_item_with_question(item, question, &db).await,
            None => { return } //Channel closed, time to shutdown down, exit
        }
    }
}

pub async fn load_data_from_surrealdb(db: &Surreal<Any>) -> (TestData, Vec<LinkageWithRecordIds>) {
    (
        TestData {
            next_steps: NextStepItem::get_all(&db).await.unwrap(),
            review_items: ReviewItem::get_all(&db).await.unwrap(),
            reason_items: ReasonItem::get_all(&db).await.unwrap(),
        },
        LinkageWithRecordIds::get_all(&db).await.unwrap()
    )
}

pub async fn add_user_processed_text(processed_text: String, for_item: NextStepItem, db: &Surreal<Any>) {
    let data = ProcessedText {
        id: None,
        text: processed_text,
        when_written: Local::now().naive_utc().and_utc().into(),
        for_item: for_item.id.unwrap()
    };
    data.create(db).await.unwrap();
}

pub async fn finish_text_step_item(mut finish_this: NextStepItem, db: &Surreal<Any>) {
    finish_this.finished = Some(Local::now().naive_utc().and_utc().into());
    finish_this.update(db).await.unwrap(); 
}

async fn new_next_step(next_step_text: String, db: &Surreal<Any>) {
    NextStepItem {
        id: None,
        summary: next_step_text,
        finished: None,
    }.create(db).await.unwrap();
}

async fn cover_item_with_question(item: ItemOwning, question: String, db: &Surreal<Any>) {
    let question = NextStepItem {
        id: None,
        summary: question,
        finished: None,
    }.create(&db).await.unwrap().into_iter().next().unwrap();

    let smaller_option: Option<Thing> = question.into();
    let parent_option: Option<Thing> = item.into();
    LinkageWithRecordIds {
        id: None,
        smaller: smaller_option.unwrap(),
        parent: parent_option.unwrap(),
    }.create(&db).await.unwrap().first().unwrap();
}