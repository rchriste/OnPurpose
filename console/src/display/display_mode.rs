use std::fmt::Display;

use crate::base_data::mode::Mode;

pub(crate) struct DisplayMode<'s> {
    mode: &'s Mode<'s>,
}

impl Display for DisplayMode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mode.get_name())
    }
}

impl<'s> DisplayMode<'s> {
    pub(crate) fn new(mode: &'s Mode<'s>) -> Self {
        Self { mode }
    }
}
