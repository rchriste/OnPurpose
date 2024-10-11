use std::time::Duration;

use ahash::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use surrealdb::{
    opt::RecordId,
    sql::{self, Datetime},
};

use crate::data_storage::surrealdb_layer::surreal_item::{
    SurrealDependency, SurrealFrequency, SurrealItem, SurrealItemType, SurrealMotivationKind,
    SurrealOrderedSubItem, SurrealReviewGuidance, SurrealUrgencyPlan,
};

#[derive(Eq, Clone, Debug)]
pub(crate) struct Item<'s> {
    id: &'s RecordId,
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

pub(crate) trait ItemVecExtensions<'t> {
    type ItemIterator: Iterator<Item = &'t Item<'t>>;

    fn filter_active_items(&self) -> Vec<&Item>;
}

impl<'s> ItemVecExtensions<'s> for HashMap<&RecordId, Item<'s>> {
    type ItemIterator = std::iter::FilterMap<
        std::slice::Iter<'s, Item<'s>>,
        Box<dyn FnMut(&'s Item<'s>) -> Option<&'s Item<'s>>>,
    >;

    fn filter_active_items(&self) -> Vec<&Item> {
        self.iter()
            .filter(|(_, x)| !x.is_finished())
            .map(|(_, v)| v)
            .collect()
    }
}

impl<'s> ItemVecExtensions<'s> for HashMap<&RecordId, &Item<'s>> {
    type ItemIterator = std::iter::FilterMap<
        std::slice::Iter<'s, &'s Item<'s>>,
        Box<dyn FnMut(&'s &'s Item<'s>) -> Option<&'s Item<'s>>>,
    >;

    fn filter_active_items(&self) -> Vec<&Item> {
        self.iter()
            .filter(|(_, x)| !x.is_finished())
            .map(|(_, v)| *v)
            .collect()
    }
}

impl PartialEq for Item<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'b> Item<'b> {
    pub(crate) fn new(surreal_item: &'b SurrealItem, now: &'b DateTime<Utc>) -> Self {
        let now_sql = (*now).into();
        Self {
            id: surreal_item.id.as_ref().expect("Already in DB"),
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

    pub(crate) fn get_finished_at(&self) -> &Option<sql::Datetime> {
        &self.surreal_item.finished
    }

    pub(crate) fn is_active(&self) -> bool {
        !self.is_finished()
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

    pub(crate) fn is_type_goal(&self) -> bool {
        matches!(self.get_item_type(), &SurrealItemType::Goal(..))
    }

    pub(crate) fn is_type_motivation(&self) -> bool {
        matches!(self.get_item_type(), &SurrealItemType::Motivation(..))
    }

    pub(crate) fn is_type_motivation_kind_not_set(&self) -> bool {
        matches!(
            self.get_item_type(),
            &SurrealItemType::Motivation(SurrealMotivationKind::NotSet)
        )
    }

    pub(crate) fn is_goal(&self) -> bool {
        matches!(self.get_item_type(), &SurrealItemType::Goal(..))
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
        other_items: &'a HashMap<&'a RecordId, Item<'a>>,
        visited: &HashSet<&RecordId>,
    ) -> Vec<&'a Item<'a>> {
        other_items
            .iter()
            .filter(|(_, other_item)| {
                other_item.is_this_a_smaller_item(self)
                    && !visited.contains(other_item.get_surreal_record_id())
            })
            .map(|(_, x)| x)
            .collect()
    }

    pub(crate) fn find_children<'a>(
        &self,
        other_items: &'a HashMap<&'a RecordId, Item<'a>>,
        visited: &HashSet<&RecordId>,
    ) -> Vec<&'a Item<'a>> {
        self.surreal_item
            .smaller_items_in_priority_order
            .iter()
            .filter_map(|x| match x {
                SurrealOrderedSubItem::SubItem { surreal_item_id } => {
                    if !visited.contains(surreal_item_id) {
                        other_items.get(surreal_item_id)
                    } else {
                        None
                    }
                }
            })
            .collect()
    }

    pub(crate) fn is_this_a_smaller_item(&self, other_item: &Item) -> bool {
        self.surreal_item
            .smaller_items_in_priority_order
            .iter()
            .any(|x| match x {
                SurrealOrderedSubItem::SubItem { surreal_item_id } => {
                    other_item.id == surreal_item_id
                }
            })
    }

    pub(crate) fn has_review_frequency(&self) -> bool {
        self.surreal_item.review_frequency.is_some()
    }

    pub(crate) fn has_review_guidance(&self) -> bool {
        self.surreal_item.review_guidance.is_some()
    }

    pub(crate) fn is_a_review_due(&self) -> bool {
        match &self.surreal_item.review_frequency {
            Some(SurrealFrequency::NoneReviewWithParent) => false,
            Some(SurrealFrequency::Range {
                range_min,
                range_max: _range_max,
            }) => self.surreal_item.last_reviewed.as_ref().map_or(true, |x| {
                let last_reviewed: DateTime<Utc> = x.clone().into();
                let range_min: Duration = (*range_min).into();
                last_reviewed + range_min < *self.now
            }),
            Some(SurrealFrequency::Hourly) => {
                let one_hour = Duration::from_secs(60 * 60);
                self.surreal_item.last_reviewed.as_ref().map_or(true, |x| {
                    let last_reviewed: DateTime<Utc> = x.clone().into();
                    last_reviewed + one_hour < *self.now
                })
            }
            Some(SurrealFrequency::Daily) => {
                let one_day = Duration::from_secs(60 * 60 * 24);
                self.surreal_item.last_reviewed.as_ref().map_or(true, |x| {
                    let last_reviewed: DateTime<Utc> = x.clone().into();
                    last_reviewed + one_day < *self.now
                })
            }
            Some(SurrealFrequency::EveryFewDays) => {
                let three_days = Duration::from_secs(60 * 60 * 24 * 3);
                self.surreal_item.last_reviewed.as_ref().map_or(true, |x| {
                    let last_reviewed: DateTime<Utc> = x.clone().into();
                    last_reviewed + three_days < *self.now
                })
            }
            Some(SurrealFrequency::Weekly) => {
                let one_week = Duration::from_secs(60 * 60 * 24 * 7);
                self.surreal_item.last_reviewed.as_ref().map_or(true, |x| {
                    let last_reviewed: DateTime<Utc> = x.clone().into();
                    last_reviewed + one_week < *self.now
                })
            }
            Some(SurrealFrequency::BiMonthly) => {
                let two_months = Duration::from_secs(60 * 60 * 24 * 30 / 2);
                self.surreal_item.last_reviewed.as_ref().map_or(true, |x| {
                    let last_reviewed: DateTime<Utc> = x.clone().into();
                    last_reviewed + two_months < *self.now
                })
            }
            Some(SurrealFrequency::Monthly) => {
                let one_month = Duration::from_secs(60 * 60 * 24 * 30);
                self.surreal_item.last_reviewed.as_ref().map_or(true, |x| {
                    let last_reviewed: DateTime<Utc> = x.clone().into();
                    last_reviewed + one_month < *self.now
                })
            }
            Some(SurrealFrequency::Quarterly) => {
                let three_months = Duration::from_secs(60 * 60 * 24 * 30 * 3);
                self.surreal_item.last_reviewed.as_ref().map_or(true, |x| {
                    let last_reviewed: DateTime<Utc> = x.clone().into();
                    last_reviewed + three_months < *self.now
                })
            }
            Some(SurrealFrequency::SemiAnnually) => {
                let six_months = Duration::from_secs(60 * 60 * 24 * 30 * 6);
                self.surreal_item.last_reviewed.as_ref().map_or(true, |x| {
                    let last_reviewed: DateTime<Utc> = x.clone().into();
                    last_reviewed + six_months < *self.now
                })
            }
            Some(SurrealFrequency::Yearly) => {
                let one_year = Duration::from_secs(60 * 60 * 24 * 365);
                self.surreal_item.last_reviewed.as_ref().map_or(true, |x| {
                    let last_reviewed: DateTime<Utc> = x.clone().into();
                    last_reviewed + one_year < *self.now
                })
            }
            None => false,
        }
    }

    pub(crate) fn get_surreal_review_guidance(&self) -> &Option<SurrealReviewGuidance> {
        &self.surreal_item.review_guidance
    }
}

#[cfg(test)]
mod tests {
    use crate::data_storage::surrealdb_layer::{
        surreal_item::SurrealItemBuilder, surreal_tables::SurrealTablesBuilder,
    };

    use super::*;

    impl Item<'_> {
        pub(crate) fn has_active_children(&self, all_items: &HashMap<&RecordId, Item<'_>>) -> bool {
            self.surreal_item
                .smaller_items_in_priority_order
                .iter()
                .any(|x| match x {
                    SurrealOrderedSubItem::SubItem { surreal_item_id } => all_items
                        .get(surreal_item_id)
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
        let items = surreal_tables.make_items(&now);

        let smaller_item = items
            .get(smaller_item.id.as_ref().expect("Set above"))
            .expect("smaller item should be there");
        let visited = HashSet::default();
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
        let items = surreal_tables.make_items(&now);

        let under_test_parent_item = items
            .get(parent_item.id.as_ref().expect("Parent item has id"))
            .expect("Will find parent item in items");

        assert!(under_test_parent_item.has_active_children(&items));
    }
}
