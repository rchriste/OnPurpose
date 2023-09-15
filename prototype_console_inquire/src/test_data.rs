use surrealdb::{Surreal, opt::RecordId};
use surrealdb_extra::table::Table;

use crate::base_data::{NextStepItem, ReviewItem, ReasonItem, LinkageWithReferences, LinkageWithRecordIds, Item};


pub struct TestData {
    pub next_steps: Vec<NextStepItem>,
    pub review_items: Vec<ReviewItem>,
    pub reason_items: Vec<ReasonItem>,
}

impl TestData {
    pub fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<Item<'a>>
    {
        if let Some(found) = self.next_steps.iter().find(|x| {
            match x.get_id() {
                Some(v) => v == record_id,
                None => false
            }
        }) {
            Some(Item::NextStepItem(found))
        } else if let Some(found) = self.review_items.iter().find(|x| {
            match x.get_id() {
                Some(v) => v == record_id,
                None => false
            }
        }) {
            Some(Item::ReviewItem(found))
        } else if let Some(found) = self.reason_items.iter().find(|x| {
            match x.get_id() {
                Some(v) => v == record_id,
                None => false
            }
        }) {
            Some(Item::ReasonItem(found))
        } else { None }
    }
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

pub async fn upload_test_data_to_surrealdb<T: surrealdb::Connection>(test_data: TestData, db: &Surreal<T>) -> TestData
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

pub fn create_linkage<'a>(test_data: &'a TestData) -> Vec<LinkageWithReferences<'a>>
{
    let linkage = vec![
        //NEXT STEPS
        LinkageWithReferences {
            parent: Item::NextStepItem(&test_data.next_steps[1]),
            smaller: Item::NextStepItem(&test_data.next_steps[2]),
        },
        //NEXT STEPS to REVIEW ITEMS
        LinkageWithReferences {
            parent: Item::ReviewItem(&test_data.review_items[1]),
            smaller: Item::NextStepItem(&test_data.next_steps[0]),
        },
        LinkageWithReferences {
            parent: Item::ReviewItem(&test_data.review_items[0]),
            smaller: Item::ReviewItem(&test_data.review_items[1])
        },
        LinkageWithReferences {
            parent: Item::ReviewItem(&test_data.review_items[2]),
            smaller: Item::ReviewItem(&test_data.review_items[3]),
        },
        LinkageWithReferences {
            parent: Item::ReviewItem(&test_data.review_items[3]),
            smaller: Item::NextStepItem(&test_data.next_steps[1]),
        },
        //REVIEW STEPS to REASONS
        LinkageWithReferences {
            parent: Item::ReasonItem(&test_data.reason_items[0]),
            smaller: Item::ReviewItem(&test_data.review_items[0]),
        },
        LinkageWithReferences {
            parent: Item::ReasonItem(&test_data.reason_items[1]),
            smaller: Item::ReviewItem(&test_data.review_items[2]),
        },
    ];

    linkage
}

pub async fn upload_linkage_to_surrealdb<'a, T: surrealdb::Connection>(linkage: Vec<LinkageWithReferences<'a>>, db: &Surreal<T>) -> Vec<LinkageWithRecordIds>
{
    let mut result = Vec::with_capacity(linkage.capacity());
    for with_references in linkage.into_iter() {
        let with_record_ids: LinkageWithRecordIds = with_references.into();
        let r = with_record_ids.create(&db).await.unwrap();
        result.extend(r.into_iter());
    }
    result
}