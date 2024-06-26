pub(crate) mod bullet_list_single_item;

use std::{fmt::Display, iter::once};

use chrono::{DateTime, Local, TimeDelta, Utc};
use inquire::{InquireError, Select};
use itertools::chain;
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::BaseData,
    calculated_data::CalculatedData,
    display::{
        display_item_lap_count::DisplayItemLapCount, display_scheduled_item::DisplayScheduledItem,
    },
    menu::inquire::top_menu::present_top_menu,
    node::item_lap_count::ItemLapCount,
    surrealdb_layer::{surreal_tables::SurrealTables, DataLayerCommands},
    systems::bullet_list::{BulletList, BulletListReason},
};

use self::bullet_list_single_item::{
    present_bullet_list_item_selected, present_is_person_or_group_around_menu,
    set_staging::{present_set_staging_menu, StagingMenuSelection},
};

use super::top_menu::capture;

pub(crate) enum InquireBulletListItem<'e> {
    CaptureNewItem,
    SetStaging(&'e ItemLapCount<'e>),
    Item(&'e ItemLapCount<'e>, &'e DateTime<Utc>),
}

impl Display for InquireBulletListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CaptureNewItem => write!(f, "🗬   Capture New Item          🗭")?,
            Self::Item(item_lap_count, _current_date_time) => {
                let display_item_lap_count = DisplayItemLapCount::new(item_lap_count);
                write!(f, "{}", display_item_lap_count)?;
            }
            Self::SetStaging(item_lap_count) => {
                let display_item_status = DisplayItemLapCount::new(item_lap_count);
                write!(f, "[SET STAGING] {}", display_item_status)?;
            }
        }
        Ok(())
    }
}

impl<'a> InquireBulletListItem<'a> {
    pub(crate) fn create_list(
        item_status: &'a [BulletListReason<'a>],
        bullet_list_created: &'a DateTime<Utc>,
    ) -> Vec<InquireBulletListItem<'a>> {
        chain!(
            once(InquireBulletListItem::CaptureNewItem),
            item_status.iter().map(|x| match x {
                BulletListReason::SetStaging(item_status) =>
                    InquireBulletListItem::SetStaging(item_status.get_item_lap_count()),
                BulletListReason::WorkOn(item_status) => {
                    InquireBulletListItem::Item(
                        item_status.get_item_lap_count(),
                        bullet_list_created,
                    )
                }
            })
        )
        .collect()
    }
}

pub(crate) async fn present_normal_bullet_list_menu_version_2(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let before_db_query = Local::now();
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let elapsed = Local::now() - before_db_query;
    if elapsed > chrono::Duration::try_seconds(1).expect("valid") {
        println!("Slow to get data from database. Time taken: {}", elapsed);
    }
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
    let bullet_list = BulletList::new_bullet_list_version_2(calculated_data, &now);
    let elapsed = Utc::now() - now;
    if elapsed > chrono::Duration::try_seconds(1).expect("valid") {
        println!("Slow to create bullet list. Time taken: {}", elapsed);
    }
    present_bullet_list_menu(&bullet_list, now, send_to_data_storage_layer).await
}

pub(crate) async fn present_normal_bullet_list_menu_version_1(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let before_db_query = Local::now();
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let elapsed = Local::now() - before_db_query;
    if elapsed > chrono::Duration::try_seconds(1).expect("valid") {
        println!("Slow to get data from database. Time taken: {}", elapsed);
    }
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
    let bullet_list = BulletList::new_bullet_list_version_1(calculated_data, &now);
    let elapsed = Utc::now() - now;
    if elapsed > chrono::Duration::try_seconds(1).expect("valid") {
        println!("Slow to create bullet list. Time taken: {}", elapsed);
    }
    present_upcoming(&bullet_list, now);
    present_bullet_list_menu(&bullet_list, now, send_to_data_storage_layer).await
}

pub(crate) fn present_upcoming(bullet_list: &BulletList, now: DateTime<Utc>) {
    let upcoming = bullet_list.get_upcoming();
    if !upcoming.is_empty() {
        println!("Upcoming:");
        for scheduled_item in upcoming
            .get_ordered_scheduled_items()
            .as_ref()
            .expect("upcoming is not empty")
        {
            let display_scheduled_item = DisplayScheduledItem::new(scheduled_item, &now);
            println!("{}", display_scheduled_item);
        }
    }
}

pub(crate) async fn present_bullet_list_menu(
    bullet_list: &BulletList,
    bullet_list_created: DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let ordered_bullet_list = bullet_list.get_ordered_bullet_list();

    let inquire_bullet_list =
        InquireBulletListItem::create_list(ordered_bullet_list, &bullet_list_created);

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("Select from the below list|", inquire_bullet_list)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::CaptureNewItem) => capture(send_to_data_storage_layer).await,
            Ok(InquireBulletListItem::Item(item_status, bullet_list_created)) => {
                if item_status.is_person_or_group() {
                    present_is_person_or_group_around_menu(
                        item_status.get_item_node(),
                        send_to_data_storage_layer,
                    )
                    .await
                } else {
                    Box::pin(present_bullet_list_item_selected(
                        item_status,
                        chrono::Utc::now(),
                        bullet_list,
                        bullet_list_created,
                        send_to_data_storage_layer,
                    ))
                    .await
                }
            }
            Ok(InquireBulletListItem::SetStaging(item_status)) => {
                present_set_staging_menu(
                    item_status.get_item(),
                    send_to_data_storage_layer,
                    Some(StagingMenuSelection::OnDeck),
                )
                .await
            }
            Err(InquireError::OperationCanceled) => {
                //Pressing Esc is meant to refresh the list unless you press it twice in a row then it will go to the top menu
                if Utc::now() - bullet_list_created > TimeDelta::seconds(5) {
                    println!("Refreshing the list");
                    Box::pin(present_normal_bullet_list_menu_version_1(
                        send_to_data_storage_layer,
                    ))
                    .await
                } else {
                    Box::pin(present_top_menu(send_to_data_storage_layer)).await
                }
            }
            Err(InquireError::OperationInterrupted) => Err(()),
            Err(err) => todo!("Unexpected InquireError of {}", err),
        }
    } else {
        println!("To Do List is Empty, falling back to main menu");
        Box::pin(present_top_menu(send_to_data_storage_layer)).await
    }
}
