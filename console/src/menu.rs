use std::fmt::Display;

pub(crate) mod inquire;
pub(crate) mod ratatui;

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
