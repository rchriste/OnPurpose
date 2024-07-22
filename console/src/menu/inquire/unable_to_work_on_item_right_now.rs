use core::iter::once;
use std::fmt::Display;

use chrono::{DateTime, Local, Utc};
use inquire::{InquireError, Select, Text};
use itertools::chain;
use parse_datetime::{parse_datetime_at_date, ParseDateTimeError};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        item::{Item, ItemVecExtensions},
        BaseData,
    },
    display::display_item::DisplayItem,
    new_item::NewItem,
    surrealdb_layer::{data_layer_commands::DataLayerCommands, surreal_tables::SurrealTables},
};

use super::super::YesOrNo;

enum UnableReason {
    SomeoneOrGroupIsNotAvailable,
    PlaceToContactIsNotOpen,
    NeedToWaitBeforeWorkingOnThis,
    NotEnoughTime,
}

impl Display for UnableReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnableReason::SomeoneOrGroupIsNotAvailable => {
                write!(f, "Someone is not available (or group is not available)")
            }
            UnableReason::PlaceToContactIsNotOpen => {
                write!(f, "Place to contact is not open")
            }
            UnableReason::NeedToWaitBeforeWorkingOnThis => {
                write!(f, "Need to wait before working on this further")
            }
            UnableReason::NotEnoughTime => {
                write!(f, "I don't have enough time to do this right now")
            }
        }
    }
}

impl UnableReason {
    fn make_list() -> Vec<UnableReason> {
        vec![
            Self::SomeoneOrGroupIsNotAvailable,
            Self::PlaceToContactIsNotOpen,
            Self::NeedToWaitBeforeWorkingOnThis,
            Self::NotEnoughTime,
        ]
    }
}

pub(crate) async fn unable_to_work_on_item_right_now(
    unable_to_do: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = UnableReason::make_list();
    let selection = inquire::Select::new("Select from the below list|", list).prompt();

    match selection {
        Ok(UnableReason::SomeoneOrGroupIsNotAvailable) => {
            person_or_group_is_not_available(unable_to_do, send_to_data_storage_layer).await
        }
        Ok(UnableReason::PlaceToContactIsNotOpen) => {
            place_to_contact_is_not_open(unable_to_do, send_to_data_storage_layer).await
        }
        Ok(UnableReason::NeedToWaitBeforeWorkingOnThis) => {
            need_to_wait_before_working_on_this(unable_to_do, send_to_data_storage_layer).await
        }
        Ok(UnableReason::NotEnoughTime) => {
            todo!()
        }
        Err(InquireError::OperationCanceled) => {
            //Just continue because I get called from multiple places and this will allow things to return to the base bullet list
            Ok(())
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("{:?}", err),
    }
}

enum WhatLibraryToUse {
    DateParser,
    ParseDateTime,
    DurationStr,
}

impl Display for WhatLibraryToUse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhatLibraryToUse::DateParser => {
                write!(f, "dateparser")
            }
            WhatLibraryToUse::ParseDateTime => {
                write!(f, "parse_datetime")
            }
            WhatLibraryToUse::DurationStr => {
                write!(f, "duration-str")
            }
        }
    }
}

impl WhatLibraryToUse {
    fn make_list() -> Vec<WhatLibraryToUse> {
        vec![Self::DateParser, Self::ParseDateTime, Self::DurationStr]
    }
}

pub(crate) async fn place_to_contact_is_not_open(
    unable_to_do: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = WhatLibraryToUse::make_list();
    let selection = inquire::Select::new(
        "What library should be used to state when they will be open",
        list,
    )
    .prompt();
    match selection {
        Ok(WhatLibraryToUse::DateParser) => {
            let when_they_will_be_open = loop {
                let when_they_will_be_open = inquire::Text::new("When will they be open?")
                    .prompt()
                    .unwrap();
                match dateparser::parse(&when_they_will_be_open) {
                    Ok(when_they_will_be_open) => break when_they_will_be_open,
                    Err(err) => {
                        println!(
                            "Unable to parse string, error is {}, please try again.",
                            err
                        );
                    }
                }
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverItemUntilAnExactDateTime(
                    unable_to_do.get_surreal_record_id().clone(),
                    when_they_will_be_open,
                ))
                .await
                .unwrap();
            Ok(())
        }
        Ok(WhatLibraryToUse::ParseDateTime) => {
            let when_they_will_be_open = loop {
                let when_they_will_be_open = inquire::Text::new("When will they be open?")
                    .prompt()
                    .unwrap();
                let now = Local::now();
                match parse_datetime_at_date(now, when_they_will_be_open) {
                    Ok(when_they_will_be_open) => break when_they_will_be_open,
                    Err(ParseDateTimeError::InvalidInput) => {
                        println!("Unable to parse string, please try again.");
                    }
                    Err(err) => todo!("{:?}", err),
                }
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverItemUntilAnExactDateTime(
                    unable_to_do.get_surreal_record_id().clone(),
                    when_they_will_be_open.into(),
                ))
                .await
                .unwrap();
            Ok(())
        }
        Ok(WhatLibraryToUse::DurationStr) => {
            todo!()
        }
        Err(InquireError::OperationCanceled) => {
            Box::pin(unable_to_work_on_item_right_now(
                unable_to_do,
                send_to_data_storage_layer,
            ))
            .await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("{:?}", err),
    }
}

enum PersonOrGroupSelection<'e> {
    ExistingPersonOrGroup(DisplayItem<'e>),
    NewPersonOrGroup,
}

impl Display for PersonOrGroupSelection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersonOrGroupSelection::ExistingPersonOrGroup(person_or_group) => {
                write!(f, "{}", person_or_group)
            }
            PersonOrGroupSelection::NewPersonOrGroup => {
                write!(f, "New Person or Group")
            }
        }
    }
}

impl<'e> PersonOrGroupSelection<'e> {
    fn make_list(persons_or_groups: impl Iterator<Item = &'e Item<'e>>) -> Vec<Self> {
        chain!(
            persons_or_groups.map(|x| Self::ExistingPersonOrGroup(DisplayItem::new(x))),
            once(Self::NewPersonOrGroup)
        )
        .collect()
    }
}

pub(crate) async fn person_or_group_is_not_available(
    unable_to_do: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let items = base_data.get_items();
    let list = PersonOrGroupSelection::make_list(items.filter_just_persons_or_groups());

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(PersonOrGroupSelection::ExistingPersonOrGroup(person_or_group)) => {
            let person_or_group: &Item = person_or_group.into();
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverItemWithAnExistingItem {
                    item_to_be_covered: unable_to_do.get_surreal_record_id().clone(),
                    item_that_should_do_the_covering: person_or_group
                        .get_surreal_record_id()
                        .clone(),
                })
                .await
                .unwrap();
            Ok(())
        }
        Ok(PersonOrGroupSelection::NewPersonOrGroup) => {
            let summary = Text::new("Enter the name of the person or group ⍠")
                .prompt()
                .unwrap();
            let new_item = NewItem::new_person_or_group(summary, Utc::now());
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverItemWithANewItem {
                    cover_this: unable_to_do.get_surreal_record_id().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
            Ok(())
        }
        Err(InquireError::OperationCanceled) => {
            Box::pin(unable_to_work_on_item_right_now(
                unable_to_do,
                send_to_data_storage_layer,
            ))
            .await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("{:?}", err),
    }
}

pub(crate) async fn need_to_wait_before_working_on_this(
    unable_to_do: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let now = Local::now();
    let wait_until: DateTime<Utc> = loop {
        let wait_for_how_long = Text::new("Wait for how long?").prompt();
        let wait_for_how_long = match wait_for_how_long {
            Ok(wait_for_how_long) => wait_for_how_long,
            Err(InquireError::OperationCanceled) => {
                Box::pin(unable_to_work_on_item_right_now(
                    unable_to_do,
                    send_to_data_storage_layer,
                ))
                .await?;
                return Ok(());
            }
            Err(InquireError::OperationInterrupted) => return Err(()),
            Err(err) => todo!("{:?}", err),
        };
        match duration_str::parse(&wait_for_how_long) {
            Ok(wait_for_how_long) => {
                let wait_until = now + wait_for_how_long;
                let yes_or_no = YesOrNo::make_list();
                let result = Select::new(&format!("Wait until {}", &wait_until), yes_or_no)
                    .prompt()
                    .unwrap();
                match result {
                    YesOrNo::Yes => break wait_until.into(),
                    YesOrNo::No => continue,
                }
            }
            Err(err) => {
                println!(
                    "Unable to parse string, error is {}, please try again.",
                    err
                );
            }
        }
    };
    send_to_data_storage_layer
        .send(DataLayerCommands::CoverItemUntilAnExactDateTime(
            unable_to_do.get_surreal_record_id().clone(),
            wait_until,
        ))
        .await
        .unwrap();

    Ok(())
}
