use super::why_in_scope_and_action_with_item_status::WhyInScopeAndActionWithItemStatus;

pub(crate) enum UrgencyLevelItemWithItemStatus<'e> {
    SingleItem(WhyInScopeAndActionWithItemStatus<'e>),
    MultipleItems(Vec<WhyInScopeAndActionWithItemStatus<'e>>),
}

impl<'e> UrgencyLevelItemWithItemStatus<'e> {
    pub(crate) fn new_multiple_items(items: Vec<WhyInScopeAndActionWithItemStatus<'e>>) -> Self {
        Self::MultipleItems(items)
    }
}
