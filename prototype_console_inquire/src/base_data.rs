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

impl From<SurrealItem> for Option<Thing> {
    fn from(value: SurrealItem) -> Self {
        value.id
    }
}

impl<'a> From<ToDo<'a>> for SurrealItem {
    fn from(value: ToDo) -> Self {
        value.surreal_item.clone()
    }
}

pub trait SurrealItemVecExtensions {
    fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<&'a SurrealItem>;
    fn filter_just_to_dos(&self) -> Vec<ToDo<'_>>;
    fn filter_just_hopes(&self) -> Vec<Hope<'_>>;
    fn filter_just_reasons(&self) -> Vec<Reason<'_>>;
}

impl SurrealItemVecExtensions for [SurrealItem] {
    fn lookup_from_record_id<'a>(&'a self, record_id: &RecordId) -> Option<&'a SurrealItem> {
        self.iter().find(|x| match x.get_id() {
            Some(v) => v == record_id,
            None => false,
        })
    }

    fn filter_just_to_dos(&self) -> Vec<ToDo<'_>> {
        self.iter()
            .filter_map(|x| {
                if x.item_type == ItemType::ToDo {
                    Some(ToDo {
                        id: &x.id,
                        summary: &x.summary,
                        finished: &x.finished,
                        surreal_item: x,
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
                if x.item_type == ItemType::Hope {
                    Some(Hope {
                        id: &x.id,
                        summary: &x.summary,
                        finished: &x.finished,
                        surreal_item: x,
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
                if x.item_type == ItemType::Reason {
                    Some(Reason {
                        id: &x.id,
                        summary: &x.summary,
                        finished: &x.finished,
                        surreal_item: x,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

impl SurrealItem {
    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }

    pub fn covered_by<'a>(&self, linkage: &[LinkageWithReferences<'a>]) -> Vec<&'a SurrealItem> {
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
    ) -> Vec<&'a SurrealItem> {
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
    pub id: &'a Option<Thing>,
    pub summary: &'a String,
    pub finished: &'a Option<Datetime>,
    surreal_item: &'a SurrealItem,
}

impl<'a> From<ToDo<'a>> for &'a Option<Thing> {
    fn from(value: ToDo<'a>) -> Self {
        value.id
    }
}

impl<'a> From<&ToDo<'a>> for &'a SurrealItem {
    fn from(value: &ToDo<'a>) -> Self {
        value.surreal_item
    }
}

impl<'a> PartialEq<SurrealItem> for ToDo<'a> {
    fn eq(&self, other: &SurrealItem) -> bool {
        //TODO: Add a static assert to notice if more fields are added to the ToDo<'a> struct
        self.id == &other.id && self.summary == &other.summary && self.finished == &other.finished
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
}

/// Could have a review_type with options for Milestone, StoppingPoint, and ReviewPoint
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Hope<'a> {
    pub id: &'a Option<Thing>,
    pub summary: &'a String,
    pub finished: &'a Option<Datetime>,
    surreal_item: &'a SurrealItem,
}

impl<'a> From<Hope<'a>> for Option<Thing> {
    fn from(value: Hope) -> Self {
        value.id.clone()
    }
}

impl PartialEq<Hope<'_>> for SurrealItem {
    fn eq(&self, other: &Hope<'_>) -> bool {
        self == other.surreal_item
    }
}

impl PartialEq<SurrealItem> for Hope<'_> {
    fn eq(&self, other: &SurrealItem) -> bool {
        self.surreal_item == other
    }
}

impl<'a> Hope<'a> {
    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }

    pub fn covered_by(&self, linkage: &[LinkageWithReferences<'a>]) -> Vec<&'a SurrealItem> {
        self.surreal_item.covered_by(linkage)
    }

    pub fn who_am_i_covering(&self, linkage: &[LinkageWithReferences<'a>]) -> Vec<&'a SurrealItem> {
        self.surreal_item.who_am_i_covering(linkage)
    }
}

/// Could have a reason_type with options for Commitment, Maintenance, or Value
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Reason<'a> {
    pub id: &'a Option<Thing>,
    pub summary: &'a String,
    pub finished: &'a Option<Datetime>,
    surreal_item: &'a SurrealItem,
}

impl<'a> From<Reason<'a>> for Option<Thing> {
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
    pub smaller: &'a SurrealItem,
    pub parent: &'a SurrealItem,
}

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
            smaller: value
                .smaller
                .get_id()
                .as_ref()
                .expect("Should already be in the DB")
                .clone(),
            parent: value
                .parent
                .get_id()
                .as_ref()
                .expect("Should already be in the DB")
                .clone(),
        }
    }
}

pub fn convert_linkage_with_record_ids_to_references<'a>(
    linkage_with_record_ids: &[LinkageWithRecordIds],
    items: &'a [SurrealItem],
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

#[derive(PartialEq, Eq)]
pub enum Item<'a> {
    ToDo(&'a ToDo<'a>),
    Hope(&'a Hope<'a>),
    Reason(&'a Reason<'a>),
}

impl<'a> From<&'a ToDo<'a>> for Item<'a> {
    fn from(value: &'a ToDo) -> Self {
        Item::ToDo(value)
    }
}

impl<'a> From<&'a Hope<'a>> for Item<'a> {
    fn from(value: &'a Hope) -> Self {
        Item::Hope(value)
    }
}

impl<'a> From<&'a Reason<'a>> for Item<'a> {
    fn from(value: &'a Reason) -> Self {
        Item::Reason(value)
    }
}

impl SurrealItem {
    pub fn find_parents<'a>(
        &self,
        linkage: &'a [LinkageWithReferences<'a>],
    ) -> Vec<&'a SurrealItem> {
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

impl<'a> Item<'a> {
    pub fn from_to_do(to_do: &'a ToDo) -> Item<'a> {
        Item::ToDo(to_do)
    }

    pub fn from_hope(hope: &'a Hope) -> Item<'a> {
        Item::Hope(hope)
    }

    pub fn from_reason_item(reason: &'a Reason) -> Item<'a> {
        Item::Reason(reason)
    }

    pub fn get_id(&'a self) -> &'a Option<Thing> {
        match self {
            Item::ToDo(to_do) => to_do.id,
            Item::Hope(hope) => hope.id,
            Item::Reason(reason_item) => reason_item.id,
        }
    }

    pub fn is_finished(&'a self) -> bool {
        match self {
            Item::ToDo(i) => i.is_finished(),
            Item::Hope(i) => i.is_finished(),
            Item::Reason(i) => i.is_finished(),
        }
    }
}
