use crate::base_data::{LinkageWithReferences, SurrealItem, ToDo};

pub struct GrowingNode<'a> {
    pub item: &'a SurrealItem,
    pub larger: Vec<GrowingNode<'a>>,
}

impl<'a> GrowingNode<'a> {
    pub fn create_growing_parents(&self) -> Vec<&'a SurrealItem> {
        let mut result = Vec::default();
        for i in self.larger.iter() {
            result.push(i.item);
            let parents = i.create_growing_parents();
            result.extend(parents.iter());
        }
        result
    }
}

pub struct ToDoNode<'a> {
    pub to_do: &'a ToDo<'a>,
    pub larger: Vec<GrowingNode<'a>>,
}

pub fn create_to_do_nodes<'a>(
    next_steps: &'a [ToDo],
    linkage: &'a [LinkageWithReferences<'a>],
) -> Vec<ToDoNode<'a>> {
    next_steps
        .iter()
        .filter_map(|x| {
            if !x.is_covered(linkage) && !x.is_finished() {
                Some(create_to_do_node(x, linkage))
            } else {
                None
            }
        })
        .collect()
}

pub fn create_to_do_node<'a>(
    to_do: &'a ToDo,
    linkage: &'a [LinkageWithReferences<'a>],
) -> ToDoNode<'a> {
    let surreal_item: &SurrealItem = to_do.into();
    let parents = surreal_item.find_parents(linkage);
    let larger = create_growing_nodes(parents, linkage);

    ToDoNode { to_do, larger }
}

pub fn create_growing_nodes<'a>(
    items: Vec<&'a SurrealItem>,
    linkage: &'a [LinkageWithReferences<'a>],
) -> Vec<GrowingNode<'a>> {
    items
        .iter()
        .map(|x| create_growing_node(x, linkage))
        .collect()
}

pub fn create_growing_node<'a>(
    item: &'a SurrealItem,
    linkage: &'a [LinkageWithReferences<'a>],
) -> GrowingNode<'a> {
    let parents = item.find_parents(linkage);
    let larger = create_growing_nodes(parents, linkage);
    GrowingNode { item, larger }
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use surrealdb::sql::Datetime;

    use crate::base_data::{ItemType, SurrealItemVecExtensions};

    use super::*;

    #[test]
    fn new_to_dos_are_shown_in_next_steps() {
        let items = vec![SurrealItem {
            id: None,
            summary: "New item".into(),
            finished: None,
            item_type: ItemType::ToDo,
        }];
        let linkage = vec![];

        let to_dos = items.filter_just_to_dos();
        let next_step_nodes = create_to_do_nodes(&to_dos, &linkage);

        assert_eq!(next_step_nodes.len(), 1);
    }

    #[test]
    fn finished_to_dos_are_not_shown() {
        let items = vec![SurrealItem {
            id: None,
            summary: "Finished item".into(),
            finished: Some(datetime_for_testing()),
            item_type: ItemType::ToDo,
        }];
        let linkage = vec![];

        let to_dos = items.filter_just_to_dos();
        let next_step_nodes = create_to_do_nodes(&to_dos, &linkage);

        assert_eq!(next_step_nodes.len(), 0);
    }

    fn datetime_for_testing() -> Datetime {
        Local::now().naive_utc().and_utc().into()
    }

    #[test]
    fn to_dos_disappear_after_they_are_covered() {
        let items = vec![
            SurrealItem {
                id: None,
                summary: "Covered Item that should not be shown".into(),
                finished: None,
                item_type: ItemType::ToDo,
            },
            SurrealItem {
                id: None,
                summary: "Covering Item that should be shown".into(),
                finished: None,
                item_type: ItemType::ToDo,
            },
        ];
        let linkage = vec![LinkageWithReferences {
            smaller: (&items[1]).into(),
            parent: (&items[0]).into(),
        }];

        let to_dos = items.filter_just_to_dos();
        let next_step_nodes = create_to_do_nodes(&to_dos, &linkage);

        assert_eq!(next_step_nodes.len(), 1);
        assert_eq!(next_step_nodes.iter().next().unwrap().to_do, &items[1]);
    }

    #[test]
    fn to_dos_return_to_next_steps_after_they_are_uncovered() {
        let items = vec![
            SurrealItem {
                id: None,
                summary: "Covered Item to show once the covering item is finished".into(),
                finished: None,
                item_type: ItemType::ToDo,
            },
            SurrealItem {
                id: None,
                summary: "Covering Item that is finished".into(),
                finished: Some(datetime_for_testing()),
                item_type: ItemType::ToDo,
            },
        ];
        let linkage = vec![LinkageWithReferences {
            smaller: (&items[1]).into(),
            parent: (&items[0]).into(),
        }];

        let to_dos = items.filter_just_to_dos();
        let next_step_nodes = create_to_do_nodes(&to_dos, &linkage);

        assert_eq!(next_step_nodes.len(), 1);
        assert_eq!(next_step_nodes.iter().next().unwrap().to_do, &items[0]);
    }
}
