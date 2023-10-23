use chrono::{DateTime, Local};

use crate::base_data::{item::Item, to_do::ToDo, Covering, CoveringUntilDateTime};

use super::{create_growing_nodes, GrowingItemNode};

pub(crate) struct ToDoNode<'s> {
    pub(crate) to_do: &'s ToDo<'s>,
    pub(crate) larger: Vec<GrowingItemNode<'s>>,
}

impl<'s> ToDoNode<'s> {
    pub(crate) fn create_next_step_parents(&'s self) -> Vec<&'s Item<'s>> {
        let mut result = Vec::default();
        for i in self.larger.iter() {
            result.push(i.item);
            let parents = i.create_growing_parents();
            result.extend(parents.iter());
        }
        result
    }
}

pub(crate) fn create_to_do_nodes<'a>(
    next_steps: &'a [ToDo],
    coverings: &'a [Covering<'a>],
    coverings_until_date_time: &'a [CoveringUntilDateTime<'a>],
    possible_parents: &'a [&'a Item<'a>],
    current_date: &DateTime<Local>,
    currently_in_focus_time: bool,
) -> Vec<ToDoNode<'a>> {
    next_steps
        .iter()
        .filter_map(|x| {
            if !x.is_covered(coverings, coverings_until_date_time, current_date)
                && !x.is_finished()
                && x.is_circumstances_met(current_date, currently_in_focus_time)
            {
                Some(create_to_do_node(x, coverings, possible_parents))
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn create_to_do_node<'a>(
    to_do: &'a ToDo,
    coverings: &'a [Covering<'a>],
    possible_parents: &'a [&'a Item<'a>],
) -> ToDoNode<'a> {
    let item: &Item = to_do.into();
    let parents = item.find_parents(coverings, possible_parents);
    let larger = create_growing_nodes(parents, coverings, possible_parents);

    ToDoNode { to_do, larger }
}

impl<'a> ToDoNode<'a> {
    #[allow(dead_code)]
    pub(crate) fn get_summary(&'a self) -> &'a str {
        self.to_do.get_summary()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::{DateTime, Local, Utc};
    use surrealdb::sql::Datetime;

    use crate::{
        base_data::{item::ItemVecExtensions, ItemType},
        node::to_do_node::create_to_do_nodes,
        surrealdb_layer::{
            surreal_covering::SurrealCovering,
            surreal_covering_until_date_time::SurrealCoveringUntilDatetime,
            surreal_item::{Responsibility, SurrealItem, SurrealOrderedSubItem},
            surreal_required_circumstance::{CircumstanceType, SurrealRequiredCircumstance},
            SurrealTables,
        },
    };

    use super::*;

    #[test]
    fn new_to_dos_are_shown_in_next_steps() {
        let surreal_tables = SurrealTables {
            surreal_items: vec![SurrealItem {
                id: Some(("surreal_item", "1").into()),
                summary: "New item".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            }],
            surreal_specific_to_hopes: vec![],
            surreal_coverings: vec![],
            surreal_required_circumstances: vec![],
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };

        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

        let to_dos = items.filter_just_to_dos();
        let wednesday_ignore =
            DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();
        let next_step_nodes = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &wednesday_ignore,
            false,
        );

        assert_eq!(next_step_nodes.len(), 1);
    }

    #[test]
    fn finished_to_dos_are_not_shown() {
        let surreal_tables = SurrealTables {
            surreal_items: vec![SurrealItem {
                id: Some(("surreal_item", "1").into()),
                summary: "Finished item".into(),
                finished: Some(undefined_datetime_for_testing()),
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            }],
            surreal_specific_to_hopes: vec![],
            surreal_coverings: vec![],
            surreal_required_circumstances: vec![],
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };

        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

        let to_dos = items.filter_just_to_dos();
        let wednesday_ignore =
            DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();
        let next_step_nodes = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &wednesday_ignore,
            false,
        );

        assert_eq!(next_step_nodes.len(), 0);
    }

    #[test]
    fn require_not_sunday_are_not_shown_on_sunday() {
        let surreal_items = vec![SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Can't do this on Sunday".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: vec![],
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        }];
        let surreal_required_circumstances = vec![SurrealRequiredCircumstance {
            id: Some(("surreal_requirement", "1").into()),
            required_for: surreal_items.first().unwrap().id.as_ref().unwrap().clone(),
            circumstance_type: CircumstanceType::NotSunday,
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings: vec![],
            surreal_required_circumstances,
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };

        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);
        let to_dos = items.filter_just_to_dos();
        let sunday =
            DateTime::parse_from_str("1983 Apr 17 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();
        let next_step_nodes = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &sunday,
            false,
        );

        assert!(next_step_nodes.is_empty()); //Not shown because it is Sunday
    }

    #[test]
    fn require_not_sunday_are_shown_on_a_weekday() {
        let surreal_items = vec![SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Can't do this on Sunday".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: vec![],
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        }];
        let surreal_required_circumstances = vec![SurrealRequiredCircumstance {
            id: Some(("surreal_requirement", "1").into()),
            required_for: surreal_items.first().unwrap().id.as_ref().unwrap().clone(),
            circumstance_type: CircumstanceType::NotSunday,
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings: vec![],
            surreal_required_circumstances,
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);
        let to_dos = items.filter_just_to_dos();
        let wednesday =
            DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();
        let next_step_nodes = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &wednesday,
            false,
        );

        assert_eq!(1, next_step_nodes.len());
    }

    #[test]
    fn require_not_sunday_are_shown_on_a_saturday() {
        let surreal_items = vec![SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Can't do this on Sunday".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: vec![],
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        }];
        let surreal_required_circumstances = vec![SurrealRequiredCircumstance {
            id: Some(("surreal_requirement", "1").into()),
            required_for: surreal_items.first().unwrap().id.as_ref().unwrap().clone(),
            circumstance_type: CircumstanceType::NotSunday,
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings: vec![],
            surreal_required_circumstances,
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let to_dos = items.filter_just_to_dos();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);
        let saturday =
            DateTime::parse_from_str("1983 Apr 16 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();
        let next_step_nodes = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &saturday,
            false,
        );

        assert_eq!(1, next_step_nodes.len());
    }

    fn undefined_datetime_for_testing() -> Datetime {
        Local::now().naive_utc().and_utc().into()
    }

    #[test]
    fn to_dos_disappear_after_they_are_covered() {
        let surreal_items = vec![
            SurrealItem {
                id: Some(("surreal_item", "1").into()),
                summary: "Covered Item that should not be shown".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
            SurrealItem {
                id: Some(("surreal_item", "2").into()),
                summary: "Covering Item that should be shown".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
        ];
        let surreal_coverings = vec![SurrealCovering {
            id: Some(("surreal_covering", "1").into()),
            smaller: surreal_items[1].id.as_ref().expect("value set").clone(),
            parent: surreal_items[0].id.as_ref().expect("value set").clone(),
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings,
            surreal_required_circumstances: vec![],
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

        let to_dos = items.filter_just_to_dos();
        let wednesday_ignore =
            DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();
        let next_step_nodes = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &wednesday_ignore,
            false,
        );

        assert_eq!(next_step_nodes.len(), 1);
        assert_eq!(next_step_nodes.iter().next().unwrap().to_do, &items[1]);
    }

    #[test]
    fn to_dos_return_to_next_steps_after_they_are_uncovered() {
        let surreal_items = vec![
            SurrealItem {
                id: Some(("surreal_item", "1").into()),
                summary: "Covered Item to show once the covering item is finished".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
            SurrealItem {
                id: Some(("surreal_item", "2").into()),
                summary: "Covering Item that is finished".into(),
                finished: Some(undefined_datetime_for_testing()),
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
        ];
        let surreal_coverings = vec![SurrealCovering {
            id: Some(("surreal_covering", "1").into()),
            smaller: surreal_items[1].id.as_ref().expect("Set above").clone(),
            parent: surreal_items[0].id.as_ref().expect("Set above").clone(),
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings,
            surreal_required_circumstances: vec![],
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

        let to_dos = items.filter_just_to_dos();
        let wednesday_ignore =
            DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();
        let next_step_nodes = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &wednesday_ignore,
            false,
        );

        assert_eq!(next_step_nodes.len(), 1);
        assert_eq!(next_step_nodes.iter().next().unwrap().to_do, &items[0]);
    }

    #[test]
    fn to_dos_covered_until_an_exact_date_time_are_not_shown() {
        let surreal_items = vec![SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Covered item that should not be shown".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: vec![],
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        }];
        let now = Local::now();
        let now_utc: DateTime<Utc> = now.into();
        let in_the_future = now_utc + Duration::from_secs(60);
        let surreal_coverings_until_date_time = vec![SurrealCoveringUntilDatetime {
            id: Some(("surreal_coverings_until_date_time", "1").into()),
            cover_this: surreal_items[0].id.as_ref().unwrap().clone(),
            until: in_the_future.into(),
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings: vec![],
            surreal_required_circumstances: vec![],
            surreal_coverings_until_date_time,
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };

        let items: Vec<Item> = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

        let to_dos = items.filter_just_to_dos();
        let next_steps_nodes = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &now,
            false,
        );

        assert!(next_steps_nodes.is_empty());
    }

    #[test]
    fn to_dos_covered_until_an_exact_date_time_are_shown_again_after_that_time_time() {
        let surreal_items = vec![SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Covered item that should not be shown".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: vec![],
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        }];

        let now = Local::now();
        let now_utc: DateTime<Utc> = now.into();
        let in_the_past = now_utc - Duration::from_secs(60);

        let surreal_coverings_until_date_time = vec![SurrealCoveringUntilDatetime {
            id: Some(("surreal_coverings_until_date_time", "1").into()),
            cover_this: surreal_items[0].id.as_ref().unwrap().clone(),
            until: in_the_past.into(),
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings: vec![],
            surreal_required_circumstances: vec![],
            surreal_coverings_until_date_time,
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items: Vec<Item> = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

        let to_dos = items.filter_just_to_dos();
        let next_steps_nodes = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &now,
            false,
        );

        assert_eq!(1, next_steps_nodes.len());
    }

    #[test]
    fn focus_items_are_not_shown_outside_of_focus_time() {
        let surreal_items = vec![SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Focus item".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: vec![],
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        }];
        let surreal_required_circumstances = vec![SurrealRequiredCircumstance {
            id: Some(("surreal_required_circumstance", "1").into()),
            required_for: surreal_items
                .first()
                .as_ref()
                .expect("set above")
                .id
                .as_ref()
                .expect("set above")
                .clone(),
            circumstance_type: CircumstanceType::DuringFocusTime,
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings: vec![],
            surreal_required_circumstances,
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let to_dos = items.filter_just_to_dos();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);
        let now = Local::now();

        let bullet_list = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &now,
            false,
        );

        assert!(bullet_list.is_empty());
    }

    #[test]
    fn only_focus_items_are_shown_during_focus_time() {
        let surreal_items = vec![SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Focus item".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: vec![],
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        }];
        let surreal_required_circumstances = vec![SurrealRequiredCircumstance {
            id: Some(("surreal_required_circumstance", "1").into()),
            required_for: surreal_items
                .first()
                .as_ref()
                .expect("set above")
                .id
                .as_ref()
                .expect("set above")
                .clone(),
            circumstance_type: CircumstanceType::DuringFocusTime,
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings: vec![],
            surreal_required_circumstances,
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let to_dos = items.filter_just_to_dos();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);
        let now = Local::now();

        let bullet_list = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &now,
            true,
        );

        assert_eq!(1, bullet_list.len());
    }

    #[test]
    fn focus_item_that_covers_is_only_shown_during_focus_time() {
        let surreal_items = vec![
            SurrealItem {
                id: Some(("surreal_item", "1").into()),
                summary: "Focus item".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
            SurrealItem {
                id: Some(("surreal_item", "2").into()),
                summary: "Covered Itemed".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
        ];
        let surreal_required_circumstances = vec![SurrealRequiredCircumstance {
            id: Some(("surreal_required_circumstance", "1").into()),
            required_for: surreal_items
                .first()
                .as_ref()
                .expect("set above")
                .id
                .as_ref()
                .expect("set above")
                .clone(),
            circumstance_type: CircumstanceType::DuringFocusTime,
        }];
        let surreal_coverings = vec![SurrealCovering {
            id: Some(("surreal_coverings", "1").into()),
            smaller: surreal_items[0].id.as_ref().expect("Set above").clone(),
            parent: surreal_items[1].id.as_ref().expect("Set above").clone(),
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings,
            surreal_required_circumstances,
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let to_dos = items.filter_just_to_dos();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);
        let now = Local::now();

        let bullet_list = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &now,
            true,
        );

        assert_eq!(1, bullet_list.len());
    }

    #[test]
    fn focus_item_that_covers_is_hidden_outside_focus_time() {
        let surreal_items = vec![
            SurrealItem {
                id: Some(("surreal_item", "1").into()),
                summary: "Focus item".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
            SurrealItem {
                id: Some(("surreal_item", "2").into()),
                summary: "Covered Itemed".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
        ];
        let surreal_required_circumstances = vec![SurrealRequiredCircumstance {
            id: Some(("surreal_required_circumstance", "1").into()),
            required_for: surreal_items
                .first()
                .as_ref()
                .expect("set above")
                .id
                .as_ref()
                .expect("set above")
                .clone(),
            circumstance_type: CircumstanceType::DuringFocusTime,
        }];
        let surreal_coverings = vec![SurrealCovering {
            id: Some(("surreal_coverings", "1").into()),
            smaller: surreal_items[0].id.as_ref().expect("Set above").clone(),
            parent: surreal_items[1].id.as_ref().expect("Set above").clone(),
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings,
            surreal_required_circumstances,
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let to_dos = items.filter_just_to_dos();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);
        let now = Local::now();

        let bullet_list = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &now,
            false,
        );

        assert!(bullet_list.is_empty());
    }

    #[test]
    fn focus_item_and_non_focus_item_during_focus_time_only_shows_the_focus_item() {
        let surreal_items = vec![
            SurrealItem {
                id: Some(("surreal_item", "1").into()),
                summary: "Focus item".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
            SurrealItem {
                id: Some(("surreal_item", "2").into()),
                summary: "Non-focus item".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
        ];
        let surreal_required_circumstances = vec![SurrealRequiredCircumstance {
            id: Some(("surreal_required_circumstance", "1").into()),
            required_for: surreal_items[0].id.as_ref().expect("set above").clone(),
            circumstance_type: CircumstanceType::DuringFocusTime,
        }];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings: vec![],
            surreal_required_circumstances,
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let to_dos = items.filter_just_to_dos();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);
        let now = Local::now();

        let bullet_list = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &now,
            true,
        );

        assert_eq!(1, bullet_list.len());
        assert_eq!("Focus item", bullet_list[0].get_summary());
    }

    #[test]
    fn to_do_item_with_a_parent_shows_as_having_a_parent_when_next_step_nodes_is_called() {
        let smaller_item = SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Smaller item".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: vec![],
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        };
        let parent_item = SurrealItem {
            id: Some(("surreal_item", "2").into()),
            summary: "Parent item".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: vec![SurrealOrderedSubItem::SubItem {
                surreal_item_id: smaller_item.id.as_ref().expect("set above").clone(),
            }],
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        };
        let surreal_tables = SurrealTables {
            surreal_items: vec![smaller_item.clone(), parent_item.clone()],
            surreal_specific_to_hopes: vec![],
            surreal_coverings: vec![],
            surreal_required_circumstances: vec![],
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items: Vec<Item> = surreal_tables.make_items();
        let active_items = items.filter_active_items();

        let to_dos = items.filter_just_to_dos();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);
        let now_ignore = Local::now();

        let next_steps_nodes = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &now_ignore,
            false,
        );

        let smaller_item_node = next_steps_nodes
            .iter()
            .find(|node| node.to_do.id == smaller_item.id.as_ref().expect("set above"))
            .expect("Unit test failure during test setup, but this might be a product bug");

        assert_eq!(smaller_item_node.larger.len(), 1);
        assert_eq!(
            smaller_item_node
                .larger
                .first()
                .expect("checked in assert above")
                .item
                .id,
            parent_item.id.as_ref().expect("set above")
        );

        let next_step_parents = smaller_item_node.create_next_step_parents();
        assert_eq!(next_step_parents.len(), 1);
        assert_eq!(next_step_parents[0].summary, "Parent item");
    }
}
