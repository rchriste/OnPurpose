use chrono::{DateTime, Datelike, Local};
use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};
use surrealdb_extra::table::Table;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "item")]
pub struct SurrealItem {
    pub id: Option<Thing>,
    pub summary: String,
    pub finished: Option<Datetime>,
    pub item_type: ItemType,
}

pub trait SurrealItemVecExtensions {
    fn make_items<'a>(&'a self, requirements: &'a [SurrealRequirement]) -> Vec<Item<'a>>;
}

impl SurrealItemVecExtensions for [SurrealItem] {
    fn make_items<'a>(&'a self, requirements: &'a [SurrealRequirement]) -> Vec<Item<'a>> {
        self.iter().map(|x| x.make_item(requirements)).collect()
    }
}

impl SurrealItem {
    pub fn make_item<'a>(&'a self, requirements: &'a [SurrealRequirement]) -> Item<'a> {
        let my_requirements = requirements
            .iter()
            .filter(|x| {
                &x.requirement_for
                    == self
                        .id
                        .as_ref()
                        .expect("Item should already be in the database and have an id")
            })
            .collect();

        Item {
            id: self
                .id
                .as_ref()
                .expect("Item should already be in the database and have an id"),
            summary: &self.summary,
            finished: &self.finished,
            item_type: &self.item_type,
            requirements: my_requirements,
            surreal_item: self,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Item<'a> {
    pub id: &'a Thing,
    pub summary: &'a String,
    pub finished: &'a Option<Datetime>,
    pub item_type: &'a ItemType,
    pub requirements: Vec<&'a SurrealRequirement>,
    surreal_item: &'a SurrealItem,
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

impl From<SurrealItem> for Option<Thing> {
    fn from(value: SurrealItem) -> Self {
        value.id
    }
}

impl<'a> From<ToDo<'a>> for SurrealItem {
    fn from(value: ToDo<'a>) -> Self {
        value.item.surreal_item.clone()
    }
}

pub trait ItemVecExtensions {
    fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<&'a Item>;
    fn filter_just_to_dos(&self) -> Vec<ToDo<'_>>;
    fn filter_just_hopes(&self) -> Vec<Hope<'_>>;
    fn filter_just_reasons(&self) -> Vec<Reason<'_>>;
}

impl<'b> ItemVecExtensions for [Item<'b>] {
    fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<&'a Item> {
        self.iter().find(|x| x.id == record_id)
    }

    fn filter_just_to_dos(&self) -> Vec<ToDo<'_>> {
        self.iter()
            .filter_map(|x| {
                if x.item_type == &ItemType::ToDo {
                    Some(ToDo {
                        id: x.id,
                        summary: x.summary,
                        finished: x.finished,
                        item: x,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn filter_just_hopes(&self) -> Vec<Hope<'_>> {
        self.iter()
            .filter_map(|x| {
                if x.item_type == &ItemType::Hope {
                    Some(Hope {
                        id: x.id,
                        summary: x.summary,
                        finished: x.finished,
                        item: x,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn filter_just_reasons(&self) -> Vec<Reason<'_>> {
        self.iter()
            .filter_map(|x| {
                if x.item_type == &ItemType::Reason {
                    Some(Reason {
                        id: x.id,
                        summary: x.summary,
                        finished: x.finished,
                        item: x,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

impl<'b> Item<'b> {
    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }

    pub fn covered_by<'a>(&self, linkage: &[LinkageWithReferences<'a>]) -> Vec<&'a Item<'a>> {
        linkage
            .iter()
            .filter_map(|x| {
                if x.parent == self {
                    Some(x.smaller)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn who_am_i_covering<'a>(
        &self,
        linkage: &[LinkageWithReferences<'a>],
    ) -> Vec<&'a Item<'a>> {
        linkage
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
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum ItemType {
    Question,
    ToDo,
    Hope,
    Reason,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ToDo<'a> {
    pub id: &'a Thing,
    pub summary: &'a String,
    pub finished: &'a Option<Datetime>,
    item: &'a Item<'a>,
}

impl<'a> From<ToDo<'a>> for &'a Thing {
    fn from(value: ToDo<'a>) -> Self {
        value.id
    }
}

impl<'a> From<&ToDo<'a>> for &'a SurrealItem {
    fn from(value: &ToDo<'a>) -> Self {
        value.item.into()
    }
}

impl<'a> From<&ToDo<'a>> for &'a Item<'a> {
    fn from(value: &ToDo<'a>) -> Self {
        value.item
    }
}

impl<'a> From<ToDo<'a>> for Item<'a> {
    fn from(value: ToDo<'a>) -> Self {
        value.item.clone()
    }
}

impl<'a> PartialEq<Item<'a>> for ToDo<'a> {
    fn eq(&self, other: &Item<'a>) -> bool {
        //TODO: Add a static assert to notice if more fields are added to the ToDo<'a> struct
        self.id == other.id && self.summary == other.summary && self.finished == other.finished
    }
}

impl<'a> ToDo<'a> {
    pub fn is_covered(&self, linkage: &[LinkageWithReferences<'_>]) -> bool {
        let mut covered_by = linkage.iter().filter(|x| self == x.parent);
        //Now see if the items that are covering are finished or active
        covered_by.any(|x| !x.smaller.is_finished())
    }

    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }

    pub fn is_requirements_met(&self, date: &DateTime<Local>) -> bool {
        !self
            .item
            .requirements
            .iter()
            .any(|x| match x.requirement_type {
                RequirementType::NotSunday => date.weekday().num_days_from_sunday() == 0,
            })
    }
}

/// Could have a review_type with options for Milestone, StoppingPoint, and ReviewPoint
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Hope<'a> {
    pub id: &'a Thing,
    pub summary: &'a String,
    pub finished: &'a Option<Datetime>,
    item: &'a Item<'a>,
}

impl<'a> From<Hope<'a>> for Thing {
    fn from(value: Hope) -> Self {
        value.id.clone()
    }
}

impl<'a> From<&'a Hope<'a>> for &'a SurrealItem {
    fn from(value: &'a Hope<'a>) -> Self {
        value.item.into()
    }
}

impl PartialEq<Hope<'_>> for Item<'_> {
    fn eq(&self, other: &Hope<'_>) -> bool {
        self == other.item
    }
}

impl PartialEq<Item<'_>> for Hope<'_> {
    fn eq(&self, other: &Item) -> bool {
        self.item == other
    }
}

impl<'a> Hope<'a> {
    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }

    pub fn covered_by(&self, linkage: &[LinkageWithReferences<'a>]) -> Vec<&'a Item<'a>> {
        self.item.covered_by(linkage)
    }

    pub fn who_am_i_covering(&self, linkage: &[LinkageWithReferences<'a>]) -> Vec<&'a Item<'a>> {
        self.item.who_am_i_covering(linkage)
    }
}

/// Could have a reason_type with options for Commitment, Maintenance, or Value
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Reason<'a> {
    pub id: &'a Thing,
    pub summary: &'a String,
    pub finished: &'a Option<Datetime>,
    item: &'a Item<'a>,
}

impl<'a> From<Reason<'a>> for Thing {
    fn from(value: Reason<'a>) -> Self {
        value.id.clone()
    }
}

impl<'a> Reason<'a> {
    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }
}

pub struct LinkageWithReferences<'a> {
    pub smaller: &'a Item<'a>,
    pub parent: &'a Item<'a>,
}

//TODO: Rename to SurrealCoverings and table name "coverings"
#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "linkage")]
pub struct LinkageWithRecordIds {
    pub id: Option<Thing>,
    pub smaller: RecordId,
    pub parent: RecordId,
}

impl<'a> From<LinkageWithReferences<'a>> for LinkageWithRecordIds {
    fn from(value: LinkageWithReferences<'a>) -> Self {
        LinkageWithRecordIds {
            id: None,
            smaller: value.smaller.id.clone(),
            parent: value.parent.id.clone(),
        }
    }
}

pub fn convert_linkage_with_record_ids_to_references<'a>(
    linkage_with_record_ids: &[LinkageWithRecordIds],
    items: &'a [Item<'a>],
) -> Vec<LinkageWithReferences<'a>> {
    linkage_with_record_ids
        .iter()
        .map(|x| LinkageWithReferences {
            smaller: items.lookup_from_record_id(&x.smaller).unwrap(),
            parent: items.lookup_from_record_id(&x.parent).unwrap(),
        })
        .collect()
}

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "processed_text")]
pub struct ProcessedText {
    pub id: Option<Thing>,
    pub text: String,
    pub when_written: Datetime,
    pub for_item: RecordId,
}

impl Item<'_> {
    pub fn find_parents<'a>(&self, linkage: &'a [LinkageWithReferences<'a>]) -> Vec<&'a Item<'a>> {
        linkage
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
}

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "requirements")]
pub struct SurrealRequirement {
    pub id: Option<Thing>,
    pub requirement_for: RecordId,
    pub requirement_type: RequirementType,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum RequirementType {
    NotSunday,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Requirement<'a> {
    pub requirement_for: &'a SurrealItem,
    pub requirement_type: &'a RequirementType,
    surreal_requirement: &'a SurrealRequirement,
}

impl<'a> From<&Requirement<'a>> for &'a SurrealRequirement {
    fn from(value: &Requirement<'a>) -> Self {
        value.surreal_requirement
    }
}

impl<'a> From<Requirement<'a>> for &'a SurrealRequirement {
    fn from(value: Requirement<'a>) -> Self {
        value.surreal_requirement
    }
}
