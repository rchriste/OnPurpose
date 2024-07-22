use std::fmt::Display;

use chrono::Utc;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::BaseData,
    display::display_item_node::DisplayItemNode,
    menu::inquire::{
        bullet_list_menu::bullet_list_single_item::give_this_item_a_parent::give_this_item_a_parent,
        select_person_or_group::select_person_or_group,
    },
    node::{item_node::ItemNode, Filter},
    surrealdb_layer::{
        data_layer_commands::DataLayerCommands,
        surreal_item::{SurrealFacing, SurrealHowWellDefined},
    },
};

use super::view_expectations;

pub(crate) async fn define_facing(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    loop {
        let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
            .await
            .unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let all_items = base_data.get_items();
        let active_items = base_data.get_active_items();
        let time_spent_log = base_data.get_time_spent_log();

        let item_nodes = active_items
            .iter()
            .filter(|x| !x.is_person_or_group())
            .map(|x| ItemNode::new(x, all_items, time_spent_log))
            .collect::<Vec<_>>();

        let item_nodes = item_nodes
            .iter()
            .filter(|x| !x.has_parents(Filter::Active) && x.is_facing_undefined())
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
                Box::pin(view_expectations(send_to_data_storage_layer)).await?
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

impl From<HowWellDefinedSelection> for SurrealHowWellDefined {
    fn from(item: HowWellDefinedSelection) -> Self {
        match item {
            HowWellDefinedSelection::NotSet => SurrealHowWellDefined::NotSet,
            HowWellDefinedSelection::WellDefined => SurrealHowWellDefined::WellDefined,
            HowWellDefinedSelection::RoughlyDefined => SurrealHowWellDefined::RoughlyDefined,
            HowWellDefinedSelection::LooselyDefined => SurrealHowWellDefined::LooselyDefined,
        }
    }
}

async fn single_item_define_facing(
    item_node: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = FacingOptions::get_list();
    let selection = Select::new("Select a facing |", list).prompt();

    match selection {
        Ok(FacingOptions::PickParent) => {
            give_this_item_a_parent(item_node.get_item(), send_to_data_storage_layer).await
        }
        Ok(FacingOptions::ForMyself) => {
            let list = HowWellDefinedSelection::get_list();
            let selection = Select::new("Select How Well Defined |", list).prompt();
            match selection {
                Ok(selection) => {
                    let facing = SurrealFacing::Myself(selection.into());
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
                    Box::pin(single_item_define_facing(
                        item_node,
                        send_to_data_storage_layer,
                    ))
                    .await
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
                    let facing = SurrealFacing::Others {
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
                    Box::pin(single_item_define_facing(
                        item_node,
                        send_to_data_storage_layer,
                    ))
                    .await
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
                    let myself_facing = SurrealFacing::Myself(selection.into());
                    let others_facing = SurrealFacing::Others {
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
                    Box::pin(single_item_define_facing(
                        item_node,
                        send_to_data_storage_layer,
                    ))
                    .await
                }
                Err(InquireError::OperationInterrupted) => Err(()),
                Err(err) => todo!("{:?}", err),
            }
        }
        Err(InquireError::OperationCanceled) => {
            Box::pin(define_facing(send_to_data_storage_layer)).await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("{:?}", err),
    }
}
