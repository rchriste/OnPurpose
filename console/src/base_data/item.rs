use chrono::{DateTime, Local, Utc};
use itertools::chain;
use surrealdb::{opt::RecordId, sql::Thing};

use crate::surrealdb_layer::{
    surreal_item::{
        Facing, ItemType, NotesLocation, Permanence, Responsibility, Staging, SurrealItem,
        SurrealOrderedSubItem,
    },
    surreal_required_circumstance::SurrealRequiredCircumstance,
};

use super::{covering::Covering, covering_until_date_time::CoveringUntilDateTime};

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct Item<'s> {
    id: &'s RecordId,
    required_circumstances: Vec<&'s SurrealRequiredCircumstance>,
    surreal_item: &'s SurrealItem,
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

impl From<Item<'_>> for RecordId {
    fn from(value: Item<'_>) -> Self {
        value
            .surreal_item
            .id
            .as_ref()
            .expect("Already in DB")
            .clone()
    }
}

pub(crate) trait ItemVecExtensions<'t> {
    type ItemIterator: Iterator<Item = &'t Item<'t>>;

    fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<&'a Item>;
    fn filter_just_actions(&'t self) -> Self::ItemIterator;
    fn filter_just_goals(&'t self) -> Self::ItemIterator;
    fn filter_just_motivations(&'t self) -> Self::ItemIterator;
    fn filter_just_persons_or_groups(&'t self) -> Self::ItemIterator;
    fn filter_just_undeclared_items(&'t self) -> Self::ItemIterator;
    fn filter_active_items(&self) -> Vec<&Item>;
}

impl<'s> ItemVecExtensions<'s> for [Item<'s>] {
    type ItemIterator = std::iter::FilterMap<
        std::slice::Iter<'s, Item<'s>>,
        Box<dyn FnMut(&'s Item<'s>) -> Option<&'s Item<'s>>>,
    >;

    fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<&'a Item> {
        self.iter().find(|x| x.id == record_id)
    }

    //TODO: This should probably be renamed to internal actions or steps
    fn filter_just_actions(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x: &'s Item<'s>| {
            if x.get_item_type() == &ItemType::Action {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_goals(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x: &'s Item<'s>| {
            if matches!(x.get_item_type(), &ItemType::Goal(..)) {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_motivations(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.get_item_type() == &ItemType::Motivation {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_active_items(&self) -> Vec<&Item> {
        self.iter().filter(|x| !x.is_finished()).collect()
    }

    fn filter_just_persons_or_groups(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.get_item_type() == &ItemType::PersonOrGroup {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_undeclared_items(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x: &'s Item<'s>| {
            if x.get_item_type() == &ItemType::Undeclared {
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

    fn filter_just_actions(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x: &&'s Item<'s>| {
            if x.get_item_type() == &ItemType::Action {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_goals(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x: &&'s Item<'s>| {
            if matches!(x.get_item_type(), &ItemType::Goal(..)) {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_motivations(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.get_item_type() == &ItemType::Motivation {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_persons_or_groups(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.get_item_type() == &ItemType::PersonOrGroup {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn filter_just_undeclared_items(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.get_item_type() == &ItemType::Undeclared {
                Some(*x)
            } else {
                None
            }
        }))
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
            required_circumstances,
            surreal_item,
        }
    }

    pub(crate) fn get_item_type(&self) -> &'b ItemType {
        &self.surreal_item.item_type
    }

    pub(crate) fn is_person_or_group(&self) -> bool {
        self.get_item_type() == &ItemType::PersonOrGroup
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.surreal_item.finished.is_some()
    }

    pub(crate) fn when_finished(&self) -> Option<DateTime<Utc>> {
        match self.surreal_item.finished {
            Some(ref finished) => {
                let finished = finished.clone().into();
                Some(finished)
            }
            None => None,
        }
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

    pub(crate) fn get_covered_by_date_time_filter_out_the_past<'a>(
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

    pub(crate) fn get_covered_by_date_time<'a>(
        &self,
        coverings_until_date_time: &'a [&'a CoveringUntilDateTime<'a>],
    ) -> Vec<&'a DateTime<Local>> {
        let covered_by_date_time = coverings_until_date_time
            .iter()
            .filter(|x| self == x.cover_this);
        covered_by_date_time.map(|x| &x.until).collect()
    }

    pub(crate) fn covered_by(
        &'b self,
        coverings: &'b [Covering<'b>],
        all_items: &'b [Item<'b>],
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
                        .find(|x| x.id == surreal_item_id && !x.is_finished()),
                    SurrealOrderedSubItem::Split { shared_priority: _ } => todo!(),
                })
        )
    }

    pub(crate) fn get_id(&self) -> &'b Thing {
        self.id
    }

    pub(crate) fn get_surreal_record_id(&self) -> &'b RecordId {
        self.id
    }

    pub(crate) fn get_summary(&self) -> &'b str {
        &self.surreal_item.summary
    }

    pub(crate) fn get_type(&self) -> &'b ItemType {
        self.get_item_type()
    }

    pub(crate) fn is_type_undeclared(&self) -> bool {
        self.get_item_type() == &ItemType::Undeclared
    }

    pub(crate) fn is_type_action(&self) -> bool {
        self.get_item_type() == &ItemType::Action
    }

    pub(crate) fn is_type_goal(&self) -> bool {
        matches!(self.get_item_type(), &ItemType::Goal(..))
    }

    pub(crate) fn is_type_motivation(&self) -> bool {
        self.get_item_type() == &ItemType::Motivation
    }

    pub(crate) fn has_children(&self) -> bool {
        !self.surreal_item.smaller_items_in_priority_order.is_empty()
    }

    pub(crate) fn is_there_notes(&self) -> bool {
        self.surreal_item.notes_location != NotesLocation::None
    }

    pub(crate) fn get_staging(&self) -> &Staging {
        &self.surreal_item.staging
    }

    pub(crate) fn is_mentally_resident(&self) -> bool {
        matches!(self.get_staging(), Staging::MentallyResident { .. })
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
        matches!(self.get_item_type(), &ItemType::Goal(..))
    }

    pub(crate) fn is_covered_by_a_goal(
        &self,
        coverings: &[Covering<'_>],
        all_items: &[Item<'_>],
    ) -> bool {
        self.covered_by(coverings, all_items)
            .any(|x| x.is_type_goal() && !x.is_finished())
    }

    pub(crate) fn get_thing(&self) -> &Thing {
        self.surreal_item.id.as_ref().expect("Already in DB")
    }

    pub(crate) fn get_facing(&self) -> &Vec<Facing> {
        &self.surreal_item.facing
    }

    pub(crate) fn get_created(&self) -> &DateTime<Utc> {
        &self.surreal_item.created
    }
}

impl Item<'_> {
    pub(crate) fn find_parents<'a>(
        &self,
        linkage: &'a [Covering<'a>],
        other_items: &'a [Item<'a>],
        visited: &[&Item<'_>],
    ) -> Vec<&'a Item<'a>> {
        chain!(
            linkage.iter().filter_map(|x| {
                if x.smaller == self && !visited.contains(&x.parent) {
                    Some(x.parent)
                } else {
                    None
                }
            }),
            other_items.iter().filter(|other_item| {
                other_item.is_this_a_smaller_item(self) && !visited.contains(other_item)
            })
        )
        .collect()
    }

    pub(crate) fn find_children<'a>(
        &self,
        linkage: &'a [Covering<'a>],
        other_items: &'a [Item<'a>],
        visited: &[&Item<'_>],
    ) -> Vec<&'a Item<'a>> {
        chain!(
            self.surreal_item
                .smaller_items_in_priority_order
                .iter()
                .filter_map(|x| match x {
                    SurrealOrderedSubItem::SubItem { surreal_item_id } => other_items
                        .iter()
                        .find(|x| x.id == surreal_item_id && !visited.contains(x)),
                    SurrealOrderedSubItem::Split { shared_priority: _ } => todo!(),
                }),
            linkage.iter().filter_map(|x| {
                if x.parent == self && !visited.contains(&x.smaller) {
                    Some(x.smaller)
                } else {
                    None
                }
            })
        )
        .collect()
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

    pub(crate) fn is_responsibility_reactive(&self) -> bool {
        self.get_responsibility() == &Responsibility::ReactiveBeAvailableToAct
    }

    pub(crate) fn get_responsibility(&self) -> &Responsibility {
        &self.surreal_item.responsibility
    }
}

#[cfg(test)]
mod tests {
    use crate::surrealdb_layer::{
        surreal_item::{SurrealItemBuilder, SurrealOrderedSubItem},
        surreal_tables::SurrealTablesBuilder,
    };

    use super::*;

    impl Item<'_> {
        pub(crate) fn has_active_children(&self, all_items: &[&Item<'_>]) -> bool {
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
    }

    #[test]
    fn to_do_item_with_a_parent_returns_the_parent_when_find_parents_is_called() {
        let smaller_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("Smaller item")
            .item_type(ItemType::Action)
            .build()
            .unwrap();
        let parent_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Parent item")
            .finished(None)
            .item_type(ItemType::Action)
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
        let coverings = surreal_tables.make_coverings(&active_items);

        let smaller_item = items
            .iter()
            .find(|x| smaller_item.id.as_ref().unwrap() == x.id)
            .unwrap();
        let visited = vec![];
        let find_results = smaller_item.find_parents(&coverings, &items, &visited);

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
            .item_type(ItemType::Action)
            .build()
            .unwrap();
        let parent_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Parent item")
            .item_type(ItemType::Action)
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

        assert!(under_test_parent_item.has_active_children(&active_items));
    }

    #[test]
    fn when_active_smaller_items_in_priority_order_exist_covered_by_returns_the_items() {
        let smaller_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("Smaller item")
            .item_type(ItemType::Action)
            .build()
            .unwrap();
        let parent_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Parent item")
            .item_type(ItemType::Action)
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
        let coverings = surreal_tables.make_coverings(&active_items);

        let under_test_parent_item = items
            .iter()
            .find(|x| parent_item.id.as_ref().unwrap() == x.id)
            .unwrap();

        let covered_by = under_test_parent_item
            .covered_by(&coverings, &items)
            .collect::<Vec<_>>();

        assert_eq!(covered_by.len(), 1);
        assert_eq!(
            covered_by.first().expect("checked in assert above").id,
            smaller_item.id.as_ref().expect("set above")
        );
    }
}
