use crate::base_data::{Item, NextStepItem, LinkageWithReferences};

pub struct GrowingNode<'a> {
    pub item: &'a Item<'a>,
    pub larger: Vec<GrowingNode<'a>>,
}

impl<'a> GrowingNode<'a> {
    pub fn create_growing_parents(&self) -> Vec<&'a Item<'a>>
    {
        let mut result: Vec<&'a Item<'a>> = Vec::default();
        for i in self.larger.iter() {
            result.push(i.item);
            let parents = i.create_growing_parents();
            result.extend(parents.iter());
        }
        result
    }
}

pub struct NextStepNode<'a> {
    pub next_step_item: &'a NextStepItem,
    pub larger: Vec<GrowingNode<'a>>
}

pub fn create_next_step_nodes<'a>(next_steps: &'a [NextStepItem], linkage: &'a [LinkageWithReferences<'a>]) -> Vec<NextStepNode<'a>>
{
    next_steps.iter().filter_map(|x| {
        if !x.is_covered(linkage) && !x.is_finished() {
            Some(create_next_step_node(x, linkage))
        } else { None }
    }).collect()
}

pub fn create_next_step_node<'a>(next_step: &'a NextStepItem, linkage: &'a [LinkageWithReferences<'a>]) -> NextStepNode<'a>
{
    let item = Item::NextStepItem(next_step);
    let parents = item.find_parents(linkage);
    let larger = create_growing_nodes(parents, linkage);

    NextStepNode {
        next_step_item: next_step,
        larger
    }
}

pub fn create_growing_nodes<'a>(items: Vec<&'a Item<'a>>, linkage: &'a [LinkageWithReferences<'a>]) -> Vec<GrowingNode<'a>>
{
    items.iter().map(|x| create_growing_node(x, linkage)).collect()
}

pub fn create_growing_node<'a>(item: &'a Item<'a>, linkage: &'a [LinkageWithReferences<'a>]) -> GrowingNode<'a>
{
    let parents = item.find_parents(linkage);
    let larger = create_growing_nodes(parents, linkage);
    GrowingNode {
        item,
        larger
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use surrealdb::sql::Datetime;

    use super::*;

    #[test]
    fn new_items_are_shown_in_next_steps() {
        let next_steps = vec![NextStepItem { id: None, summary: "New item".into(), finished: None }];
        let linkage = vec![];

        let next_step_nodes = create_next_step_nodes(&next_steps, &linkage);

        assert_eq!(next_step_nodes.len(), 1);
    }

    #[test]
    fn finished_items_are_not_shown_in_next_steps() {
        let next_steps = vec![NextStepItem { id: None, summary: "Finished item".into(), finished: Some(datetime_for_testing()) }];
        let linkage = vec![];

        let next_step_nodes = create_next_step_nodes(&next_steps, &linkage);

        assert_eq!(next_step_nodes.len(), 0);
    }

    fn datetime_for_testing() -> Datetime {
        Local::now().naive_utc().and_utc().into()
    }

    #[test]
    fn items_disappear_from_next_steps_after_they_are_covered() {
        let next_steps = vec![
            NextStepItem { id: None, summary: "Covered Item that should not be shown".into(), finished: None },
            NextStepItem { id: None, summary: "Covering Item that should be shown".into(), finished: None},
        ];
        let linkage = vec![
            LinkageWithReferences { smaller: (&next_steps[1]).into(), parent: (&next_steps[0]).into() }
        ];

        let next_step_nodes = create_next_step_nodes(&next_steps, &linkage);

        assert_eq!(next_step_nodes.len(), 1);
        assert_eq!(next_step_nodes.iter().next().unwrap().next_step_item, &next_steps[1]);
    }

    #[test]
    fn items_return_to_next_steps_after_they_are_uncovered() {
        let next_steps = vec![
            NextStepItem { id: None, summary: "Covered Item to show once the covering item is finished".into(), finished: None },
            NextStepItem { id: None, summary: "Covering Item that is finished".into(), finished: Some(datetime_for_testing())},
        ];
        let linkage = vec![
            LinkageWithReferences { smaller: (&next_steps[1]).into(), parent: (&next_steps[0]).into() }
        ];

        let next_step_nodes = create_next_step_nodes(&next_steps, &linkage);

        assert_eq!(next_step_nodes.len(), 1);
        assert_eq!(next_step_nodes.iter().next().unwrap().next_step_item, &next_steps[0]);
    }
}