use std::fmt::Display;

use async_recursion::async_recursion;
use inquire::{Editor, InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        hope::Hope,
        item::{Item, ItemVecExtensions},
        Covering,
    },
    menu::bullet_list::bullet_list_single_item::cover_with_item,
    menu::top_menu::present_top_menu,
    surrealdb_layer::{
        surreal_item::SurrealItem,
        surreal_specific_to_hope::{Permanence, Staging},
        DataLayerCommands,
    },
    update_item_summary,
};

struct ProjectHopeItem<'a> {
    pub(crate) hope_node: &'a HopeNode<'a>,
}

impl<'a> Display for ProjectHopeItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.hope_node.next_steps.is_empty() {
            write!(f, "[NEEDS NEXT STEP] ")?;
        }
        write!(f, "{}", self.hope_node.hope.summary)?;
        for i in self.hope_node.towards_motivation_chain.iter() {
            write!(f, " ⬅  {}", i.summary)?;
        }
        Ok(())
    }
}

impl<'a> From<ProjectHopeItem<'a>> for &'a Hope<'a> {
    fn from(value: ProjectHopeItem<'a>) -> Self {
        value.hope_node.hope
    }
}

impl<'a> From<&'a HopeNode<'a>> for ProjectHopeItem<'a> {
    fn from(value: &'a HopeNode<'a>) -> Self {
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
    fn create_list<'b>(hope_nodes: &'b [HopeNode<'b>]) -> Vec<ProjectHopeItem<'b>> {
        hope_nodes.iter().map(|x| x.into()).collect()
    }
}

pub(crate) fn create_hope_nodes<'a>(
    hopes: &'a [Hope<'a>],
    coverings: &[Covering<'a>],
) -> Vec<HopeNode<'a>> {
    hopes
        .iter()
        .filter_map(|x| {
            if !x.is_covered_by_another_hope(coverings) && !x.is_finished() {
                Some(create_hope_node(x, coverings))
            } else {
                None
            }
        })
        .collect()
}

fn create_hope_node<'a>(hope: &'a Hope<'a>, coverings: &[Covering<'a>]) -> HopeNode<'a> {
    HopeNode {
        hope,
        next_steps: calculate_next_steps(hope, coverings),
        towards_motivation_chain: build_towards_motivation_chain(hope.get_item(), coverings),
    }
}

fn calculate_next_steps<'a>(hope: &Hope<'a>, coverings: &[Covering<'a>]) -> Vec<&'a Item<'a>> {
    let covered_by = hope.covered_by(coverings);
    covered_by
        .into_iter()
        .flat_map(|x| {
            let mut covered_by = vec![x];
            covered_by.extend(x.covered_by(coverings));
            covered_by
        })
        .collect()
}

fn build_towards_motivation_chain<'a>(
    item: &Item<'a>,
    coverings: &[Covering<'a>],
) -> Vec<&'a Item<'a>> {
    let who_i_am_covering = item.who_am_i_covering(coverings);
    who_i_am_covering
        .into_iter()
        .flat_map(|x| {
            let mut v = vec![x];
            v.extend(build_towards_motivation_chain(x, coverings));
            v
        })
        .collect()
}

pub(crate) struct HopeNode<'a> {
    pub(crate) hope: &'a Hope<'a>,
    pub(crate) next_steps: Vec<&'a Item<'a>>,
    pub(crate) towards_motivation_chain: Vec<&'a Item<'a>>,
}

impl<'a> From<&'a HopeNode<'a>> for &'a Hope<'a> {
    fn from(value: &HopeNode<'a>) -> Self {
        value.hope
    }
}

impl<'a> From<&'a HopeNode<'a>> for &'a SurrealItem {
    fn from(value: &'a HopeNode<'a>) -> Self {
        value.hope.into()
    }
}

impl<'a> HopeNode<'a> {
    pub(crate) fn is_maintenance(&self) -> bool {
        self.hope.is_maintenance()
    }

    pub(crate) fn is_project(&self) -> bool {
        self.hope.is_project()
    }
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

    let selection = Select::new("", list).prompt();

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
    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();

    let items = surreal_tables.make_items();
    let coverings = surreal_tables.make_coverings(&items);

    let hopes: Vec<Hope<'_>> = items
        .filter_just_hopes(&surreal_tables.surreal_specific_to_hopes)
        .into_iter()
        .filter(|x| x.is_project() && x.is_mentally_resident())
        .collect();
    let hope_nodes: Vec<HopeNode> = create_hope_nodes(&hopes, &coverings)
        .into_iter()
        .filter(|x| x.is_project())
        .collect();

    let inquire_list = ProjectHopeItem::create_list(&hope_nodes);

    if !inquire_list.is_empty() {
        let selected = Select::new("", inquire_list).with_page_size(30).prompt();

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
    MaintenanceHope(&'a HopeNode<'a>),
}

impl Display for MaintenanceHopeItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaintenanceHope(hope_node) => {
                if hope_node.next_steps.is_empty() {
                    write!(f, "[NEEDS NEXT STEP] ")?;
                }
                write!(f, "{}", hope_node.hope.summary)?;
                for i in hope_node.towards_motivation_chain.iter() {
                    write!(f, " ⬅  {}", i.summary)?;
                }
                Ok(())
            }
        }
    }
}

impl<'a> MaintenanceHopeItem<'a> {
    fn create_list(hope_nodes: &'a [HopeNode<'a>]) -> Vec<MaintenanceHopeItem<'a>> {
        hope_nodes
            .iter()
            .map(MaintenanceHopeItem::MaintenanceHope)
            .collect()
    }
}

#[async_recursion]
pub(crate) async fn view_maintenance_hopes(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();

    let items = surreal_tables.make_items();
    let coverings = surreal_tables.make_coverings(&items);

    let hopes: Vec<Hope<'_>> = items
        .filter_just_hopes(&surreal_tables.surreal_specific_to_hopes)
        .into_iter()
        .filter(|x| x.is_maintenance())
        .collect();
    let hope_nodes: Vec<HopeNode> = create_hope_nodes(&hopes, &coverings)
        .into_iter()
        .filter(|x| x.is_maintenance())
        .collect();

    let list = MaintenanceHopeItem::create_list(&hope_nodes);

    if !list.is_empty() {
        let selected = Select::new("", list).prompt();
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

pub(crate) async fn present_mentally_resident_hope_selected_menu(
    hope_selected: &Hope<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = MentallyResidentHopeSelectedMenuItem::create_list();

    let selection = Select::new("", list).with_page_size(15).prompt();
    match selection {
        Ok(MentallyResidentHopeSelectedMenuItem::CoverWithNextStep) => {
            cover_with_item(hope_selected.get_item(), send_to_data_storage_layer).await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::CoverWithMilestone) => {
            present_add_milestone(hope_selected, send_to_data_storage_layer).await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::ProcessAndFinish) => {
            process_and_finish_hope(hope_selected, send_to_data_storage_layer).await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::SwitchToMaintenanceHope) => {
            switch_to_maintenance_hope(hope_selected, send_to_data_storage_layer).await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::SwitchToOnDeckHope) => {
            update_hope_staging(hope_selected, send_to_data_storage_layer, Staging::OnDeck).await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::SwitchToIntensionHope) => {
            update_hope_staging(
                hope_selected,
                send_to_data_storage_layer,
                Staging::Intension,
            )
            .await
        }
        Ok(MentallyResidentHopeSelectedMenuItem::ReleaseHope) => {
            update_hope_staging(hope_selected, send_to_data_storage_layer, Staging::Released).await
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
    selected_hope: &Hope<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = AddMilestoneMenuItem::make_list();

    let selection = Select::new("", list).prompt().unwrap();

    match selection {
        AddMilestoneMenuItem::NewMilestone => {
            cover_hope_with_new_milestone(selected_hope, send_to_data_storage_layer).await
        }
        AddMilestoneMenuItem::ExistingMilestone => cover_hope_with_existing_milestone().await,
    }
}

async fn cover_hope_with_new_milestone(
    existing_hope: &Hope<'_>,
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
    selected_hope: &Hope<'_>,
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

async fn switch_to_maintenance_hope(
    selected_hope: &Hope<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateHopePermanence(
            selected_hope.hope_specific.clone(),
            Permanence::Maintenance,
        ))
        .await
        .unwrap();
}

async fn update_hope_staging(
    selected_hope: &Hope<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
    new_staging: Staging,
) {
    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateHopeStaging(
            selected_hope.hope_specific.clone(),
            new_staging,
        ))
        .await
        .unwrap();
}

async fn cover_hope_with_existing_milestone() {
    todo!()
}
