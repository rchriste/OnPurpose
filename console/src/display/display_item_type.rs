use std::fmt::Display;

use crate::surrealdb_layer::surreal_item::{SurrealItemType, SurrealMotivationKind};

use super::DisplayStyle;

pub(crate) struct DisplayItemType<'s> {
    item_type: &'s SurrealItemType,
    style: DisplayStyle,
}

impl<'s> Display for DisplayItemType<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.item_type {
            SurrealItemType::Goal(_) => {
                write!(f, "🪧")?;
                match self.style {
                    DisplayStyle::Abbreviated => Ok(()),
                    DisplayStyle::Full => write!(f, " Goal"),
                }
            }
            SurrealItemType::Motivation(kind) => {
                write!(f, "🎯")?;
                match self.style {
                    DisplayStyle::Abbreviated => {}
                    DisplayStyle::Full => write!(f, " Motivation")?,
                }
                match kind {
                    SurrealMotivationKind::NotSet => Ok(()),
                    SurrealMotivationKind::CoreWork => {
                        write!(f, "🏢")?;
                        match self.style {
                            DisplayStyle::Abbreviated => Ok(()),
                            DisplayStyle::Full => write!(f, " Core Work"),
                        }
                    }
                    SurrealMotivationKind::NonCoreWork => {
                        write!(f, "🏞")?;
                        match self.style {
                            DisplayStyle::Abbreviated => Ok(()),
                            DisplayStyle::Full => write!(f, " Non-Core Work"),
                        }
                    }
                }
            }
            SurrealItemType::Action => {
                write!(f, "🪜")?;
                match self.style {
                    DisplayStyle::Abbreviated => Ok(()),
                    DisplayStyle::Full => write!(f, " Action"),
                }
            }
            SurrealItemType::Undeclared => match self.style {
                DisplayStyle::Abbreviated => Ok(()),
                DisplayStyle::Full => write!(f, "❓ Undeclared"),
            },
            SurrealItemType::PersonOrGroup => {
                write!(f, "👤")?;
                match self.style {
                    DisplayStyle::Abbreviated => Ok(()),
                    DisplayStyle::Full => write!(f, " Person or Group"),
                }
            }
            SurrealItemType::IdeaOrThought => {
                write!(f, "💡")?;
                match self.style {
                    DisplayStyle::Abbreviated => Ok(()),
                    DisplayStyle::Full => write!(f, " Idea or Thought"),
                }
            }
        }
    }
}

impl<'s> DisplayItemType<'s> {
    pub(crate) fn new(style: DisplayStyle, item_type: &'s SurrealItemType) -> Self {
        DisplayItemType { item_type, style }
    }
}
