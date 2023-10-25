use super::item::Item;

pub(crate) struct GroupedItem<'e> {
    item: &'e Item<'e>,
    group_item_type: GroupedItemType,
}

pub(crate) enum GroupedItemType {
    NotSet,
    /// This reason why this is being done
    MotivationalRequiredItem,
    /// Something that is intended to do when already in the area
    InTheAreaPreferredItem,
}
