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
            summary: String::from("Clean Dometic")
        },
        NextStepItem {
            summary: String::from("Fill out SafeAccess Health & Safety Invitation for RustConf 2023")
        },
        NextStepItem {
            summary: String::from("Get a Covid vaccine")
        },
    ];

    let review_items = vec![
        ReviewItem {
            summary: String::from("Go camping")
        },
        ReviewItem {
            summary: String::from("After")
        },

        ReviewItem {
            summary: String::from("Attend Rust conference")
        },
        ReviewItem {
            summary: String::from("Prepare")
        }
    ];

    let reason_items = vec![
        ReasonItem {
            summary: String::from("Family Trips")
        },
        ReasonItem {
            summary: String::from("On-Purpose")
        }
    ];

    TestData { next_steps, review_items, reason_items }
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