use chrono::Local;
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

use crate::base_data::{
    ItemType, ProcessedText, RequirementType, SurrealCovering, SurrealItem, SurrealRequirement,
};

pub enum DataLayerCommands {
    SendRawData(
        oneshot::Sender<(
            Vec<SurrealItem>,
            Vec<SurrealCovering>,
            Vec<SurrealRequirement>,
        )>,
    ),
    AddProcessedText(String, SurrealItem),
    #[allow(dead_code)] //This can be removed after this is used beyond the unit tests
    GetProcessedText(SurrealItem, oneshot::Sender<Vec<ProcessedText>>),
    FinishItem(SurrealItem),
    NewToDo(String),
    NewHope(String),
    NewReason(String),
    CoverItemWithANewToDo(SurrealItem, String),
    CoverItemWithAQuestion(SurrealItem, String),
    AddRequirementNotSunday(SurrealItem),
}

impl DataLayerCommands {
    pub async fn get_raw_data(
        sender: &Sender<DataLayerCommands>,
    ) -> Result<
        (
            Vec<SurrealItem>,
            Vec<SurrealCovering>,
            Vec<SurrealRequirement>,
        ),
        RecvError,
    > {
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
                let (items, linkage, requirements) = load_data_from_surrealdb(&db).await;
                oneshot.send((items, linkage, requirements)).unwrap();
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
            Some(DataLayerCommands::NewReason(reason_text)) => new_reason(reason_text, &db).await,
            Some(DataLayerCommands::CoverItemWithANewToDo(item_to_cover, new_to_do_text)) => {
                cover_item_with_new_next_step(item_to_cover, new_to_do_text, &db).await
            }
            Some(DataLayerCommands::CoverItemWithAQuestion(item, question)) => {
                cover_item_with_question(item, question, &db).await
            }
            Some(DataLayerCommands::AddRequirementNotSunday(add_requirement_to_this)) => {
                add_requirement_not_sunday(add_requirement_to_this, &db).await
            }
            None => return, //Channel closed, time to shutdown down, exit
        }
    }
}

pub async fn load_data_from_surrealdb(
    db: &Surreal<Any>,
) -> (
    Vec<SurrealItem>,
    Vec<SurrealCovering>,
    Vec<SurrealRequirement>,
) {
    let all_items = SurrealItem::get_all(db);
    let all_coverings = SurrealCovering::get_all(db);
    let all_requirements = SurrealRequirement::get_all(db);
    (
        all_items.await.unwrap(),
        all_coverings.await.unwrap(),
        all_requirements.await.unwrap(),
    )
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

async fn new_reason(reason_text: String, db: &Surreal<Any>) {
    SurrealItem {
        id: None,
        summary: reason_text,
        finished: None,
        item_type: ItemType::Reason,
    }
    .create(db)
    .await
    .unwrap();
}

async fn cover_item_with_question(item: SurrealItem, question: String, db: &Surreal<Any>) {
    //For now covering an item with a question is the same implementation as just covering with a next step so just call into that
    cover_item_with_new_next_step(item, question, db).await
}

async fn cover_item_with_new_next_step(
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
        smaller: smaller_option.unwrap(),
        parent: parent_option.unwrap(),
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

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use crate::base_data::SurrealItemVecExtensions;

    use super::*;

    #[tokio::test]
    async fn data_starts_empty() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let (items, linkage, requirements) =
            DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert!(items.is_empty());
        assert!(linkage.is_empty());
        assert!(requirements.is_empty());

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

        let (items, linkage, _) = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(ItemType::ToDo, items.first().unwrap().item_type);
        assert_eq!(linkage.len(), 0);

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

        let (items, linkage, _) = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(ItemType::Hope, items.first().unwrap().item_type);
        assert_eq!(linkage.len(), 0);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn add_new_reason() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        sender
            .send(DataLayerCommands::NewReason("New reason".into()))
            .await
            .unwrap();

        let (items, linkage, _) = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(ItemType::Reason, items.first().unwrap().item_type);
        assert_eq!(linkage.len(), 0);

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

        let (items, _, _) = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        let item = items.first().unwrap();
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
        let next_step = items.first().unwrap();
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

        let (items, linkage, requirements) =
            DataLayerCommands::get_raw_data(&sender).await.unwrap();
        let items = items.make_items(&requirements);

        assert_eq!(items.len(), 1);
        let next_step_item = items.first().unwrap();
        assert_eq!(next_step_item.is_finished(), false);
        assert_eq!(linkage.len(), 0);

        sender
            .send(DataLayerCommands::FinishItem(next_step_item.clone().into()))
            .await
            .unwrap();

        let (items, linkage, requirements) =
            DataLayerCommands::get_raw_data(&sender).await.unwrap();
        let items = items.make_items(&requirements);

        assert_eq!(items.len(), 1);
        let next_step_item = items.first().unwrap();
        assert_eq!(next_step_item.is_finished(), true);
        assert_eq!(linkage.len(), 0);

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

        let (items, linkage, _) = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(1, items.len());
        assert_eq!(0, linkage.len()); //length of zero means nothing is covered
        let item_to_cover = items.first().unwrap();

        sender
            .send(DataLayerCommands::CoverItemWithANewToDo(
                item_to_cover.clone(),
                "Covering item".into(),
            ))
            .await
            .unwrap();

        let (items, linkage, _) = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(2, items.len());
        assert_eq!(1, linkage.len()); //expect one item to be is covered
        let covering = linkage.first().unwrap();
        let item_that_should_be_covered = items
            .iter()
            .find(|x| x.summary == "Item to be covered")
            .unwrap();
        let item_that_should_cover = items.iter().find(|x| x.summary == "Covering item").unwrap();
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

        let (test_data, linkage, _) = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(1, test_data.len());
        assert_eq!(0, linkage.len()); //length of zero means nothing is covered
        let item_to_cover = test_data.first().unwrap();

        sender
            .send(DataLayerCommands::CoverItemWithAQuestion(
                item_to_cover.clone(),
                "Covering item".into(),
            ))
            .await
            .unwrap();

        let (test_data, linkage, _) = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(2, test_data.len());
        assert_eq!(1, linkage.len()); //expect one item to be is covered
        let covering = linkage.first().unwrap();
        let item_that_should_be_covered = test_data
            .iter()
            .find(|x| x.summary == "Item to be covered")
            .unwrap();
        let item_that_should_cover = test_data
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
    async fn cover_item_with_the_requirement_not_sunday() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        sender
            .send(DataLayerCommands::NewToDo("Item to get requirement".into()))
            .await
            .unwrap();

        let (items, _, requirements) = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(1, items.len());
        assert!(requirements.is_empty());

        sender
            .send(DataLayerCommands::AddRequirementNotSunday(
                items.into_iter().next().unwrap(),
            ))
            .await
            .unwrap();

        let (items, _, requirements) = DataLayerCommands::get_raw_data(&sender).await.unwrap();

        assert_eq!(1, items.len());
        assert_eq!(1, requirements.len());
        assert_eq!(
            RequirementType::NotSunday,
            requirements.first().unwrap().requirement_type
        );
        assert_eq!(
            items.first().unwrap().id.as_ref().unwrap(),
            &requirements.first().unwrap().requirement_for
        );

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }
}
