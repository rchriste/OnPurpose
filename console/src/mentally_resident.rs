use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::Utc;
use inquire::{Editor, InquireError, Select, Text};
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
        bullet_list_menu::bullet_list_single_item::cover_with_item, staging_query::on_deck_query,
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
            if !x.is_covered_by_a_hope(coverings, all_items) && !x.is_finished() {
                Some(ItemNode::new(x, coverings, snoozed, all_items))
            } else {
                None
            }
        })
        .collect()
}

enum HopeMenuItem {
    MentallyResidentProjects,
    OnDeckProjects,
    IntensionProjects,
    ReleasedProjects,
    MaintenanceItems,
}

impl Display for HopeMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MentallyResidentProjects => write!(f, "🧠 Mentally Resident Projects 🏗️"),
            Self::OnDeckProjects => write!(f, "🚧 On Deck Projects           🏗️"),
            Self::IntensionProjects => write!(f, "🌠 Intension Projects         🏗️"),
            Self::ReleasedProjects => write!(f, "🔓 Released Projects          🏗️"),
            Self::MaintenanceItems => write!(f, "🔁 Maintenance Items          🔁"),
        }
    }
}

impl HopeMenuItem {
    fn make_list() -> Vec<HopeMenuItem> {
        vec![
            Self::MentallyResidentProjects,
            Self::OnDeckProjects,
            Self::IntensionProjects,
            Self::ReleasedProjects,
            Self::MaintenanceItems,
        ]
    }
}

#[async_recursion]
pub(crate) async fn view_hopes(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let list = HopeMenuItem::make_list();

    let selection = Select::new("Select from the below list", list).prompt();

    match selection {
        Ok(HopeMenuItem::MentallyResidentProjects) => {
            view_mentally_resident_project_hopes(send_to_data_storage_layer).await
        }
        Ok(HopeMenuItem::OnDeckProjects) => todo!(),
        Ok(HopeMenuItem::IntensionProjects) => todo!(),
        Ok(HopeMenuItem::ReleasedProjects) => todo!(),
        Ok(HopeMenuItem::MaintenanceItems) => {
            view_maintenance_hopes(send_to_data_storage_layer).await
        }
        Err(InquireError::OperationCanceled) => present_top_menu(send_to_data_storage_layer).await,
        Err(err) => todo!("{}", err),
    }
}

#[async_recursion]
pub(crate) async fn view_mentally_resident_project_hopes(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
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
        let selected = Select::new("Select from the below list", inquire_list)
            .with_page_size(30)
            .prompt();

        match selected {
            Ok(selected) => {
                present_mentally_resident_hope_selected_menu(
                    selected.into(),
                    send_to_data_storage_layer,
                )
                .await;
            }
            Err(err) => match err {
                InquireError::OperationCanceled => {
                    present_top_menu(send_to_data_storage_layer).await
                }
                _ => panic!("Unexpected InquireError of {}", err),
            },
        };
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
pub(crate) async fn view_maintenance_hopes(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
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
        let selected = Select::new("Select from the below list", list).prompt();
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

enum MentallyResidentHopeSelectedMenuItem {
    CoverWithNextStep,
    CoverWithMilestone,
    ProcessAndFinish,
    SwitchToMaintenanceHope,
    SwitchToOnDeckHope,
    SwitchToIntensionHope,
    ReleaseHope,
    UpdateSummary,
}

impl Display for MentallyResidentHopeSelectedMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CoverWithNextStep => write!(f, "Cover with next step (To Do)"),
            Self::CoverWithMilestone => write!(f, "Cover with milestone (Hope)"),
            Self::ProcessAndFinish => write!(f, "Process and Finish"),
            Self::SwitchToMaintenanceHope => write!(f, "Switch to a maintenance Hope"),
            Self::SwitchToOnDeckHope => write!(f, "Switch to on deck Hope"),
            Self::SwitchToIntensionHope => write!(f, "Switch to intension Hope"),
            Self::ReleaseHope => write!(f, "Release Hope"),
            Self::UpdateSummary => write!(f, "Update Summary"),
        }
    }
}

impl MentallyResidentHopeSelectedMenuItem {
    fn create_list() -> Vec<MentallyResidentHopeSelectedMenuItem> {
        vec![
            Self::CoverWithNextStep,
            Self::CoverWithMilestone,
            Self::ProcessAndFinish,
            Self::SwitchToMaintenanceHope,
            Self::SwitchToOnDeckHope,
            Self::SwitchToIntensionHope,
            Self::ReleaseHope,
            Self::UpdateSummary,
        ]
    }
}

#[async_recursion]
pub(crate) async fn present_mentally_resident_hope_selected_menu(
    hope_selected: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = MentallyResidentHopeSelectedMenuItem::create_list();

    let selection = Select::new("Select from the below list", list)
        .with_page_size(15)
        .prompt();
    match selection {
        Ok(MentallyResidentHopeSelectedMenuItem::CoverWithNextStep) => {
            cover_with_item(hope_selected, send_to_data_storage_layer).await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::CoverWithMilestone) => {
            present_add_milestone(hope_selected, send_to_data_storage_layer).await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::ProcessAndFinish) => {
            process_and_finish_hope(hope_selected, send_to_data_storage_layer).await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::SwitchToMaintenanceHope) => {
            switch_to_maintenance_item(hope_selected, send_to_data_storage_layer).await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::SwitchToOnDeckHope) => {
            let result = on_deck_query().await;
            match result {
                Ok(staging) => {
                    send_to_data_storage_layer
                        .send(DataLayerCommands::UpdateItemStaging(
                            hope_selected.get_surreal_record_id().clone(),
                            staging,
                        ))
                        .await
                        .unwrap();
                }
                Err(InquireError::OperationCanceled) => {
                    present_mentally_resident_hope_selected_menu(
                        hope_selected,
                        send_to_data_storage_layer,
                    )
                    .await;
                }
                Err(err) => todo!("Unexpected InquireError of {}", err),
            }
        }
        Ok(MentallyResidentHopeSelectedMenuItem::SwitchToIntensionHope) => {
            update_item_staging(
                hope_selected,
                send_to_data_storage_layer,
                Staging::Intension,
            )
            .await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::ReleaseHope) => {
            update_item_staging(hope_selected, send_to_data_storage_layer, Staging::Released).await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::UpdateSummary) => {
            update_item_summary(
                hope_selected.get_surreal_item().clone(),
                send_to_data_storage_layer,
            )
            .await
        }
        Err(InquireError::OperationCanceled) => view_hopes(send_to_data_storage_layer).await,
        Err(err) => todo!("{}", err),
    }
}

enum AddMilestoneMenuItem {
    NewMilestone,
    ExistingMilestone,
}

impl Display for AddMilestoneMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddMilestoneMenuItem::NewMilestone => write!(f, "New Milestone"),
            AddMilestoneMenuItem::ExistingMilestone => write!(f, "Existing Milestone"),
        }
    }
}

impl AddMilestoneMenuItem {
    fn make_list() -> Vec<Self> {
        vec![Self::NewMilestone, Self::ExistingMilestone]
    }
}

async fn present_add_milestone(
    selected_hope: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = AddMilestoneMenuItem::make_list();

    let selection = Select::new("Select from the below list", list)
        .prompt()
        .unwrap();

    match selection {
        AddMilestoneMenuItem::NewMilestone => {
            cover_hope_with_new_milestone(selected_hope, send_to_data_storage_layer).await
        }
        AddMilestoneMenuItem::ExistingMilestone => cover_hope_with_existing_milestone().await,
    }
}

async fn cover_hope_with_new_milestone(
    existing_hope: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let new_milestone_text = Text::new("Enter milestone (Hope) ⍠").prompt().unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::CoverItemWithANewMilestone(
            existing_hope.get_surreal_item().clone(),
            new_milestone_text,
        ))
        .await
        .unwrap();
}

async fn process_and_finish_hope(
    selected_hope: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let user_processed_text = Editor::new("Process text").prompt().unwrap();

    let surreal_item = selected_hope.get_surreal_item();
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
            selected.get_surreal_item().clone(),
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

async fn cover_hope_with_existing_milestone() {
    todo!()
}
