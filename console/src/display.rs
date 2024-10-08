pub(crate) mod display_action_with_item_status;
pub(crate) mod display_dependencies_with_item_node;
pub(crate) mod display_duration;
pub(crate) mod display_duration_one_unit;
pub(crate) mod display_item;
pub(crate) mod display_item_node;
pub(crate) mod display_item_status;
pub(crate) mod display_item_type;
pub(crate) mod display_scheduled_item;
pub(crate) mod display_urgency_plan;

pub(crate) enum DisplayStyle {
    Abbreviated,
    Full,
}
