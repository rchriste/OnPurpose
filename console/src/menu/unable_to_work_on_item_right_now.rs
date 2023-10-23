use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::Local;
use inquire::InquireError;
use parse_datetime::{parse_datetime_at_date, ParseDateTimeError};
use tokio::sync::mpsc::Sender;

use crate::{base_data::item::Item, surrealdb_layer::DataLayerCommands};

enum UnableReason {
    SomeoneIsNotAround,
    PlaceToContactIsNotOpen,
}

impl Display for UnableReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnableReason::SomeoneIsNotAround => {
                write!(f, "Someone is not around")
            }
            UnableReason::PlaceToContactIsNotOpen => {
                write!(f, "Place to contact is not open")
            }
        }
    }
}

impl UnableReason {
    fn make_list() -> Vec<UnableReason> {
        vec![Self::SomeoneIsNotAround, Self::PlaceToContactIsNotOpen]
    }
}

pub(crate) async fn unable_to_work_on_item_right_now(
    unable_to_do: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = UnableReason::make_list();
    let selection = inquire::Select::new("", list).prompt();

    match selection {
        Ok(UnableReason::SomeoneIsNotAround) => {
            todo!()
        }
        Ok(UnableReason::PlaceToContactIsNotOpen) => {
            place_to_contact_is_not_open(unable_to_do, send_to_data_storage_layer).await
        }
        Err(InquireError::OperationCanceled) => {
            todo!("I put in this todo because back at the time I would need to adjust some calling parameters to make this work")
        }
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

#[async_recursion]
pub(crate) async fn place_to_contact_is_not_open(
    unable_to_do: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
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
                    unable_to_do.get_surreal_item().clone(),
                    when_they_will_be_open,
                ))
                .await
                .unwrap();
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
                    unable_to_do.get_surreal_item().clone(),
                    when_they_will_be_open.into(),
                ))
                .await
                .unwrap();
        }
        Ok(WhatLibraryToUse::DurationStr) => {
            todo!()
        }
        Err(InquireError::OperationCanceled) => {
            unable_to_work_on_item_right_now(unable_to_do, send_to_data_storage_layer).await
        }
        Err(err) => todo!("{:?}", err),
    }
}
