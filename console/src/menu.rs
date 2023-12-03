use std::fmt::Display;

pub(crate) mod bullet_list_menu;
pub(crate) mod expectations;
pub(crate) mod select_higher_priority_than_this;
pub(crate) mod select_person_or_group;
pub(crate) mod staging_query;
pub(crate) mod top_menu;
pub(crate) mod unable_to_work_on_item_right_now;
pub(crate) mod update_item_summary;
pub(crate) enum YesOrNo {
    Yes,
    No,
}

impl Display for YesOrNo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YesOrNo::Yes => write!(f, "Yes"),
            YesOrNo::No => write!(f, "No"),
        }
    }
}

impl YesOrNo {
    fn make_list() -> Vec<Self> {
        vec![YesOrNo::Yes, YesOrNo::No]
    }
}
