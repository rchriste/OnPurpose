use std::time::Duration;

use chrono::{DateTime, Utc};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};

use crate::surrealdb_layer::{
    surreal_item::{
        NotesLocation, SurrealDependency, SurrealFacing, SurrealFrequency, SurrealItem,
        SurrealItemType, SurrealOrderedSubItem, SurrealUrgencyPlan,
    },
    surreal_required_circumstance::SurrealRequiredCircumstance,
};

use super::FindRecordId;

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct Item<'s> {
    id: &'s RecordId,
    required_circumstances: Vec<&'s SurrealRequiredCircumstance>,
    surreal_item: &'s SurrealItem,
    now: &'s DateTime<Utc>,
    now_sql: Datetime,
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

impl<'r> FindRecordId<'r, Item<'r>> for &'r [Item<'r>] {
    fn find_record_id(&self, record_id: &RecordId) -> Option<&'r Item<'r>> {
        self.iter().find(|x| x.get_surreal_record_id() == record_id)
    }
}

pub(crate) trait ItemVecExtensions<'t> {
    type ItemIterator: Iterator<Item = &'t Item<'t>>;

    fn filter_just_persons_or_groups(&'t self) -> Self::ItemIterator;
    fn filter_active_items(&self) -> Vec<&Item>;
}

impl<'s> ItemVecExtensions<'s> for [Item<'s>] {
    type ItemIterator = std::iter::FilterMap<
        std::slice::Iter<'s, Item<'s>>,
        Box<dyn FnMut(&'s Item<'s>) -> Option<&'s Item<'s>>>,
    >;

    fn filter_active_items(&self) -> Vec<&Item> {
        self.iter().filter(|x| !x.is_finished()).collect()
    }

    fn filter_just_persons_or_groups(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.get_item_type() == &SurrealItemType::PersonOrGroup {
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

    fn filter_just_persons_or_groups(&'s self) -> Self::ItemIterator {
        self.iter().filter_map(Box::new(|x| {
            if x.get_item_type() == &SurrealItemType::PersonOrGroup {
                Some(x)
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
        now: &'b DateTime<Utc>,
    ) -> Self {
        let now_sql = (*now).into();
        Self {
            id: surreal_item.id.as_ref().expect("Already in DB"),
            required_circumstances,
            surreal_item,
            now,
            now_sql,
        }
    }

    pub(crate) fn get_item_type(&self) -> &'b SurrealItemType {
        &self.surreal_item.item_type
    }

    pub(crate) fn is_person_or_group(&self) -> bool {
        self.get_item_type() == &SurrealItemType::PersonOrGroup
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.surreal_item.finished.is_some()
    }

    pub(crate) fn is_active(&self) -> bool {
        !self.is_finished()
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

    pub(crate) fn get_type(&self) -> &'b SurrealItemType {
        self.get_item_type()
    }

    pub(crate) fn is_type_undeclared(&self) -> bool {
        self.get_item_type() == &SurrealItemType::Undeclared
    }

    pub(crate) fn is_type_action(&self) -> bool {
        self.get_item_type() == &SurrealItemType::Action
    }

    pub(crate) fn is_type_goal(&self) -> bool {
        matches!(self.get_item_type(), &SurrealItemType::Goal(..))
    }

    pub(crate) fn is_type_motivation(&self) -> bool {
        matches!(self.get_item_type(), &SurrealItemType::Motivation(..))
    }

    pub(crate) fn is_there_notes(&self) -> bool {
        self.surreal_item.notes_location != NotesLocation::None
    }

    pub(crate) fn is_goal(&self) -> bool {
        matches!(self.get_item_type(), &SurrealItemType::Goal(..))
    }

    pub(crate) fn get_surreal_facing(&self) -> &Vec<SurrealFacing> {
        &self.surreal_item.facing
    }

    pub(crate) fn get_created(&self) -> &DateTime<Utc> {
        &self.surreal_item.created
    }

    pub(crate) fn get_surreal_urgency_plan(&self) -> &Option<SurrealUrgencyPlan> {
        &self.surreal_item.urgency_plan
    }

    pub(crate) fn get_surreal_dependencies(&self) -> &Vec<SurrealDependency> {
        &self.surreal_item.dependencies
    }

    pub(crate) fn get_now(&self) -> &DateTime<Utc> {
        self.now
    }

    pub(crate) fn get_now_sql(&self) -> &Datetime {
        &self.now_sql
    }
}

impl Item<'_> {
    pub(crate) fn find_parents<'a>(
        &self,
        other_items: &'a [Item<'a>],
        visited: &[&Item<'_>],
    ) -> Vec<&'a Item<'a>> {
        other_items
            .iter()
            .filter(|other_item| {
                other_item.is_this_a_smaller_item(self) && !visited.contains(other_item)
            })
            .collect()
    }

    pub(crate) fn find_children<'a>(
        &self,
        other_items: &'a [Item<'a>],
        visited: &[&Item<'_>],
    ) -> Vec<&'a Item<'a>> {
        self.surreal_item
            .smaller_items_in_priority_order
            .iter()
            .filter_map(|x| match x {
                SurrealOrderedSubItem::SubItem { surreal_item_id } => other_items
                    .iter()
                    .find(|x| x.id == surreal_item_id && !visited.contains(x)),
            })
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
            })
    }

    pub(crate) fn has_review_frequency(&self) -> bool {
        self.surreal_item.item_review.is_some()
    }

    pub(crate) fn is_a_review_due(&self) -> bool {
        match &self.surreal_item.item_review {
            Some(item_review) => match item_review.review_frequency {
                SurrealFrequency::NoneReviewWithParent => false,
                SurrealFrequency::Range {
                    range_min,
                    range_max: _range_max,
                } => item_review.last_reviewed.as_ref().map_or(true, |x| {
                    let last_reviewed: DateTime<Utc> = x.clone().into();
                    let range_min: Duration = range_min.into();
                    last_reviewed + range_min < *self.now
                }),
                SurrealFrequency::Hourly => {
                    let one_hour = Duration::from_secs(60 * 60);
                    item_review.last_reviewed.as_ref().map_or(true, |x| {
                        let last_reviewed: DateTime<Utc> = x.clone().into();
                        last_reviewed + one_hour < *self.now
                    })
                }
                SurrealFrequency::Daily => {
                    let one_day = Duration::from_secs(60 * 60 * 24);
                    item_review.last_reviewed.as_ref().map_or(true, |x| {
                        let last_reviewed: DateTime<Utc> = x.clone().into();
                        last_reviewed + one_day < *self.now
                    })
                }
                SurrealFrequency::EveryFewDays => {
                    let three_days = Duration::from_secs(60 * 60 * 24 * 3);
                    item_review.last_reviewed.as_ref().map_or(true, |x| {
                        let last_reviewed: DateTime<Utc> = x.clone().into();
                        last_reviewed + three_days < *self.now
                    })
                }
                SurrealFrequency::Weekly => {
                    let one_week = Duration::from_secs(60 * 60 * 24 * 7);
                    item_review.last_reviewed.as_ref().map_or(true, |x| {
                        let last_reviewed: DateTime<Utc> = x.clone().into();
                        last_reviewed + one_week < *self.now
                    })
                }
                SurrealFrequency::BiMonthly => {
                    let two_months = Duration::from_secs(60 * 60 * 24 * 30 / 2);
                    item_review.last_reviewed.as_ref().map_or(true, |x| {
                        let last_reviewed: DateTime<Utc> = x.clone().into();
                        last_reviewed + two_months < *self.now
                    })
                }
                SurrealFrequency::Monthly => {
                    let one_month = Duration::from_secs(60 * 60 * 24 * 30);
                    item_review.last_reviewed.as_ref().map_or(true, |x| {
                        let last_reviewed: DateTime<Utc> = x.clone().into();
                        last_reviewed + one_month < *self.now
                    })
                }
                SurrealFrequency::Quarterly => {
                    let three_months = Duration::from_secs(60 * 60 * 24 * 30 * 3);
                    item_review.last_reviewed.as_ref().map_or(true, |x| {
                        let last_reviewed: DateTime<Utc> = x.clone().into();
                        last_reviewed + three_months < *self.now
                    })
                }
                SurrealFrequency::SemiAnnually => {
                    let six_months = Duration::from_secs(60 * 60 * 24 * 30 * 6);
                    item_review.last_reviewed.as_ref().map_or(true, |x| {
                        let last_reviewed: DateTime<Utc> = x.clone().into();
                        last_reviewed + six_months < *self.now
                    })
                }
                SurrealFrequency::Yearly => {
                    let one_year = Duration::from_secs(60 * 60 * 24 * 365);
                    item_review.last_reviewed.as_ref().map_or(true, |x| {
                        let last_reviewed: DateTime<Utc> = x.clone().into();
                        last_reviewed + one_year < *self.now
                    })
                }
            },
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::surrealdb_layer::{
        surreal_item::SurrealItemBuilder, surreal_tables::SurrealTablesBuilder,
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
                })
        }
    }

    #[test]
    fn to_do_item_with_a_parent_returns_the_parent_when_find_parents_is_called() {
        let smaller_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "1").into()))
            .summary("Smaller item")
            .item_type(SurrealItemType::Action)
            .build()
            .unwrap();
        let parent_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Parent item")
            .finished(None)
            .item_type(SurrealItemType::Action)
            .smaller_items_in_priority_order(vec![SurrealOrderedSubItem::SubItem {
                surreal_item_id: smaller_item.id.as_ref().expect("set above").clone(),
            }])
            .build()
            .unwrap();
        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![smaller_item.clone(), parent_item.clone()])
            .build()
            .unwrap();
        let now = Utc::now();
        let items: Vec<Item> = surreal_tables.make_items(&now);

        let smaller_item = items
            .iter()
            .find(|x| smaller_item.id.as_ref().unwrap() == x.id)
            .unwrap();
        let visited = vec![];
        let find_results = smaller_item.find_parents(&items, &visited);

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
            .item_type(SurrealItemType::Action)
            .build()
            .unwrap();
        let parent_item = SurrealItemBuilder::default()
            .id(Some(("surreal_item", "2").into()))
            .summary("Parent item")
            .item_type(SurrealItemType::Action)
            .smaller_items_in_priority_order(vec![SurrealOrderedSubItem::SubItem {
                surreal_item_id: smaller_item.id.as_ref().expect("set above").clone(),
            }])
            .build()
            .unwrap();
        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(vec![smaller_item.clone(), parent_item.clone()])
            .build()
            .unwrap();
        let now = Utc::now();
        let items: Vec<Item> = surreal_tables.make_items(&now);
        let active_items = items.filter_active_items();

        let under_test_parent_item = items
            .iter()
            .find(|x| parent_item.id.as_ref().unwrap() == x.id)
            .unwrap();

        assert!(under_test_parent_item.has_active_children(&active_items));
    }
}
