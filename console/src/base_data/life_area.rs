use crate::surrealdb_layer::surreal_life_area::SurrealLifeArea;

pub(crate) struct LifeArea<'s> {
    pub(crate) surreal_life_area: &'s SurrealLifeArea,
}

impl<'s> LifeArea<'s> {
    pub(crate) fn new(surreal_life_area: &'s SurrealLifeArea) -> Self {
        Self { surreal_life_area }
    }

    pub(crate) fn summary(&self) -> &str {
        &self.surreal_life_area.summary
    }
}
