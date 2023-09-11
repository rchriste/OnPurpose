use std::fmt::Display;

use inquire::Select;

#[derive(PartialEq, Eq)]
struct NextStepItem {
    summary: String,
}

/// Could have a review_type with options for Milestone, StoppingPoint, and ReviewPoint
#[derive(PartialEq, Eq)]
struct ReviewItem {
    summary: String,
}

/// Could have a reason_type with options for Commitment, Maintenance, or Value
#[derive(PartialEq, Eq)]
struct ReasonItem {
    summary: String,
}

struct Linkage<'a> {
    smaller: Item<'a>,
    parent: Item<'a>,
}

struct GrowingNode<'a> {
    item: &'a Item<'a>,
    larger: Vec<GrowingNode<'a>>,
}

struct NextStepNode<'a> {
    next_step_item: &'a NextStepItem,
    larger: Vec<GrowingNode<'a>>
}

#[derive(PartialEq, Eq)]
enum Item<'a> {
    NextStepItem(&'a NextStepItem),
    ReviewItem(&'a ReviewItem),
    ReasonItem(&'a ReasonItem)
}

fn create_next_step_nodes<'a>(next_steps: &'a Vec<NextStepItem>, linkage: &'a Vec<Linkage<'a>>) -> Vec<NextStepNode<'a>>
{
    next_steps.iter().filter_map(|x| {
        if !is_covered(&x, &linkage) {
            Some(create_next_step_node(x, &linkage))
        } else { None }
    }).collect()
}

fn is_covered(next_step_item: &NextStepItem, linkage: &Vec<Linkage<'_>>) -> bool {
    let next_step_item = Item::NextStepItem(&next_step_item);
    linkage.iter().any(|x| x.parent == next_step_item)
}

fn create_next_step_node<'a>(next_step: &'a NextStepItem, linkage: &'a Vec<Linkage<'a>>) -> NextStepNode<'a>
{
    let item = Item::NextStepItem(&next_step);
    let parents = find_parents(&item, &linkage);
    let larger = create_growing_nodes(parents, &linkage);

    NextStepNode {
        next_step_item: &next_step,
        larger
    }
}

fn create_growing_nodes<'a>(items: Vec<&'a Item<'a>>, linkage: &'a Vec<Linkage<'a>>) -> Vec<GrowingNode<'a>>
{
    items.iter().map(|x| create_growing_node(x, &linkage)).collect()
}

fn create_growing_node<'a>(item: &'a Item<'a>, linkage: &'a Vec<Linkage<'a>>) -> GrowingNode<'a>
{
    let parents = find_parents(item, &linkage);
    let larger = create_growing_nodes(parents, linkage);
    GrowingNode {
        item,
        larger
    }
}

fn find_parents<'a>(item: &Item<'a>, linkage: &'a Vec<Linkage<'a>>) -> Vec<&'a Item<'a>>
{
    linkage.iter().filter_map(|x| {
        if &x.smaller == item {Some(&x.parent)}
        else {None}
    }).collect()
}

struct InquireBulletListItem<'a> {
    bullet_item: &'a NextStepItem,
    parents: Vec<&'a Item<'a>>,
}

impl Display for InquireBulletListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.bullet_item.summary)?;
        for item in &self.parents {
            match item {
                Item::NextStepItem(next_step) => write!(f, "⬅ 🪜  {}", &next_step.summary)?,
                Item::ReviewItem(review) => write!(f, "⬅ 🧠 {}", &review.summary)?,
                Item::ReasonItem(reason) => write!(f, "⬅ 🎁 {}", &reason.summary)?,
            }
        }
        Ok(())
    }
}

impl<'a> InquireBulletListItem<'a> {
    fn create_list(next_step_nodes: &'a Vec<NextStepNode<'a>>) -> Vec<InquireBulletListItem<'a>>
    {
        next_step_nodes.iter().map(|x| {
            InquireBulletListItem {
                bullet_item: x.next_step_item,
                parents: create_next_step_parents(&x),
            }
        }).collect()
    }
}

fn create_next_step_parents<'a>(item: &'a NextStepNode) -> Vec<&'a Item<'a>>
{
    let mut result: Vec<&'a Item<'a>> = Vec::default();
    for i in item.larger.iter() {
        result.push(&i.item);
        let parents = create_growing_parents(&i);
        result.extend(parents.iter());
    }
    result
}

fn create_growing_parents<'a>(item: &'a GrowingNode) -> Vec<&'a Item<'a>>
{
    let mut result: Vec<&'a Item<'a>> = Vec::default();
    for i in item.larger.iter() {
        result.push(&i.item);
        let parents = create_growing_parents(&i);
        result.extend(parents.iter());
    }
    result
}

fn main() {
    const CARGO_PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

    println!("Welcome to On-Purpose: Time Management Rethought");
    println!("This is the console prototype using the inquire package");
    println!("Version {}", CARGO_PKG_VERSION.unwrap_or("UNKNOWN"));

    let next_steps = vec![
        NextStepItem {
            summary: String::from("Clean Dometic")
        },
        NextStepItem {
            summary: String::from("Fill out SafeAccess Health & Safety Invitation for RustConf 2023")
        },
        NextStepItem {
            summary: String::from("Get a Covid vaccine")
        },
    ];

    let review_items = vec![
        ReviewItem {
            summary: String::from("Go camping")
        },
        ReviewItem {
            summary: String::from("After")
        },

        ReviewItem {
            summary: String::from("Attend Rust conference")
        },
        ReviewItem {
            summary: String::from("Prepare")
        }
    ];

    let reason_items = vec![
        ReasonItem {
            summary: String::from("Family Trips")
        },
        ReasonItem {
            summary: String::from("On-Purpose")
        }
    ];

    let linkage = vec![
        //NEXT STEPS
        Linkage {
            parent: Item::NextStepItem(&next_steps[1]),
            smaller: Item::NextStepItem(&next_steps[2]),
        },
        //NEXT STEPS to REVIEW ITEMS
        Linkage {
            parent: Item::ReviewItem(&review_items[1]),
            smaller: Item::NextStepItem(&next_steps[0]),
        },
        Linkage {
            parent: Item::ReviewItem(&review_items[0]),
            smaller: Item::ReviewItem(&review_items[1])
        },
        Linkage {
            parent: Item::ReviewItem(&review_items[2]),
            smaller: Item::ReviewItem(&review_items[3]),
        },
        Linkage {
            parent: Item::ReviewItem(&review_items[3]),
            smaller: Item::NextStepItem(&next_steps[1]),
        },
        //REVIEW STEPS to REASONS
        Linkage {
            parent: Item::ReasonItem(&reason_items[0]),
            smaller: Item::ReviewItem(&review_items[0]),
        },
        Linkage {
            parent: Item::ReasonItem(&reason_items[1]),
            smaller: Item::ReviewItem(&review_items[2]),
        },
    ];


    let next_step_nodes = create_next_step_nodes(&next_steps, &linkage);

    let inquire_bullet_list = InquireBulletListItem::create_list(&next_step_nodes);

    let selected = Select::new("Select one", inquire_bullet_list).prompt();

    let selected = selected.unwrap();

    println!("{} selected", selected);
}