use std::fmt::Display;

use async_recursion::async_recursion;
use inquire::{InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        Covering, Hope, Item, ItemVecExtensions, SurrealCoveringVecExtensions, SurrealItem,
        SurrealItemVecExtensions,
    },
    surrealdb_layer::DataLayerCommands,
    top_menu::present_top_menu,
};

struct MentallyResidentItem<'a> {
    pub hope_node: &'a HopeNode<'a>,
}

impl<'a> Display for MentallyResidentItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.hope_node.next_steps.is_empty() {
            write!(f, "[NEEDS NEXT STEP] ")?;
        }
        write!(f, "{}", self.hope_node.hope.summary)?;
        if !self.hope_node.towards_motivation_chain.is_empty() {
            todo!()
        }
        Ok(())
    }
}

impl<'a> From<&'a HopeNode<'a>> for MentallyResidentItem<'a> {
    fn from(value: &'a HopeNode<'a>) -> Self {
        MentallyResidentItem { hope_node: value }
    }
}

impl<'a> From<MentallyResidentItem<'a>> for &'a SurrealItem {
    fn from(value: MentallyResidentItem<'a>) -> Self {
        value.hope_node.into()
    }
}

impl<'a> From<&'a MentallyResidentItem<'a>> for &'a SurrealItem {
    fn from(value: &'a MentallyResidentItem) -> Self {
        value.hope_node.into()
    }
}

impl<'a> MentallyResidentItem<'a> {
    fn create_list<'b>(hope_nodes: &'b [HopeNode<'b>]) -> Vec<MentallyResidentItem<'b>> {
        hope_nodes.iter().map(|x| x.into()).collect()
    }
}

fn create_hope_nodes<'a>(hopes: &'a [Hope<'a>], coverings: &[Covering<'a>]) -> Vec<HopeNode<'a>> {
    hopes
        .iter()
        .map(|x| create_hope_node(x, coverings))
        .collect()
}

fn create_hope_node<'a>(hope: &'a Hope<'a>, coverings: &[Covering<'a>]) -> HopeNode<'a> {
    HopeNode {
        hope,
        next_steps: calculate_next_steps(hope, coverings),
        towards_motivation_chain: build_towards_motivation_chain(hope, coverings),
    }
}

fn calculate_next_steps<'a>(hope: &Hope<'a>, coverings: &[Covering<'a>]) -> Vec<&'a Item<'a>> {
    let covered_by = hope.covered_by(coverings);
    covered_by
        .into_iter()
        .flat_map(|x| {
            let covered_by = x.covered_by(coverings);
            if covered_by.is_empty() {
                vec![x]
            } else {
                todo!()
            }
        })
        .collect()
}

fn build_towards_motivation_chain<'a>(
    hope: &Hope<'a>,
    coverings: &[Covering<'a>],
) -> Vec<&'a Item<'a>> {
    let who_i_am_covering = hope.who_am_i_covering(coverings);
    who_i_am_covering.into_iter().map(|_| todo!()).collect()
}

struct HopeNode<'a> {
    pub hope: &'a Hope<'a>,
    pub next_steps: Vec<&'a Item<'a>>,
    pub towards_motivation_chain: Vec<&'a Item<'a>>,
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

#[async_recursion]
pub async fn view_hopes(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let (items, coverings, requirements) =
        DataLayerCommands::get_raw_data(send_to_data_storage_layer)
            .await
            .unwrap();

    let items = items.make_items(&requirements);
    let coverings = coverings.make_covering(&items);

    let hopes = &items.filter_just_hopes();
    let hope_nodes = create_hope_nodes(hopes, &coverings);

    let inquire_list = MentallyResidentItem::create_list(&hope_nodes);

    if !inquire_list.is_empty() {
        let selected = Select::new("Select one", inquire_list).prompt();

        match selected {
            Ok(selected) => {
                present_hope_selected_menu(selected.into(), send_to_data_storage_layer).await;
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

enum HopeSelectedMenuItem {
    CoverWithNextStep,
    CoverWithMilestone,
}

impl Display for HopeSelectedMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HopeSelectedMenuItem::CoverWithNextStep => write!(f, "Cover with next step (To Do)"),
            HopeSelectedMenuItem::CoverWithMilestone => write!(f, "Cover with milestone (Hope)"),
        }
    }
}

impl HopeSelectedMenuItem {
    fn create_list() -> Vec<HopeSelectedMenuItem> {
        vec![Self::CoverWithNextStep, Self::CoverWithMilestone]
    }
}

async fn present_hope_selected_menu(
    hope_selected: &SurrealItem,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = HopeSelectedMenuItem::create_list();

    let selection = Select::new("Select one", list).prompt().unwrap();
    match selection {
        HopeSelectedMenuItem::CoverWithNextStep => {
            present_add_next_step(hope_selected, send_to_data_storage_layer).await
        }
        HopeSelectedMenuItem::CoverWithMilestone => present_add_milestone().await,
    }
}

enum AddNextStepMenuItem {
    NewToDo,
    ExistingToDo,
}

impl Display for AddNextStepMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddNextStepMenuItem::NewToDo => write!(f, "New To Do"),
            AddNextStepMenuItem::ExistingToDo => write!(f, "Existing To Do"),
        }
    }
}

impl AddNextStepMenuItem {
    fn create_list() -> Vec<Self> {
        vec![Self::NewToDo, Self::ExistingToDo]
    }
}

async fn present_add_next_step(
    hope_to_cover: &SurrealItem,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = AddNextStepMenuItem::create_list();

    let selection = Select::new("Select one", list).prompt().unwrap();

    match selection {
        AddNextStepMenuItem::NewToDo => {
            present_new_to_do(hope_to_cover, send_to_data_storage_layer).await
        }
        AddNextStepMenuItem::ExistingToDo => todo!(),
    }
}

async fn present_new_to_do(
    hope_to_cover: &SurrealItem,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let new_hope_text = Text::new("Enter To Do (i.e. Next Step) ⍠")
        .prompt()
        .unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::CoverItemWithANewToDo(
            hope_to_cover.clone(),
            new_hope_text,
        ))
        .await
        .unwrap();
}

async fn present_add_milestone()
{
    todo!()
}