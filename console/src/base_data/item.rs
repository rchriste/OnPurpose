use chrono::{DateTime, Datelike, Local};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};

use crate::surrealdb_layer::{
    surreal_item::{NotesLocation, Responsibility, SurrealItem},
    surreal_required_circumstance::{CircumstanceType, SurrealRequiredCircumstance},
    surreal_specific_to_hope::{SurrealSpecificToHope, SurrealSpecificToHopes},
};

use super::{
    hope::Hope, motivation::Motivation, motivation_or_responsive_item::MotivationOrResponsiveItem,
    person_or_group::PersonOrGroup, responsive_item::ResponsiveItem, simple::Simple, to_do::ToDo,
    undeclared::Undeclared, Covering, CoveringUntilDateTime, ItemType,
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct Item<'s> {
    pub(crate) id: &'s Thing,
    pub(crate) summary: &'s String,
    pub(crate) finished: &'s Option<Datetime>,
    pub(crate) responsibility: &'s Responsibility,
    pub(crate) item_type: &'s ItemType,
    pub(crate) required_circumstances: Vec<&'s SurrealRequiredCircumstance>,
    pub(crate) surreal_item: &'s SurrealItem,
}

impl<'a> From<&'a Item<'a>> for &'a SurrealItem {
    fn from(value: &Item<'a>) -> Self {
        value.surreal_item
    }
}

impl From<Item<'_>> for SurrealItem {
    fn from(value: Item<'_>) -> Self {
        value.surreal_item.clone()
    }
}

pub(crate) trait ItemVecExtensions {
    fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<&'a Item>;
    fn filter_just_to_dos(&self) -> Vec<ToDo<'_>>;
    fn filter_just_hopes<'a>(
        &'a self,
        surreal_specific_to_hopes: &'a [SurrealSpecificToHope],
    ) -> Vec<Hope<'a>>;
    fn filter_just_motivations(&self) -> Vec<Motivation<'_>>;
    fn filter_just_persons_or_groups(&self) -> Vec<PersonOrGroup<'_>>;
    fn filter_just_undeclared_items(&self) -> Vec<Undeclared<'_>>;
    fn filter_just_simple_items(&self) -> Vec<Simple<'_>>;
    fn filter_just_motivations_or_responsive_items(&self) -> Vec<MotivationOrResponsiveItem<'_>>;
    fn filter_active_items(&self) -> Vec<&Item>; //TODO: I might consider having an ActiveItem type and then have the rest of the Filter methods be just for this activeItem type
}

impl<'s> ItemVecExtensions for [Item<'s>] {
    fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<&'a Item> {
        self.iter().find(|x| x.id == record_id)
    }

    fn filter_just_to_dos(&self) -> Vec<ToDo<'_>> {
        self.iter()
            .filter_map(|x| {
                if x.item_type == &ItemType::ToDo {
                    Some(ToDo::new(x))
                } else {
                    None
                }
            })
            .collect()
    }

    fn filter_just_hopes<'a>(
        &'a self,
        surreal_specific_to_hopes: &'a [SurrealSpecificToHope],
    ) -> Vec<Hope<'a>> {
        //Initially I had this with a iter().filter_map() but then I had some issue with the borrow checker and surreal_specific_to_hopes that I didn't understand so I refactored it to this code to work around that issue
        let mut just_hopes = Vec::default();
        for x in self.iter() {
            if x.item_type == &ItemType::Hope {
                let hope_specific: Option<&SurrealSpecificToHope> =
                    surreal_specific_to_hopes.get_by_id(x.id);
                let hope_specific = hope_specific.unwrap().clone(); //TODO: Figure out how to use borrow rather than clone()
                let hope = Hope::new(x, hope_specific);
                just_hopes.push(hope);
            }
        }
        just_hopes
    }

    fn filter_just_motivations(&self) -> Vec<Motivation<'_>> {
        self.iter()
            .filter_map(|x| {
                if x.item_type == &ItemType::Motivation {
                    Some(Motivation::new(x))
                } else {
                    None
                }
            })
            .collect()
    }

    fn filter_just_motivations_or_responsive_items(&self) -> Vec<MotivationOrResponsiveItem<'_>> {
        self.iter()
            .filter_map(|x| {
                if x.item_type == &ItemType::Motivation {
                    Some(MotivationOrResponsiveItem::Motivation(Motivation::new(x)))
                } else if x.responsibility == &Responsibility::ReactiveBeAvailableToAct {
                    Some(MotivationOrResponsiveItem::ResponsiveItem(
                        ResponsiveItem::new(x),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    fn filter_active_items(&self) -> Vec<&Item> {
        self.iter().filter(|x| !x.is_finished()).collect()
    }

    fn filter_just_persons_or_groups(&self) -> Vec<PersonOrGroup<'_>> {
        self.iter()
            .filter_map(|x| {
                if x.item_type == &ItemType::PersonOrGroup {
                    Some(PersonOrGroup::new(x))
                } else {
                    None
                }
            })
            .collect()
    }

    fn filter_just_undeclared_items(&self) -> Vec<Undeclared<'_>> {
        self.iter()
            .filter_map(|x| {
                if x.item_type == &ItemType::Undeclared {
                    Some(Undeclared::new(x))
                } else {
                    None
                }
            })
            .collect()
    }

    fn filter_just_simple_items(&self) -> Vec<Simple<'_>> {
        self.iter()
            .filter_map(|x| {
                if x.item_type == &ItemType::Simple {
                    Some(Simple::new(x))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl<'b> Item<'b> {
    pub(crate) fn new(
        surreal_item: &'b SurrealItem,
        required_circumstances: Vec<&'b SurrealRequiredCircumstance>,
    ) -> Self {
        Self {
            id: surreal_item.id.as_ref().expect("Already in DB"),
            summary: &surreal_item.summary,
            finished: &surreal_item.finished,
            item_type: &surreal_item.item_type,
            responsibility: &surreal_item.responsibility,
            required_circumstances,
            surreal_item,
        }
    }

    pub(crate) fn is_circumstances_met(
        &self,
        date: &DateTime<Local>,
        are_we_in_focus_time: bool,
    ) -> bool {
        self.is_circumstances_met_sunday(date)
            && self.is_circumstances_met_focus_time(are_we_in_focus_time)
    }

    pub(crate) fn is_circumstances_met_sunday(&self, date: &DateTime<Local>) -> bool {
        !self
            .required_circumstances
            .iter()
            .any(|x| match x.circumstance_type {
                CircumstanceType::NotSunday => date.weekday().num_days_from_sunday() == 0,
                _ => false,
            })
    }

    pub(crate) fn is_circumstances_met_focus_time(&self, are_we_in_focus_time: bool) -> bool {
        let should_this_be_done_during_focus_time = self
            .required_circumstances
            .iter()
            .any(|x| matches!(x.circumstance_type, CircumstanceType::DuringFocusTime));

        should_this_be_done_during_focus_time == are_we_in_focus_time
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.finished.is_some()
    }

    pub(crate) fn is_covered_by_another_item(&self, coverings: &[Covering<'_>]) -> bool {
        let mut covered_by = coverings.iter().filter(|x| self == x.parent);
        //Now see if the items that are covering are finished or active
        covered_by.any(|x| !x.smaller.is_finished())
    }

    pub(crate) fn get_covered_by_another_item(&self, coverings: &[Covering<'b>]) -> Vec<&Self> {
        let covered_by = coverings.iter().filter(|x| self == x.parent);
        //Now see if the items that are covering are finished or active
        covered_by
            .filter_map(|x| {
                if !x.smaller.is_finished() {
                    Some(x.smaller)
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn is_covering_another_item(&self, coverings: &[Covering<'_>]) -> bool {
        let mut cover_others = coverings.iter().filter(|x| self == x.smaller);
        //Now see if the items that are covering are finished or active
        cover_others.any(|x| !x.parent.is_finished())
    }

    pub(crate) fn get_covering_another_item(&self, coverings: &[Covering<'b>]) -> Vec<&Self> {
        let cover_others = coverings.iter().filter(|x| self == x.smaller);
        //Now see if the items that are covering are finished or active
        cover_others
            .filter_map(|x| {
                if !x.parent.is_finished() {
                    Some(x.parent)
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn is_covered_by_date_time(
        &self,
        coverings_until_date_time: &[CoveringUntilDateTime<'_>],
        now: &DateTime<Local>,
    ) -> bool {
        let mut covered_by_date_time = coverings_until_date_time
            .iter()
            .filter(|x| self == x.cover_this);
        covered_by_date_time.any(|x| now < &x.until)
    }

    pub(crate) fn get_covered_by_date_time<'a>(
        &self,
        coverings_until_date_time: &'a [CoveringUntilDateTime<'a>],
        now: &DateTime<Local>,
    ) -> Vec<&'a DateTime<Local>> {
        let covered_by_date_time = coverings_until_date_time
            .iter()
            .filter(|x| self == x.cover_this);
        covered_by_date_time
            .filter_map(|x| if now < &x.until { Some(&x.until) } else { None })
            .collect()
    }

    pub(crate) fn is_covered(
        &self,
        coverings: &[Covering<'_>],
        coverings_until_date_time: &[CoveringUntilDateTime<'_>],
        now: &DateTime<Local>,
    ) -> bool {
        self.is_covered_by_another_item(coverings)
            || self.is_covered_by_date_time(coverings_until_date_time, now)
    }

    pub(crate) fn covered_by<'a>(&self, coverings: &[Covering<'a>]) -> Vec<&'a Item<'a>> {
        coverings
            .iter()
            .filter_map(|x| {
                if x.parent == self && !x.smaller.is_finished() {
                    Some(x.smaller)
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn who_am_i_covering<'a>(&self, coverings: &[Covering<'a>]) -> Vec<&'a Item<'a>> {
        coverings
            .iter()
            .filter_map(|x| {
                if x.smaller == self {
                    Some(x.parent)
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn get_surreal_item(&self) -> &'b SurrealItem {
        self.surreal_item
    }

    pub(crate) fn get_summary(&self) -> &'b str {
        self.summary
    }

    pub(crate) fn is_type_undeclared(&self) -> bool {
        self.item_type == &ItemType::Undeclared
    }

    pub(crate) fn is_type_simple(&self) -> bool {
        self.item_type == &ItemType::Simple
    }

    pub(crate) fn is_type_action(&self) -> bool {
        self.item_type == &ItemType::ToDo
    }

    pub(crate) fn is_type_hope(&self) -> bool {
        self.item_type == &ItemType::Hope
    }

    pub(crate) fn is_type_motivation(&self) -> bool {
        self.item_type == &ItemType::Motivation
    }

    pub(crate) fn is_circumstance_focus_time(&self) -> bool {
        self.required_circumstances
            .iter()
            .any(|x| matches!(x.circumstance_type, CircumstanceType::DuringFocusTime))
    }

    pub(crate) fn get_estimated_focus_periods(&self) -> Option<u32> {
        todo!("I need to ensure that we are storing this data with an Item and then I can implement this method")
    }

    pub(crate) fn has_children(&self) -> bool {
        !self.surreal_item.smaller_items_in_priority_order.is_empty()
    }

    pub(crate) fn is_there_notes(&self) -> bool {
        self.surreal_item.notes_location != NotesLocation::None
    }
}

#[cfg(test)]
mod tests {
    use crate::surrealdb_layer::{
        surreal_item::SurrealOrderedSubItem, surreal_required_circumstance::CircumstanceType,
        SurrealTables,
    };

    use super::*;

    #[test]
    fn is_circumstances_met_returns_false_if_circumstance_type_is_not_sunday_and_it_is_sunday() {
        let surreal_item = SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Circumstance type is not Sunday".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: Vec::default(),
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        };

        let required_circumstance = SurrealRequiredCircumstance {
            id: Some(("surreal_required_circumstance", "1").into()),
            required_for: surreal_item.id.as_ref().expect("set above").clone(),
            circumstance_type: CircumstanceType::NotSunday,
        };

        let item = Item::new(&surreal_item, vec![&required_circumstance]);

        let sunday =
            DateTime::parse_from_str("1983 Apr 17 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();

        assert!(!item.is_circumstances_met(&sunday, false));
    }

    #[test]
    fn is_circumstances_met_returns_true_if_circumstance_type_is_not_sunday_and_it_is_not_sunday() {
        let surreal_item = SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Circumstance type is not Sunday".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: Vec::default(),
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        };

        let required_circumstance = SurrealRequiredCircumstance {
            id: Some(("surreal_required_circumstance", "1").into()),
            required_for: surreal_item.id.as_ref().expect("set above").clone(),
            circumstance_type: CircumstanceType::NotSunday,
        };

        let item = Item::new(&surreal_item, vec![&required_circumstance]);

        let wednesday =
            DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();

        assert!(item.is_circumstances_met(&wednesday, false));
    }

    #[test]
    fn is_circumstances_met_returns_false_if_focus_time_is_not_active_and_circumstance_type_is_during_focus_time(
    ) {
        let surreal_item = SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Circumstance type is not Sunday".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: Vec::default(),
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        };

        let required_circumstance = SurrealRequiredCircumstance {
            id: Some(("surreal_required_circumstance", "1").into()),
            required_for: surreal_item.id.as_ref().expect("set above").clone(),
            circumstance_type: CircumstanceType::DuringFocusTime,
        };

        let item = Item::new(&surreal_item, vec![&required_circumstance]);

        let wednesday_ignore =
            DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();

        assert!(item.is_circumstances_met(&wednesday_ignore, true));
    }

    #[test]
    fn is_circumstances_met_returns_true_if_focus_time_is_active_and_circumstance_type_is_during_focus_time(
    ) {
        let surreal_item = SurrealItem {
            id: Some(("surreal_item", "1").into()),
            summary: "Circumstance type is not Sunday".into(),
            finished: None,
            item_type: ItemType::ToDo,
            smaller_items_in_priority_order: Vec::default(),
            responsibility: Responsibility::default(),
            notes_location: Default::default(),
        };

        let required_circumstance = SurrealRequiredCircumstance {
            id: Some(("surreal_required_circumstance", "1").into()),
            required_for: surreal_item.id.as_ref().expect("set above").clone(),
            circumstance_type: CircumstanceType::DuringFocusTime,
        };

        let item = Item::new(&surreal_item, vec![&required_circumstance]);

        let wednesday_ignore =
            DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();

        assert!(!item.is_circumstances_met(&wednesday_ignore, false));
    }

    #[test]
    fn to_do_item_with_a_parent_returns_the_parent_when_find_parents_is_called() {
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
        let coverings = surreal_tables.make_coverings(&items);

        let smaller_item = items
            .iter()
            .find(|x| smaller_item.id.as_ref().unwrap() == x.id)
            .unwrap();
        let visited = vec![];
        let find_results = smaller_item.find_parents(&coverings, &active_items, &visited);

        assert_eq!(find_results.len(), 1);
        assert_eq!(
            find_results.first().expect("checked in assert above").id,
            parent_item.id.as_ref().expect("set above")
        );
    }
}
