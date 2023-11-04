use inquire::{InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{item::Item, BaseData},
    display::display_item::DisplayItem,
    menu::bullet_list_menu::bullet_list_single_item::ItemTypeSelection,
    new_item,
    surrealdb_layer::{
        surreal_item::{ItemType, Responsibility},
        surreal_tables::SurrealTables,
        DataLayerCommands,
    },
};

pub(crate) async fn parent_to_a_goal(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables);
    let active_items = base_data.get_active_items();
    let goals = base_data.get_just_hopes();
    let list = goals
        .iter()
        .map(|x| DisplayItem::new(x.get_item()))
        .collect::<Vec<_>>();

    let selection = Select::new("", list).prompt();
    match selection {
        Ok(parent) => {
            let parent: &Item<'_> = parent.into();
            if parent.has_children(active_items) {
                todo!("I need to pick a priority for this item among the children of the parent");
            } else {
                send_to_data_storage_layer
                    .send(DataLayerCommands::ParentItemWithExistingItem {
                        child: parent_this.get_surreal_item().clone(),
                        parent: parent.get_surreal_item().clone(),
                    })
                    .await
                    .unwrap();
            }
        }
        Err(InquireError::OperationCanceled) => {
            parent_to_a_goal_new_goal(parent_this, send_to_data_storage_layer).await;
        }
        Err(err) => {
            todo!("Error: {:?}", err);
        }
    }
}

async fn parent_to_a_goal_new_goal(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = ItemTypeSelection::create_list_just_goals();
    let selection = Select::new("", list).prompt();
    match selection {
        Ok(ItemTypeSelection::ProactiveGoalThatIsAHope) => {
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
        Ok(ItemTypeSelection::ProactiveGoalThatIsAMilestone) => {
            todo!("Implement the ability to set a goal to be a milestone right away in the data layer and then this can be written")
        }
        Ok(ItemTypeSelection::ResponsiveGoal) => {
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
        Err(InquireError::OperationCanceled) => {
            todo!("I need to go back to what first called this");
        }
        Err(err) => {
            todo!("Error: {:?}", err);
        }
        Ok(
            ItemTypeSelection::ProactiveAction
            | ItemTypeSelection::ResponsiveAction
            | ItemTypeSelection::ProactiveMotivation
            | ItemTypeSelection::ResponsiveMotivation,
        ) => {
            panic!("This items should never be offered when selecting a goal to parent to");
        }
    }
}