use chrono::{DateTime, Datelike, Local};
use itertools::chain;
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};

use crate::surrealdb_layer::{
    surreal_item::{
        ItemType, NotesLocation, Permanence, Responsibility, Staging, SurrealItem,
        SurrealOrderedSubItem,
    },
    surreal_required_circumstance::{CircumstanceType, SurrealRequiredCircumstance},
};

use super::{
    covering::Covering, covering_until_date_time::CoveringUntilDateTime, motivation::Motivation,
    motivation_or_responsive_item::MotivationOrResponsiveItem, responsive_item::ResponsiveItem,
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

pub(crate) trait ItemVecExtensions<'t> {
    type ItemIterator: Iterator<Item = &'t Item<'t>>;

    fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<&'a Item>;
    fn filter_just_to_dos(&'t self) -> Self::ItemIterator;
    fn filter_just_hopes(&'t self) -> Self::ItemIterator;
    fn filter_just_motivations(&self) -> Vec<Motivation<'_>>;
    fn filter_just_persons_or_groups(&'t self) -> Self::ItemIterator;
    fn filter_just_undeclared_items(&'t self) -> Self::ItemIterator;
    fn filter_just_simple_items(&'t self) -> Self::ItemIterator;
    fn filter_just_motivations_or_responsive_items(&self) -> Vec<MotivationOrResponsiveItem<'_>>;
    fn filter_active_items(&self) -> Vec<&Item>; //TODO: I might consider having an ActiveItem type and then have the rest of the Filter methods be just for this activeItem type
}

impl<'s> ItemVecExtensions<'s> for [Item<'s>] {
    type ItemIterator = std::iter::FilterMap<
        std::slice::Iter<'s, Item<'s>>,
        Box<dyn FnMut(&'s Item<'s>) -> Option<&'s Item<'s>>>,
    >;

    fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<&'a Item> {
        self.iter().find(|x| x.id == record_id)
    }

    fn filter_just_to_dos(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x: &'s Item<'s>| {
            if x.item_type == &ItemType::ToDo {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_hopes(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x: &'s Item<'s>| {
            if x.item_type == &ItemType::Hope {
                Some(x)
            } else {
                None
            }
        }))
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

    fn filter_just_persons_or_groups(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.item_type == &ItemType::PersonOrGroup {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_undeclared_items(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x: &'s Item<'s>| {
            if x.item_type == &ItemType::Undeclared {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_simple_items(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.item_type == &ItemType::Simple {
                Some(x)
            } else {
                None
            }
        }))
    }
}

impl<'s> ItemVecExtensions<'s> for [&Item<'s>] {
    type ItemIterator = std::iter::FilterMap<
        std::slice::Iter<'s, &'s Item<'s>>,
        Box<dyn FnMut(&'s &'s Item<'s>) -> Option<&'s Item<'s>>>,
    >;

    fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<&'a Item> {
        self.iter().find(|x| x.id == record_id).copied()
    }

    fn filter_just_to_dos(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x: &&'s Item<'s>| {
            if x.item_type == &ItemType::ToDo {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_hopes(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x: &&'s Item<'s>| {
            if x.item_type == &ItemType::Hope {
                Some(x)
            } else {
                None
            }
        }))
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

    fn filter_just_persons_or_groups(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.item_type == &ItemType::PersonOrGroup {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_undeclared_items(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.item_type == &ItemType::Undeclared {
                Some(*x)
            } else {
                None
            }
        }))
    }

    fn filter_just_simple_items(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.item_type == &ItemType::Simple {
                Some(x)
            } else {
                None
            }
        }))
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
        self.iter().filter(|x| !x.is_finished()).copied().collect()
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

    pub(crate) fn is_person_or_group(&self) -> bool {
        self.item_type == &ItemType::PersonOrGroup
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
        all_items: &[&Item<'_>],
        now: &DateTime<Local>,
    ) -> bool {
        self.has_children(all_items)
            || self.is_covered_by_another_item(coverings)
            || self.is_covered_by_date_time(coverings_until_date_time, now)
    }

    pub(crate) fn covered_by(
        &'b self,
        coverings: &'b [Covering<'b>],
        all_items: &'b [&Item<'b>],
    ) -> impl Iterator<Item = &'b Item<'b>> + 'b {
        chain!(
            coverings.iter().filter_map(move |x| {
                if x.parent == self && !x.smaller.is_finished() {
                    Some(x.smaller)
                } else {
                    None
                }
            }),
            self.surreal_item
                .smaller_items_in_priority_order
                .iter()
                .filter_map(|x| match x {
                    SurrealOrderedSubItem::SubItem { surreal_item_id } => all_items
                        .iter()
                        .copied()
                        .find(|x| x.id == surreal_item_id && !x.is_finished()),
                    SurrealOrderedSubItem::Split { shared_priority: _ } => todo!(),
                })
        )
    }

    pub(crate) fn get_id(&self) -> &'b Thing {
        self.id
    }

    pub(crate) fn get_surreal_item(&self) -> &'b SurrealItem {
        self.surreal_item
    }

    pub(crate) fn get_summary(&self) -> &'b str {
        self.summary
    }

    pub(crate) fn get_finished(&self) -> &'b Option<Datetime> {
        self.finished
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

    pub(crate) fn has_children(&self, all_items: &[&Item<'_>]) -> bool {
        self.surreal_item
            .smaller_items_in_priority_order
            .iter()
            .any(|x| match x {
                SurrealOrderedSubItem::SubItem { surreal_item_id } => all_items
                    .iter()
                    .find(|x| x.id == surreal_item_id)
                    .is_some_and(|x| !x.is_finished()),
                SurrealOrderedSubItem::Split { shared_priority: _ } => todo!(),
            })
    }

    pub(crate) fn is_there_notes(&self) -> bool {
        self.surreal_item.notes_location != NotesLocation::None
    }

    pub(crate) fn get_staging(&self) -> &Staging {
        &self.surreal_item.staging
    }

    pub(crate) fn is_mentally_resident(&self) -> bool {
        self.get_staging() == &Staging::MentallyResident
    }

    pub(crate) fn is_staging_not_set(&self) -> bool {
        self.get_staging() == &Staging::NotSet
    }

    pub(crate) fn get_permanence(&self) -> &Permanence {
        &self.surreal_item.permanence
    }

    pub(crate) fn is_project(&self) -> bool {
        self.get_permanence() == &Permanence::Project
    }

    pub(crate) fn is_permanence_not_set(&self) -> bool {
        self.get_permanence() == &Permanence::NotSet
    }

    pub(crate) fn is_maintenance(&self) -> bool {
        self.get_permanence() == &Permanence::Maintenance
    }

    pub(crate) fn is_goal(&self) -> bool {
        self.item_type == &ItemType::Hope
    }

    pub(crate) fn is_covered_by_a_hope(
        &self,
        coverings: &[Covering<'_>],
        all_items: &[&Item<'_>],
    ) -> bool {
        self.covered_by(coverings, all_items)
            .any(|x| x.is_type_hope() && !x.is_finished())
    }
}

impl Item<'_> {
    pub(crate) fn find_parents<'a>(
        &self,
        linkage: &'a [Covering<'a>],
        other_items: &'a [&'a Item<'a>],
        visited: &[&Item<'_>],
    ) -> Vec<&'a Item<'a>> {
        //TODO: Update the below code to use the chain! macros rather than this manual extend thing
        let mut result = linkage
            .iter()
            .filter_map(|x| {
                if x.smaller == self && !visited.contains(&x.parent) {
                    Some(x.parent)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        result.extend(other_items.iter().filter_map(|other_item| {
            if other_item.is_this_a_smaller_item(self) && !visited.contains(other_item) {
                Some(*other_item)
            } else {
                None
            }
        }));
        result
    }

    pub(crate) fn find_children<'a>(
        &self,
        linkage: &'a [Covering<'a>],
        other_items: &'a [&'a Item<'a>],
        visited: &[&Item<'_>],
    ) -> Vec<&'a Item<'a>> {
        //TODO: Update the below code to use the chain macro rather than this manual extend thing
        let mut result = linkage
            .iter()
            .filter_map(|x| {
                if x.parent == self && !visited.contains(&x.smaller) {
                    Some(x.smaller)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        result.extend(other_items.iter().filter_map(|other_item| {
            if self.is_this_a_smaller_item(other_item) && !visited.contains(other_item) {
                Some(*other_item)
            } else {
                None
            }
        }));
        result
    }

    pub(crate) fn is_this_a_smaller_item(&self, other_item: &Item) -> bool {
        self.surreal_item
            .smaller_items_in_priority_order
            .iter()
            .any(|x| match x {
                SurrealOrderedSubItem::SubItem { surreal_item_id } => {
                    other_item
                        .surreal_item
                        .id
                        .as_ref()
                        .expect("Should always be in DB")
                        == surreal_item_id
                }
                SurrealOrderedSubItem::Split { shared_priority: _ } => {
                    todo!("Implement this now that this variant is more than a placeholder")
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::surrealdb_layer::{
        surreal_item::{SurrealItemBuilder, SurrealOrderedSubItem},
        surreal_required_circumstance::CircumstanceType,
        surreal_tables::SurrealTablesBuilder,
    };

    use super::*;

    #[test]
    fn is_circumstances_met_returns_false_if_circumstance_type_is_not_sunday_and_it_is_sunday() {
        let surreal_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("Circumstance type is not Sunday")
            .item_type(ItemType::ToDo)
            .build()
            .unwrap();

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
        let surreal_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("Circumstance type is not Sunday")
            .item_type(ItemType::ToDo)
            .build()
            .unwrap();

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
        let surreal_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("Circumstance type is not Sunday")
            .item_type(ItemType::ToDo)
            .build()
            .unwrap();

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
        let surreal_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("Circumstance type is not Sunday")
            .item_type(ItemType::ToDo)
            .build()
            .unwrap();

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
        let smaller_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("Smaller item")
            .item_type(ItemType::ToDo)
            .build()
            .unwrap();
        let parent_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Parent item")
            .finished(None)
            .item_type(ItemType::ToDo)
            .smaller_items_in_priority_order(vec![SurrealOrderedSubItem::SubItem {
                surreal_item_id: smaller_item.id.as_ref().expect("set above").clone(),
            }])
            .build()
            .unwrap();
        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![smaller_item.clone(), parent_item.clone()])
            .build()
            .unwrap();
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

    #[test]
    fn when_active_smaller_items_in_priority_order_exist_has_children_returns_true() {
        let smaller_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("Smaller item")
            .item_type(ItemType::ToDo)
            .build()
            .unwrap();
        let parent_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Parent item")
            .item_type(ItemType::ToDo)
            .smaller_items_in_priority_order(vec![SurrealOrderedSubItem::SubItem {
                surreal_item_id: smaller_item.id.as_ref().expect("set above").clone(),
            }])
            .build()
            .unwrap();
        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![smaller_item.clone(), parent_item.clone()])
            .build()
            .unwrap();
        let items: Vec<Item> = surreal_tables.make_items();
        let active_items = items.filter_active_items();

        let under_test_parent_item = items
            .iter()
            .find(|x| parent_item.id.as_ref().unwrap() == x.id)
            .unwrap();

        assert!(under_test_parent_item.has_children(&active_items));
    }

    #[test]
    fn when_active_smaller_items_in_priority_order_exist_covered_by_returns_the_items() {
        let smaller_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("Smaller item")
            .item_type(ItemType::ToDo)
            .build()
            .unwrap();
        let parent_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Parent item")
            .item_type(ItemType::ToDo)
            .smaller_items_in_priority_order(vec![SurrealOrderedSubItem::SubItem {
                surreal_item_id: smaller_item.id.as_ref().expect("set above").clone(),
            }])
            .build()
            .unwrap();
        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![smaller_item.clone(), parent_item.clone()])
            .build()
            .unwrap();
        let items: Vec<Item> = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let coverings = surreal_tables.make_coverings(&items);

        let under_test_parent_item = items
            .iter()
            .find(|x| parent_item.id.as_ref().unwrap() == x.id)
            .unwrap();

        let covered_by = under_test_parent_item
            .covered_by(&coverings, &active_items)
            .collect::<Vec<_>>();

        assert_eq!(covered_by.len(), 1);
        assert_eq!(
            covered_by.first().expect("checked in assert above").id,
            smaller_item.id.as_ref().expect("set above")
        );
    }
}
