pub mod base_data;
mod node;
mod test_data;
mod bullet_list;
mod surrealdb_layer;

use base_data::Item;
use inquire::Select;
use node::NextStepNode;
use surrealdb::engine::any::connect;

use crate::{
    node::create_next_step_nodes, 
    bullet_list::InquireBulletListItem, 
    base_data::convert_linkage_with_record_ids_to_references, surrealdb_layer::load_data_from_surrealdb
};

//I get an error about lifetimes that I can't figure out when I refactor this to be a member function of NextStepNode and I don't understand why
fn create_next_step_parents<'a>(item: &'a NextStepNode<'a>) -> Vec<&'a Item<'a>>
{
    let mut result: Vec<&'a Item<'a>> = Vec::default();
    for i in item.larger.iter() {
        result.push(&i.item);
        let parents = i.create_growing_parents();
        result.extend(parents.iter());
    }
    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const CARGO_PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

    println!("Welcome to On-Purpose: Time Management Rethought");
    println!("This is the console prototype using the inquire package");
    println!("Version {}", CARGO_PKG_VERSION.unwrap_or("UNKNOWN"));

    let db = connect("file:://~/.on_purpose.db").await?;
    db.use_ns("OnPurpose").use_db("Russ").await.unwrap();

    let (test_data, linkage) = load_data_from_surrealdb(&db).await;
    let linkage = convert_linkage_with_record_ids_to_references(&linkage, &test_data);

    let next_step_nodes = create_next_step_nodes(&test_data.next_steps, &linkage);

    let inquire_bullet_list = InquireBulletListItem::create_list(&next_step_nodes);

    let selected = Select::new("Select one", inquire_bullet_list).prompt();

    let selected = selected.unwrap();

    println!("{} selected", selected);
    Ok(())
}