use std::fmt::Display;

use crate::base_data::person_or_group::PersonOrGroup;

pub struct DisplayPersonOrGroup<'s> {
    person_or_group: &'s PersonOrGroup<'s>,
}

impl Display for DisplayPersonOrGroup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.person_or_group.get_summary())
    }
}

impl<'s> From<DisplayPersonOrGroup<'s>> for &PersonOrGroup<'s> {
    fn from(display_person_or_group: DisplayPersonOrGroup<'s>) -> Self {
        display_person_or_group.person_or_group
    }
}

impl<'s> DisplayPersonOrGroup<'s> {
    pub(crate) fn new(person_or_group: &'s PersonOrGroup<'s>) -> Self {
        Self { person_or_group }
    }
}
