use crate::base_data::{Item, LinkageWithReferences, ToDo};

pub struct GrowingNode<'a> {
    pub item: &'a Item<'a>,
    pub larger: Vec<GrowingNode<'a>>,
}

impl<'a> GrowingNode<'a> {
    pub fn create_growing_parents(&self) -> Vec<&'a Item<'a>> {
        let mut result: Vec<&'a Item<'a>> = Vec::default();
        for i in self.larger.iter() {
            result.push(i.item);
            let parents = i.create_growing_parents();
            result.extend(parents.iter());
        }
        result
    }
}

pub struct ToDoNode<'a> {
    pub to_do: &'a ToDo,
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
    let item = Item::ToDo(to_do);
    let parents = item.find_parents(linkage);
    let larger = create_growing_nodes(parents, linkage);

    ToDoNode { to_do, larger }
}

pub fn create_growing_nodes<'a>(
    items: Vec<&'a Item<'a>>,
    linkage: &'a [LinkageWithReferences<'a>],
) -> Vec<GrowingNode<'a>> {
    items
        .iter()
        .map(|x| create_growing_node(x, linkage))
        .collect()
}

pub fn create_growing_node<'a>(
    item: &'a Item<'a>,
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

    use super::*;

    #[test]
    fn new_to_dos_are_shown_in_next_steps() {
        let next_steps = vec![ToDo {
            id: None,
            summary: "New item".into(),
            finished: None,
        }];
        let linkage = vec![];

        let next_step_nodes = create_to_do_nodes(&next_steps, &linkage);

        assert_eq!(next_step_nodes.len(), 1);
    }

    #[test]
    fn finished_to_dos_are_not_shown() {
        let next_steps = vec![ToDo {
            id: None,
            summary: "Finished item".into(),
            finished: Some(datetime_for_testing()),
        }];
        let linkage = vec![];

        let next_step_nodes = create_to_do_nodes(&next_steps, &linkage);

        assert_eq!(next_step_nodes.len(), 0);
    }

    fn datetime_for_testing() -> Datetime {
        Local::now().naive_utc().and_utc().into()
    }

    #[test]
    fn to_dos_disappear_after_they_are_covered() {
        let to_dos = vec![
            ToDo {
                id: None,
                summary: "Covered Item that should not be shown".into(),
                finished: None,
            },
            ToDo {
                id: None,
                summary: "Covering Item that should be shown".into(),
                finished: None,
            },
        ];
        let linkage = vec![LinkageWithReferences {
            smaller: (&to_dos[1]).into(),
            parent: (&to_dos[0]).into(),
        }];

        let next_step_nodes = create_to_do_nodes(&to_dos, &linkage);

        assert_eq!(next_step_nodes.len(), 1);
        assert_eq!(next_step_nodes.iter().next().unwrap().to_do, &to_dos[1]);
    }

    #[test]
    fn to_dos_return_to_next_steps_after_they_are_uncovered() {
        let next_steps = vec![
            ToDo {
                id: None,
                summary: "Covered Item to show once the covering item is finished".into(),
                finished: None,
            },
            ToDo {
                id: None,
                summary: "Covering Item that is finished".into(),
                finished: Some(datetime_for_testing()),
            },
        ];
        let linkage = vec![LinkageWithReferences {
            smaller: (&next_steps[1]).into(),
            parent: (&next_steps[0]).into(),
        }];

        let next_step_nodes = create_to_do_nodes(&next_steps, &linkage);

        assert_eq!(next_step_nodes.len(), 1);
        assert_eq!(next_step_nodes.iter().next().unwrap().to_do, &next_steps[0]);
    }
}
