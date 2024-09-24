use std::fmt::{self, Display, Formatter};

use chrono::Utc;
use inquire::Select;
use tokio::sync::mpsc::Sender;

use crate::{
    data_storage::surrealdb_layer::{
        data_layer_commands::DataLayerCommands,
        surreal_in_the_moment_priority::SurrealAction,
        surreal_item::{SurrealFrequency, SurrealReviewGuidance, SurrealUrgency},
    },
    new_time_spent::NewTimeSpent,
    node::item_status::ItemStatus,
};

pub(crate) enum Frequency {
    NoneReviewWithParent,
    Custom,
    Hourly,
    Daily,
    EveryFewDays,
    Weekly,
    BiMonthly,
    Monthly,
    Quarterly,
    SemiAnnually,
    Yearly,
}

impl Display for Frequency {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Frequency::NoneReviewWithParent => write!(f, "None, Review With Parent"),
            Frequency::Custom => write!(f, "Custom Frequency"),
            Frequency::Hourly => write!(f, "Hourly"),
            Frequency::Daily => write!(f, "Daily"),
            Frequency::EveryFewDays => write!(f, "Every Few Days"),
            Frequency::Weekly => write!(f, "Weekly"),
            Frequency::BiMonthly => write!(f, "Bi-Monthly"),
            Frequency::Monthly => write!(f, "Monthly"),
            Frequency::Quarterly => write!(f, "Quarterly"),
            Frequency::SemiAnnually => write!(f, "Semi-Annually"),
            Frequency::Yearly => write!(f, "Yearly"),
        }
    }
}

impl Frequency {
    pub(crate) fn make_list() -> Vec<Frequency> {
        vec![
            Frequency::NoneReviewWithParent,
            Frequency::Hourly,
            Frequency::Daily,
            Frequency::EveryFewDays,
            Frequency::Weekly,
            Frequency::BiMonthly,
            Frequency::Monthly,
            Frequency::Quarterly,
            Frequency::SemiAnnually,
            Frequency::Yearly,
            Frequency::Custom,
        ]
    }
}

enum ReviewGuidance {
    ReviewChildrenTogether,
    ReviewChildrenSeparately,
}

impl Display for ReviewGuidance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ReviewGuidance::ReviewChildrenTogether => write!(f, "Review Children Together"),
            ReviewGuidance::ReviewChildrenSeparately => write!(f, "Review Children Separately"),
        }
    }
}

impl From<ReviewGuidance> for SurrealReviewGuidance {
    fn from(review_guidance: ReviewGuidance) -> Self {
        match review_guidance {
            ReviewGuidance::ReviewChildrenTogether => {
                SurrealReviewGuidance::AlwaysReviewChildrenWithThisItem
            }
            ReviewGuidance::ReviewChildrenSeparately => {
                SurrealReviewGuidance::ReviewChildrenSeparately
            }
        }
    }
}

impl ReviewGuidance {
    pub(crate) fn make_list() -> Vec<ReviewGuidance> {
        vec![
            ReviewGuidance::ReviewChildrenTogether,
            ReviewGuidance::ReviewChildrenSeparately,
        ]
    }
}

pub(crate) async fn present_pick_item_review_frequency_menu(
    item_status: &ItemStatus<'_>,
    current_urgency: SurrealUrgency,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let started_present_pick_item_review_frequency = Utc::now();
    let review_frequency = Select::new(
        "How often should you review this item?",
        Frequency::make_list(),
    )
    .with_page_size(10)
    .prompt()
    .unwrap();

    let surreal_review_frequency = match review_frequency {
        Frequency::NoneReviewWithParent => SurrealFrequency::NoneReviewWithParent,
        Frequency::Custom => {
            todo!("Prompt for a minimum duration and a maximum duration")
        }
        Frequency::Hourly => SurrealFrequency::Hourly,
        Frequency::Daily => SurrealFrequency::Daily,
        Frequency::EveryFewDays => SurrealFrequency::EveryFewDays,
        Frequency::Weekly => SurrealFrequency::Weekly,
        Frequency::BiMonthly => SurrealFrequency::BiMonthly,
        Frequency::Monthly => SurrealFrequency::Monthly,
        Frequency::Quarterly => SurrealFrequency::Quarterly,
        Frequency::SemiAnnually => SurrealFrequency::SemiAnnually,
        Frequency::Yearly => SurrealFrequency::Yearly,
    };

    let review_guidance = Select::new(
        "Should children items be reviewed with this item?",
        ReviewGuidance::make_list(),
    )
    .prompt()
    .unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateItemReviewFrequency(
            item_status.get_surreal_record_id().clone(),
            surreal_review_frequency,
            review_guidance.into(),
        ))
        .await
        .unwrap();

    let new_time_spent = NewTimeSpent {
        working_on: vec![SurrealAction::ReviewItem(
            item_status.get_surreal_record_id().clone(),
        )], //TODO: Should I also add all the parent items that this is making progress towards the goal
        when_started: started_present_pick_item_review_frequency,
        when_stopped: Utc::now(),
        dedication: None,
        urgency: Some(current_urgency),
    };
    send_to_data_storage_layer
        .send(DataLayerCommands::RecordTimeSpent(new_time_spent))
        .await
        .unwrap();

    Ok(())
}
