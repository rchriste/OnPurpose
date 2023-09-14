use surrealdb::{engine::any::Any, Surreal};
use surrealdb_extra::table::Table;

use crate::base_data::{NextStepItem, ReviewItem, ReasonItem, Linkage, Item};


pub struct TestData {
    pub next_steps: Vec<NextStepItem>,
    pub review_items: Vec<ReviewItem>,
    pub reason_items: Vec<ReasonItem>,
}

pub fn create_items() -> TestData
{
    let next_steps = vec![
        NextStepItem {
            id: None,
            summary: String::from("Clean Dometic")
        },
        NextStepItem {
            id: None,
            summary: String::from("Fill out SafeAccess Health & Safety Invitation for RustConf 2023")
        },
        NextStepItem {
            id: None,
            summary: String::from("Get a Covid vaccine")
        },
    ];

    let review_items = vec![
        ReviewItem {
            id: None,
            summary: String::from("Go camping")
        },
        ReviewItem {
            id: None,
            summary: String::from("After")
        },

        ReviewItem {
            id: None,
            summary: String::from("Attend Rust conference")
        },
        ReviewItem {
            id: None,
            summary: String::from("Prepare")
        }
    ];

    let reason_items = vec![
        ReasonItem {
            id: None,
            summary: String::from("Family Trips")
        },
        ReasonItem {
            id: None,
            summary: String::from("On-Purpose")
        }
    ];

    TestData { next_steps, review_items, reason_items }
}

pub async fn upload_test_data_to_surrealdb(test_data: TestData, db: &Surreal<Any>) -> TestData
{
    let mut next_steps_surreal = Vec::with_capacity(test_data.next_steps.capacity());
    for x in test_data.next_steps.into_iter()
    {
        let r = x.create(&db).await.unwrap();
        next_steps_surreal.extend(r.into_iter());
    }

    let mut review_items_surreal = Vec::with_capacity(test_data.review_items.capacity());
    for x in test_data.review_items.into_iter()
    {
        let r = x.create(&db).await.unwrap();
        review_items_surreal.extend(r.into_iter());
    }

    let mut reason_items_surreal = Vec::with_capacity(test_data.reason_items.capacity());
    for x in test_data.reason_items.into_iter()
    {
        let r = x.create(&db).await.unwrap();
        reason_items_surreal.extend(r.into_iter());
    }

    TestData {
        next_steps: next_steps_surreal,
        review_items: review_items_surreal,
        reason_items: reason_items_surreal,
    }
}

pub fn create_linkage<'a>(test_data: &'a TestData) -> Vec<Linkage<'a>>
{
    let linkage = vec![
        //NEXT STEPS
        Linkage {
            parent: Item::NextStepItem(&test_data.next_steps[1]),
            smaller: Item::NextStepItem(&test_data.next_steps[2]),
        },
        //NEXT STEPS to REVIEW ITEMS
        Linkage {
            parent: Item::ReviewItem(&test_data.review_items[1]),
            smaller: Item::NextStepItem(&test_data.next_steps[0]),
        },
        Linkage {
            parent: Item::ReviewItem(&test_data.review_items[0]),
            smaller: Item::ReviewItem(&test_data.review_items[1])
        },
        Linkage {
            parent: Item::ReviewItem(&test_data.review_items[2]),
            smaller: Item::ReviewItem(&test_data.review_items[3]),
        },
        Linkage {
            parent: Item::ReviewItem(&test_data.review_items[3]),
            smaller: Item::NextStepItem(&test_data.next_steps[1]),
        },
        //REVIEW STEPS to REASONS
        Linkage {
            parent: Item::ReasonItem(&test_data.reason_items[0]),
            smaller: Item::ReviewItem(&test_data.review_items[0]),
        },
        Linkage {
            parent: Item::ReasonItem(&test_data.reason_items[1]),
            smaller: Item::ReviewItem(&test_data.review_items[2]),
        },
    ];

    linkage
}