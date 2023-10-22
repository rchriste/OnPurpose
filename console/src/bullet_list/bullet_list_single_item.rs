mod cover_bullet_item;

use std::fmt::Display;

use async_recursion::async_recursion;
use inquire::{Editor, InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        item::{Item, ItemVecExtensions},
        to_do::ToDo,
        ItemType,
    },
    bullet_list::bullet_list_single_item::cover_bullet_item::cover_bullet_item,
    display_item::DisplayItem,
    new_item,
    surrealdb_layer::{surreal_item::Responsibility, DataLayerCommands},
    update_item_summary, UnexpectedNextMenuAction,
};

enum BulletListSingleItemSelection<'e> {
    ProcessAndFinish,
    Cover,
    UpdateSummary,
    //Take a DisplayItem rather than a reference because it is felt that this type is only created
    //for this scenario rather than kept around.
    SwitchToParentItem(DisplayItem<'e>),
    ParentToItem,
    DebugPrintItem,
}

impl Display for BulletListSingleItemSelection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessAndFinish => write!(f, "Process & Finish 📕"),
            Self::Cover => write!(f, "Cover ⼍"),
            Self::UpdateSummary => write!(f, "Update Summary"),
            Self::SwitchToParentItem(parent_item) => write!(f, "Switch to: {}", parent_item),
            Self::ParentToItem => {
                write!(f, "⭱ Parent to a new or existing Item")
            }
            Self::DebugPrintItem => write!(f, "Debug Print Item"),
        }
    }
}

impl<'e> BulletListSingleItemSelection<'e> {
    fn create_list(parent_items: &[&'e Item<'e>]) -> Vec<Self> {
        let mut list = vec![Self::ProcessAndFinish, Self::Cover, Self::UpdateSummary];

        if parent_items.is_empty() {
            list.push(Self::ParentToItem);
        } else {
            list.extend(
                parent_items
                    .iter()
                    .map(|x: &&'e Item<'e>| Self::SwitchToParentItem(DisplayItem::new(x))),
            );
        }

        list.push(Self::DebugPrintItem);

        list
    }
}

#[async_recursion]
pub(crate) async fn present_bullet_list_item_selected(
    menu_for: &ToDo<'_>,
    parents: &[&Item<'_>],
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = BulletListSingleItemSelection::create_list(parents);

    let selection = Select::new("", list).prompt();

    match selection {
        Ok(BulletListSingleItemSelection::ProcessAndFinish) => {
            process_and_finish_bullet_item(menu_for.get_item(), send_to_data_storage_layer).await;
        }
        Ok(BulletListSingleItemSelection::Cover) => {
            let r = cover_bullet_item(menu_for, send_to_data_storage_layer).await;
            match r {
                Ok(()) => (),
                Err(UnexpectedNextMenuAction::Back) => {
                    present_bullet_list_item_selected(menu_for, parents, send_to_data_storage_layer)
                        .await
                }
                Err(UnexpectedNextMenuAction::Close) => todo!(),
            }
        }
        Ok(BulletListSingleItemSelection::UpdateSummary) => {
            update_item_summary(
                menu_for.get_surreal_item().clone(),
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::SwitchToParentItem(item)) => {
            present_bullet_list_item_parent_selected(item.into(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::ParentToItem) => {
            parent_to_item(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::DebugPrintItem) => {
            println!("{:?}", menu_for);
        }
        Err(InquireError::OperationCanceled) => (), //Nothing to do we just want to return to the bullet list
        Err(err) => todo!("Unexpected {}", err),
    }
}

async fn process_and_finish_bullet_item(
    item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    //I should probably be processing and finishing all of the children next steps but this requires some thought
    //because sometimes or if there are multiple children next steps that that shouldn't happen rather the user
    //should be prompted to pick which children to also process and finish.
    let user_processed_text = Editor::new("Process text").prompt().unwrap();

    let surreal_item = item.get_surreal_item();
    if !user_processed_text.is_empty() {
        send_to_data_storage_layer
            .send(DataLayerCommands::AddProcessedText(
                user_processed_text,
                surreal_item.clone(),
            ))
            .await
            .unwrap();
    }

    send_to_data_storage_layer
        .send(DataLayerCommands::FinishItem(surreal_item.clone()))
        .await
        .unwrap();
}

async fn present_bullet_list_item_parent_selected(
    selected_item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    match selected_item.item_type {
        crate::base_data::ItemType::ToDo => {
            let to_do = ToDo::new(selected_item);
            let raw_data = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
                .await
                .unwrap();
            let items = raw_data.make_items();
            let active_items = items.filter_active_items();
            let coverings = raw_data.make_coverings(&items);
            let parents = selected_item.find_parents(&coverings, &active_items);
            present_bullet_list_item_selected(&to_do, &parents, send_to_data_storage_layer).await
        }
        crate::base_data::ItemType::Hope => todo!(),
        crate::base_data::ItemType::Motivation => todo!(),
    }
}

async fn parent_to_item(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let raw_data = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();
    let items: Vec<Item> = raw_data
        .make_items()
        .into_iter()
        .filter(|x| !x.is_finished())
        .collect();

    let list = DisplayItem::make_list(&items);

    let selection = Select::new("Type to Search or Press Esc to enter a new one", list).prompt();
    match selection {
        Ok(display_item) => {
            let item: &Item = display_item.into();
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent: item.get_surreal_item().clone(),
                })
                .await
                .unwrap();
        }
        Err(InquireError::OperationCanceled | InquireError::InvalidConfiguration(_)) => {
            parent_to_new_item(parent_this, send_to_data_storage_layer).await
        }
        Err(err) => todo!("Unexpected {}", err),
    }
}

pub(crate) async fn cover_with_item(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) { //TODO: cover_to_item and parent_to_item are the same except for the command sent to the data storage layer, refactor to reduce duplicated code
    let raw_data = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();
    let items: Vec<Item> = raw_data
        .make_items()
        .into_iter()
        .filter(|x| !x.is_finished())
        .collect();

    let list = DisplayItem::make_list(&items);

    let selection = Select::new("Type to Search or Press Esc to enter a new one", list).prompt();
    match selection {
        Ok(display_item) => {
            let item: &Item = display_item.into();
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent: item.get_surreal_item().clone(),
                })
                .await
                .unwrap();
        }
        Err(InquireError::OperationCanceled | InquireError::InvalidConfiguration(_)) => {
            cover_with_new_item(parent_this, send_to_data_storage_layer).await
        }
        Err(err) => todo!("Unexpected {}", err),
    }
}


enum NewItemSelection {
    ProactiveToDo,
    ResponsiveToDo,
    ProactiveHope,
    ProactiveMilestone,
    ResponsiveHope,
    ProactiveMotivation,
    ResponsiveMotivation,
}

impl Display for NewItemSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProactiveToDo => write!(f, "New Proactive To Do"),
            Self::ResponsiveToDo => write!(f, "New Responsive To Do"),
            Self::ProactiveHope => write!(f, "New Proactive Hope"),
            Self::ResponsiveHope => write!(f, "New Responsive Hope"),
            Self::ProactiveMotivation => write!(f, "New Proactive Motivation"),
            Self::ResponsiveMotivation => write!(f, "New Responsive Motivation"),
            Self::ProactiveMilestone => write!(f, "New Proactive Milestone"),
        }
    }
}

impl NewItemSelection {
    fn create_list() -> Vec<Self> {
        vec![
            Self::ProactiveToDo,
            Self::ResponsiveToDo,
            Self::ProactiveHope,
            Self::ProactiveMilestone,
            Self::ResponsiveHope,
            Self::ProactiveMotivation,
            Self::ResponsiveMotivation,
        ]
    }
}

pub(crate) async fn parent_to_new_item(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = NewItemSelection::create_list();

    let selection = Select::new("", list).prompt();
    match selection {
        Ok(NewItemSelection::ProactiveToDo) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::ToDo,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ResponsiveToDo) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::ToDo,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveHope) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Hope,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveMilestone) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let _new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Hope,
            };
            todo!("I need to also set this hope to be a milestone");
            // send_to_data_storage_layer.send(DataLayerCommands::ParentItemWithANewItem{
            //     child: parent_this.get_surreal_item().clone(),
            //     parent_new_item: new_item,
            // }).await.unwrap();
        }
        Ok(NewItemSelection::ResponsiveHope) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::Hope,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveMotivation) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Motivation,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ResponsiveMotivation) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::Motivation,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }

        Err(InquireError::OperationCanceled) => todo!(),
        Err(err) => todo!("Unexpected {}", err),
    }
}

pub(crate) async fn cover_with_new_item(
    cover_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = NewItemSelection::create_list();

    let selection = Select::new("", list).prompt();
    match selection {
        Ok(NewItemSelection::ProactiveToDo) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::ToDo,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ResponsiveToDo) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::ToDo,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveHope) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Hope,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveMilestone) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let _new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Hope,
            };
            todo!("I need to also set this hope to be a milestone");
            // send_to_data_storage_layer.send(DataLayerCommands::CoverWithANewItem{
            //     cover_this: cover_this.get_surreal_item().clone(),
            //     cover_with: new_item,
            // }).await.unwrap();
        }
        Ok(NewItemSelection::ResponsiveHope) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::Hope,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveMotivation) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Motivation,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ResponsiveMotivation) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::Motivation,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }

        Err(InquireError::OperationCanceled) => todo!(),
        Err(err) => todo!("Unexpected {}", err),
    }
}
