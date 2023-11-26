pub(crate) mod define_facing;

use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::Utc;
use inquire::{Editor, InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        covering::Covering,
        covering_until_date_time::CoveringUntilDateTime,
        item::{Item, ItemVecExtensions},
        BaseData,
    },
    menu::top_menu::present_top_menu,
    menu::{
        bullet_list_menu::bullet_list_single_item::cover_with_item,
        expectations::define_facing::define_facing, staging_query::on_deck_query,
    },
    node::item_node::ItemNode,
    surrealdb_layer::{
        surreal_item::{Permanence, Staging, SurrealItem},
        surreal_tables::SurrealTables,
        DataLayerCommands,
    },
    update_item_summary,
};

//TODO: Move things from this file into the menu folder and menu mod

struct ProjectHopeItem<'a> {
    pub(crate) hope_node: &'a ItemNode<'a>,
}

impl<'a> Display for ProjectHopeItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.hope_node.get_smaller().is_empty() {
            write!(f, "[NEEDS NEXT STEP] ")?;
        }
        write!(f, "{}", self.hope_node.get_item().get_summary())?;
        for i in self.hope_node.create_parent_chain().iter() {
            write!(f, " ⬅  {}", i.get_summary())?;
        }
        Ok(())
    }
}

impl<'a> From<ProjectHopeItem<'a>> for &'a ItemNode<'a> {
    fn from(value: ProjectHopeItem<'a>) -> Self {
        value.hope_node
    }
}

impl<'a> From<ProjectHopeItem<'a>> for &'a Item<'a> {
    fn from(value: ProjectHopeItem<'a>) -> Self {
        value.hope_node.into()
    }
}

impl<'a> From<&'a ItemNode<'a>> for ProjectHopeItem<'a> {
    fn from(value: &'a ItemNode<'a>) -> Self {
        ProjectHopeItem { hope_node: value }
    }
}

impl<'a> From<ProjectHopeItem<'a>> for &'a SurrealItem {
    fn from(value: ProjectHopeItem<'a>) -> Self {
        value.hope_node.into()
    }
}

impl<'a> From<&'a ProjectHopeItem<'a>> for &'a SurrealItem {
    fn from(value: &'a ProjectHopeItem) -> Self {
        value.hope_node.into()
    }
}

impl<'a> ProjectHopeItem<'a> {
    fn create_list<'b>(hope_nodes: &'b [ItemNode<'b>]) -> Vec<ProjectHopeItem<'b>> {
        hope_nodes.iter().map(|x| x.into()).collect()
    }
}

pub(crate) fn create_hope_nodes<'a>(
    hopes: &[&'a Item<'a>],
    coverings: &'a [Covering<'a>],
    snoozed: &'a [&'a CoveringUntilDateTime<'a>],
    all_items: &'a [&Item<'_>],
) -> Vec<ItemNode<'a>> {
    hopes
        .iter()
        .filter_map(|x| {
            if !x.is_covered_by_a_goal(coverings, all_items) && !x.is_finished() {
                Some(ItemNode::new(x, coverings, snoozed, all_items))
            } else {
                None
            }
        })
        .collect()
}

enum ExpectationsMenuItem {
    DefineFacing,
    MentallyResidentProjects,
    OnDeckProjects,
    IntensionProjects,
    ReleasedProjects,
    MaintenanceItems,
}

impl Display for ExpectationsMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DefineFacing => write!(f, "👀 Define Facing              👀"),
            Self::MentallyResidentProjects => write!(f, "🧠 Mentally Resident Projects 🏗️"),
            Self::OnDeckProjects => write!(f, "🚧 On Deck Projects           🏗️"),
            Self::IntensionProjects => write!(f, "🌠 Intension Projects         🏗️"),
            Self::ReleasedProjects => write!(f, "🔓 Released Projects          🏗️"),
            Self::MaintenanceItems => write!(f, "🔁 Maintenance Items          🔁"),
        }
    }
}

impl ExpectationsMenuItem {
    fn make_list() -> Vec<Self> {
        vec![
            Self::DefineFacing,
            Self::MentallyResidentProjects,
            Self::OnDeckProjects,
            Self::IntensionProjects,
            Self::ReleasedProjects,
            Self::MaintenanceItems,
        ]
    }
}

#[async_recursion]
pub(crate) async fn view_expectations(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = ExpectationsMenuItem::make_list();

    let selection = Select::new("Select from the below list|", list).prompt();

    match selection {
        Ok(ExpectationsMenuItem::DefineFacing) => define_facing(send_to_data_storage_layer).await,
        Ok(ExpectationsMenuItem::MentallyResidentProjects) => {
            view_mentally_resident_project_goals(send_to_data_storage_layer).await
        }
        Ok(ExpectationsMenuItem::OnDeckProjects) => todo!(),
        Ok(ExpectationsMenuItem::IntensionProjects) => todo!(),
        Ok(ExpectationsMenuItem::ReleasedProjects) => todo!(),
        Ok(ExpectationsMenuItem::MaintenanceItems) => {
            view_maintenance_hopes(send_to_data_storage_layer).await
        }
        Err(InquireError::OperationCanceled) => present_top_menu(send_to_data_storage_layer).await,
        Err(err) => todo!("{}", err),
    }
}

#[async_recursion]
pub(crate) async fn view_mentally_resident_project_goals(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();

    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let active_items = base_data.get_active_items();
    let coverings = base_data.get_coverings();
    let active_snoozes = base_data.get_active_snoozed();

    let hopes = active_items
        .filter_just_goals()
        .filter(|x| {
            (x.is_project() || x.is_permanence_not_set())
                && (x.is_mentally_resident() || x.is_staging_not_set())
        })
        .collect::<Vec<_>>();
    let hope_nodes: Vec<ItemNode> =
        create_hope_nodes(&hopes, coverings, active_snoozes, active_items);

    let inquire_list = ProjectHopeItem::create_list(&hope_nodes);

    if !inquire_list.is_empty() {
        let selected = Select::new("Select from the below list|", inquire_list)
            .with_page_size(30)
            .prompt();

        match selected {
            Ok(selected) => {
                present_mentally_resident_goal_selected_menu(
                    selected.into(),
                    send_to_data_storage_layer,
                )
                .await
            }
            Err(err) => match err {
                InquireError::OperationCanceled => {
                    present_top_menu(send_to_data_storage_layer).await
                }
                _ => panic!("Unexpected InquireError of {}", err),
            },
        }
    } else {
        println!("Hope List is Empty, falling back to main menu.");
        present_top_menu(send_to_data_storage_layer).await
    }
}

enum MaintenanceHopeItem<'a> {
    MaintenanceHope(&'a ItemNode<'a>),
}

impl Display for MaintenanceHopeItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaintenanceHope(hope_node) => {
                if hope_node.get_smaller().is_empty() {
                    write!(f, "[NEEDS NEXT STEP] ")?;
                }
                write!(f, "{}", hope_node.get_item().get_summary())?;
                for i in hope_node.create_parent_chain().iter() {
                    write!(f, " ⬅  {}", i.get_summary())?;
                }
                Ok(())
            }
        }
    }
}

impl<'a> MaintenanceHopeItem<'a> {
    fn create_list(hope_nodes: &'a [ItemNode<'a>]) -> Vec<MaintenanceHopeItem<'a>> {
        hope_nodes
            .iter()
            .map(MaintenanceHopeItem::MaintenanceHope)
            .collect()
    }
}

#[async_recursion]
pub(crate) async fn view_maintenance_hopes(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();

    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let active_items = base_data.get_active_items();
    let coverings = base_data.get_coverings();
    let active_snoozes = base_data.get_active_snoozed();

    let hopes = active_items
        .filter_just_goals()
        .filter(|x| x.is_maintenance())
        .collect::<Vec<_>>();
    let hope_nodes = create_hope_nodes(&hopes, coverings, active_snoozes, active_items)
        .into_iter()
        .filter(|x| x.is_maintenance())
        .collect::<Vec<_>>();

    let list = MaintenanceHopeItem::create_list(&hope_nodes);

    if !list.is_empty() {
        let selected = Select::new("Select from the below list|", list).prompt();
        match selected {
            Ok(MaintenanceHopeItem::MaintenanceHope(_hope_node)) => todo!(),
            Err(InquireError::OperationCanceled) => {
                present_top_menu(send_to_data_storage_layer).await
            }
            Err(err) => todo!("{}", err),
        }
    } else {
        println!("Maintenance List is empty, falling back to main menu.");
        present_top_menu(send_to_data_storage_layer).await
    }
}

enum MentallyResidentGoalSelectedMenuItem {
    CoverWithNextStep,
    ProcessAndFinish,
    SwitchToMaintenanceGoal,
    SwitchToOnDeckGoal,
    SwitchToIntensionGoal,
    ReleaseGoal,
    UpdateSummary,
}

impl Display for MentallyResidentGoalSelectedMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CoverWithNextStep => write!(f, "Cover with next step (Action)"),
            Self::ProcessAndFinish => write!(f, "Process and Finish"),
            Self::SwitchToMaintenanceGoal => write!(f, "Switch to a maintenance Goal"),
            Self::SwitchToOnDeckGoal => write!(f, "Switch to on deck Goal"),
            Self::SwitchToIntensionGoal => write!(f, "Switch to intension Goal"),
            Self::ReleaseGoal => write!(f, "Release Goal"),
            Self::UpdateSummary => write!(f, "Update Summary"),
        }
    }
}

impl MentallyResidentGoalSelectedMenuItem {
    fn create_list() -> Vec<MentallyResidentGoalSelectedMenuItem> {
        vec![
            Self::CoverWithNextStep,
            Self::ProcessAndFinish,
            Self::SwitchToMaintenanceGoal,
            Self::SwitchToOnDeckGoal,
            Self::SwitchToIntensionGoal,
            Self::ReleaseGoal,
            Self::UpdateSummary,
        ]
    }
}

#[async_recursion]
pub(crate) async fn present_mentally_resident_goal_selected_menu(
    goal_selected: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = MentallyResidentGoalSelectedMenuItem::create_list();

    let selection = Select::new("Select from the below list|", list)
        .with_page_size(15)
        .prompt();
    match selection {
        Ok(MentallyResidentGoalSelectedMenuItem::CoverWithNextStep) => {
            Ok(cover_with_item(goal_selected, send_to_data_storage_layer).await)
        }
        Ok(MentallyResidentGoalSelectedMenuItem::ProcessAndFinish) => {
            Ok(process_and_finish_goal(goal_selected, send_to_data_storage_layer).await)
        }
        Ok(MentallyResidentGoalSelectedMenuItem::SwitchToMaintenanceGoal) => {
            Ok(switch_to_maintenance_item(goal_selected, send_to_data_storage_layer).await)
        }
        Ok(MentallyResidentGoalSelectedMenuItem::SwitchToOnDeckGoal) => {
            let result = on_deck_query().await;
            match result {
                Ok(staging) => {
                    send_to_data_storage_layer
                        .send(DataLayerCommands::UpdateItemStaging(
                            goal_selected.get_surreal_record_id().clone(),
                            staging,
                        ))
                        .await
                        .unwrap();
                    Ok(())
                }
                Err(InquireError::OperationCanceled) => {
                    present_mentally_resident_goal_selected_menu(
                        goal_selected,
                        send_to_data_storage_layer,
                    )
                    .await
                }
                Err(err) => todo!("Unexpected InquireError of {}", err),
            }
        }
        Ok(MentallyResidentGoalSelectedMenuItem::SwitchToIntensionGoal) => Ok(update_item_staging(
            goal_selected,
            send_to_data_storage_layer,
            Staging::Intension,
        )
        .await),
        Ok(MentallyResidentGoalSelectedMenuItem::ReleaseGoal) => {
            Ok(
                update_item_staging(goal_selected, send_to_data_storage_layer, Staging::Released)
                    .await,
            )
        }
        Ok(MentallyResidentGoalSelectedMenuItem::UpdateSummary) => Ok(update_item_summary(
            goal_selected.get_surreal_record_id().clone(),
            send_to_data_storage_layer,
        )
        .await),
        Err(InquireError::OperationCanceled) => view_expectations(send_to_data_storage_layer).await,
        Err(err) => todo!("{}", err),
    }
}

async fn process_and_finish_goal(
    selected_hope: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let user_processed_text = Editor::new("Process text").prompt().unwrap();

    let surreal_item = selected_hope.get_surreal_record_id();
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

async fn switch_to_maintenance_item(
    selected: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateItemPermanence(
            selected.get_surreal_record_id().clone(),
            Permanence::Maintenance,
        ))
        .await
        .unwrap();
}

async fn update_item_staging(
    selected: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
    new_staging: Staging,
) {
    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateItemStaging(
            selected.get_surreal_record_id().clone(),
            new_staging,
        ))
        .await
        .unwrap();
}
