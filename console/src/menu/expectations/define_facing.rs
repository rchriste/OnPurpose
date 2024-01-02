use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::Utc;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::BaseData,
    display::display_item_node::DisplayItemNode,
    menu::{
        bullet_list_menu::bullet_list_single_item::parent_to_a_goal_or_motivation::parent_to_a_goal_or_motivation,
        select_person_or_group::select_person_or_group,
    },
    node::item_node::ItemNode,
    surrealdb_layer::{
        surreal_item::{Facing, HowWellDefined},
        DataLayerCommands,
    },
};

use super::view_expectations;

#[async_recursion]
pub(crate) async fn define_facing(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    loop {
        let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
            .await
            .unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let active_items = base_data.get_active_items();
        let covering = base_data.get_coverings();
        let active_covering_until_date_time = base_data.get_active_snoozed();

        let item_nodes = active_items
            .iter()
            .filter(|x| !x.is_person_or_group())
            .map(|x| ItemNode::new(x, covering, active_covering_until_date_time, active_items))
            .collect::<Vec<_>>();

        let item_nodes = item_nodes
            .iter()
            .filter(|x| !x.has_larger() && x.is_facing_undefined())
            .collect::<Vec<_>>();

        let display_item_nodes = item_nodes
            .iter()
            .map(|x| DisplayItemNode::new(x))
            .collect::<Vec<_>>();

        let selection = Select::new(
            &format!("Select an item ({} Remaining)|", display_item_nodes.len()),
            display_item_nodes,
        )
        .prompt();

        match selection {
            Ok(selection) => {
                let item_node = selection.get_item_node();
                single_item_define_facing(item_node, send_to_data_storage_layer).await?
            }
            Err(InquireError::OperationCanceled) => {
                view_expectations(send_to_data_storage_layer).await?
            }
            Err(InquireError::OperationInterrupted) => return Err(()),
            Err(err) => todo!("{:?}", err),
        }
    }
}

pub(crate) enum FacingOptions {
    PickParent,
    ForMyself,
    ForAnother,
    ForMyselfAndAnother,
}

impl Display for FacingOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FacingOptions::PickParent => write!(f, "Pick a Parent"),
            FacingOptions::ForMyself => write!(f, "For Myself"),
            FacingOptions::ForMyselfAndAnother => write!(f, "For Myself and Another"),
            FacingOptions::ForAnother => write!(f, "For Another"),
        }
    }
}

impl FacingOptions {
    pub(crate) fn get_list() -> Vec<Self> {
        vec![
            FacingOptions::PickParent,
            FacingOptions::ForMyself,
            FacingOptions::ForAnother,
            FacingOptions::ForMyselfAndAnother,
        ]
    }
}

#[derive(Clone, Copy)]
enum HowWellDefinedSelection {
    NotSet,
    WellDefined,
    RoughlyDefined,
    LooselyDefined,
}

impl Display for HowWellDefinedSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HowWellDefinedSelection::NotSet => write!(f, "Not Set"),
            HowWellDefinedSelection::WellDefined => write!(f, "Well Defined"),
            HowWellDefinedSelection::RoughlyDefined => write!(f, "Roughly Defined"),
            HowWellDefinedSelection::LooselyDefined => write!(f, "Loosely Defined"),
        }
    }
}

impl HowWellDefinedSelection {
    pub(crate) fn get_list() -> Vec<Self> {
        vec![
            HowWellDefinedSelection::NotSet,
            HowWellDefinedSelection::WellDefined,
            HowWellDefinedSelection::RoughlyDefined,
            HowWellDefinedSelection::LooselyDefined,
        ]
    }
}

impl From<HowWellDefinedSelection> for HowWellDefined {
    fn from(item: HowWellDefinedSelection) -> Self {
        match item {
            HowWellDefinedSelection::NotSet => HowWellDefined::NotSet,
            HowWellDefinedSelection::WellDefined => HowWellDefined::WellDefined,
            HowWellDefinedSelection::RoughlyDefined => HowWellDefined::RoughlyDefined,
            HowWellDefinedSelection::LooselyDefined => HowWellDefined::LooselyDefined,
        }
    }
}

#[async_recursion]
async fn single_item_define_facing(
    item_node: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = FacingOptions::get_list();
    let selection = Select::new("Select a facing |", list).prompt();

    match selection {
        Ok(FacingOptions::PickParent) => {
            parent_to_a_goal_or_motivation(item_node.get_item(), send_to_data_storage_layer).await
        }
        Ok(FacingOptions::ForMyself) => {
            let list = HowWellDefinedSelection::get_list();
            let selection = Select::new("Select How Well Defined |", list).prompt();
            match selection {
                Ok(selection) => {
                    let facing = Facing::Myself(selection.into());
                    send_to_data_storage_layer
                        .send(DataLayerCommands::UpdateFacing(
                            item_node.get_surreal_record_id().clone(),
                            vec![facing],
                        ))
                        .await
                        .unwrap();
                    Ok(())
                }
                Err(InquireError::OperationCanceled) => {
                    single_item_define_facing(item_node, send_to_data_storage_layer).await
                }
                Err(InquireError::OperationInterrupted) => Err(()),
                Err(err) => todo!("{:?}", err),
            }
        }
        Ok(FacingOptions::ForAnother) => {
            let person_or_group = select_person_or_group(send_to_data_storage_layer)
                .await
                .unwrap();
            let list = HowWellDefinedSelection::get_list();
            let selection = Select::new("Select How Well Defined |", list).prompt();
            match selection {
                Ok(selection) => {
                    let facing = Facing::Others {
                        how_well_defined: selection.into(),
                        who: person_or_group,
                    };
                    send_to_data_storage_layer
                        .send(DataLayerCommands::UpdateFacing(
                            item_node.get_surreal_record_id().clone(),
                            vec![facing],
                        ))
                        .await
                        .unwrap();
                    Ok(())
                }
                Err(InquireError::OperationCanceled) => {
                    single_item_define_facing(item_node, send_to_data_storage_layer).await
                }
                Err(InquireError::OperationInterrupted) => Err(()),
                Err(err) => todo!("{:?}", err),
            }
        }
        Ok(FacingOptions::ForMyselfAndAnother) => {
            let person_or_group = select_person_or_group(send_to_data_storage_layer)
                .await
                .unwrap();
            let list = HowWellDefinedSelection::get_list();
            let selection = Select::new("Select How Well Defined |", list).prompt();
            match selection {
                Ok(selection) => {
                    let myself_facing = Facing::Myself(selection.into());
                    let others_facing = Facing::Others {
                        how_well_defined: selection.into(),
                        who: person_or_group,
                    };
                    send_to_data_storage_layer
                        .send(DataLayerCommands::UpdateFacing(
                            item_node.get_surreal_record_id().clone(),
                            vec![myself_facing, others_facing],
                        ))
                        .await
                        .unwrap();
                    Ok(())
                }
                Err(InquireError::OperationCanceled) => {
                    single_item_define_facing(item_node, send_to_data_storage_layer).await
                }
                Err(InquireError::OperationInterrupted) => Err(()),
                Err(err) => todo!("{:?}", err),
            }
        }
        Err(InquireError::OperationCanceled) => define_facing(send_to_data_storage_layer).await,
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("{:?}", err),
    }
}
