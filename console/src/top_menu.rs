use std::fmt::Display;

use async_recursion::async_recursion;
use inquire::{InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::item::{Item, ItemVecExtensions},
    bullet_list::present_unfocused_bullet_list_menu,
    change_routine::change_routine,
    display_item::DisplayItem,
    mentally_resident::view_hopes,
    surrealdb_layer::DataLayerCommands,
};

enum TopMenuSelection {
    CaptureToDo,
    ChangeRoutine,
    ViewBulletList,
    CaptureHope,
    ViewHopes,
    CaptureMotivation,
    ViewMotivations,
    DebugViewAllItems,
}

impl Display for TopMenuSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopMenuSelection::CaptureToDo => write!(f, "🗬 🗒️ Capture To Do             🗭"),
            TopMenuSelection::ChangeRoutine => write!(f, "↝ ↝ Change Routine            ↜"),
            TopMenuSelection::ViewBulletList => write!(f, "👁 🗒️ View Bullet List (To Dos) 👁"),
            TopMenuSelection::CaptureHope => write!(f, "🗬 🙏 Capture Hope              🗭"),
            TopMenuSelection::ViewHopes => {
                write!(f, "👁 🙏 View Hopes                👁")
            }
            TopMenuSelection::CaptureMotivation => {
                write!(f, "🗬 🎯 Capture Motivation        🗭")
            }
            TopMenuSelection::ViewMotivations => {
                write!(f, "👁 🎯 View Motivations          👁")
            }
            TopMenuSelection::DebugViewAllItems => {
                write!(f, "👁 🗒️ Debug View All Items      👁")
            }
        }
    }
}

impl TopMenuSelection {
    fn make_list() -> Vec<TopMenuSelection> {
        vec![
            Self::CaptureToDo,
            Self::ChangeRoutine,
            Self::ViewBulletList,
            Self::CaptureHope,
            Self::ViewHopes,
            Self::CaptureMotivation,
            Self::ViewMotivations,
            Self::DebugViewAllItems,
        ]
    }
}

#[async_recursion]
pub(crate) async fn present_top_menu(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let top_menu = TopMenuSelection::make_list();

    let selection = Select::new("", top_menu).prompt().unwrap();
    match selection {
        TopMenuSelection::CaptureToDo => capture_to_do(send_to_data_storage_layer).await,
        TopMenuSelection::ChangeRoutine => change_routine(send_to_data_storage_layer).await,
        TopMenuSelection::CaptureHope => capture_hope(send_to_data_storage_layer).await,
        TopMenuSelection::ViewHopes => view_hopes(send_to_data_storage_layer).await,
        TopMenuSelection::ViewBulletList => {
            present_unfocused_bullet_list_menu(send_to_data_storage_layer).await
        }
        TopMenuSelection::CaptureMotivation => capture_motivation(send_to_data_storage_layer).await,
        TopMenuSelection::ViewMotivations => view_motivations().await,
        TopMenuSelection::DebugViewAllItems => {
            debug_view_all_items(send_to_data_storage_layer).await
        }
    }
}

async fn capture_to_do(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let new_next_step_text = Text::new("Enter To Do ⍠").prompt().unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::NewToDo(new_next_step_text))
        .await
        .unwrap();
}

async fn capture_hope(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let new_hope_text = Text::new("Enter Hope ⍠").prompt().unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::NewHope(new_hope_text))
        .await
        .unwrap();
}

async fn capture_motivation(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let summary_text = Text::new("Enter Motivation ⍠").prompt().unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::NewMotivation(summary_text))
        .await
        .unwrap();
}

async fn view_motivations() {
    todo!()
}

enum DebugViewItem<'e> {
    Item(DisplayItem<'e>),
}

impl Display for DebugViewItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugViewItem::Item(item) => write!(f, "{}", item),
        }
    }
}

impl<'e> DebugViewItem<'e> {
    fn make_list(items: &'e [&'e Item<'e>]) -> Vec<DebugViewItem<'e>> {
        items
            .iter()
            .map(|x| DebugViewItem::Item(DisplayItem::new(x)))
            .collect()
    }
}

async fn debug_view_all_items(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();

    let items = surreal_tables.make_items();
    let active_items = items.filter_active_items();

    let list = DebugViewItem::make_list(&active_items);

    let selection = Select::new("Select an item to show the debug view of...", list).prompt();
    match selection {
        Ok(DebugViewItem::Item(item)) => {
            let item: &Item = item.item;
            println!("{:#?}", item);
        }
        Err(InquireError::OperationCanceled) => present_top_menu(send_to_data_storage_layer).await,
        Err(err) => todo!("Unexpected InquireError of {}", err),
    }
}
