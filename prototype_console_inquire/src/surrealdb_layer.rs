use surrealdb::{Surreal, engine::any::Any};
use surrealdb_extra::table::Table;

use crate::{test_data::TestData, base_data::{LinkageWithRecordIds, ReviewItem, ReasonItem, NextStepItem}};


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