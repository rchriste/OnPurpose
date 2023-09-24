use std::fmt::Display;

use async_recursion::async_recursion;
use inquire::{InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        convert_linkage_with_record_ids_to_references, Hope, LinkageWithReferences, SurrealItem,
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

fn create_hope_nodes<'a>(
    hopes: &'a [Hope<'a>],
    linkage: &[LinkageWithReferences<'a>],
) -> Vec<HopeNode<'a>> {
    hopes.iter().map(|x| create_hope_node(x, linkage)).collect()
}

fn create_hope_node<'a>(hope: &'a Hope<'a>, linkage: &[LinkageWithReferences<'a>]) -> HopeNode<'a> {
    HopeNode {
        hope,
        next_steps: calculate_next_steps(hope, linkage),
        towards_motivation_chain: build_towards_motivation_chain(hope, linkage),
    }
}

fn calculate_next_steps<'a>(
    hope: &Hope<'a>,
    linkage: &[LinkageWithReferences<'a>],
) -> Vec<&'a SurrealItem> {
    let covered_by = hope.covered_by(linkage);
    covered_by
        .into_iter()
        .flat_map(|x| {
            let covered_by = x.covered_by(linkage);
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
    linkage: &[LinkageWithReferences<'a>],
) -> Vec<&'a SurrealItem> {
    let who_i_am_covering = hope.who_am_i_covering(linkage);
    who_i_am_covering.into_iter().map(|_| todo!()).collect()
}

struct HopeNode<'a> {
    pub hope: &'a Hope<'a>,
    pub next_steps: Vec<&'a SurrealItem>,
    pub towards_motivation_chain: Vec<&'a SurrealItem>,
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
    let (items, linkage) = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();

    let linkage = convert_linkage_with_record_ids_to_references(&linkage, &items);

    let hopes = &items.filter_just_hopes();
    let hope_nodes = create_hope_nodes(hopes, &linkage);

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
    AddToDo,
    CoverWithMilestone,
}

impl Display for HopeSelectedMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HopeSelectedMenuItem::AddToDo => write!(f, "Add to do"),
            HopeSelectedMenuItem::CoverWithMilestone => write!(f, "Cover with milestone"),
        }
    }
}

impl HopeSelectedMenuItem {
    fn create_list() -> Vec<HopeSelectedMenuItem> {
        vec![Self::AddToDo, Self::CoverWithMilestone]
    }
}

async fn present_hope_selected_menu(
    hope_selected: &SurrealItem,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = HopeSelectedMenuItem::create_list();

    let selection = Select::new("Select one", list).prompt().unwrap();
    match selection {
        HopeSelectedMenuItem::AddToDo => {
            present_add_to_do(hope_selected, send_to_data_storage_layer).await
        }
        HopeSelectedMenuItem::CoverWithMilestone => todo!(),
    }
}

enum AddToDoMenuItem {
    NewToDo,
    ExistingToDo,
}

impl Display for AddToDoMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddToDoMenuItem::NewToDo => write!(f, "New To Do"),
            AddToDoMenuItem::ExistingToDo => write!(f, "Existing To Do"),
        }
    }
}

impl AddToDoMenuItem {
    fn create_list() -> Vec<AddToDoMenuItem> {
        vec![Self::NewToDo, Self::ExistingToDo]
    }
}

async fn present_add_to_do(
    hope_to_cover: &SurrealItem,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = AddToDoMenuItem::create_list();

    let selection = Select::new("Select one", list).prompt().unwrap();

    match selection {
        AddToDoMenuItem::NewToDo => {
            present_new_to_do(hope_to_cover, send_to_data_storage_layer).await
        }
        AddToDoMenuItem::ExistingToDo => todo!(),
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