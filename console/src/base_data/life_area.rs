use crate::surrealdb_layer::surreal_life_area::SurrealLifeArea;

pub struct LifeArea<'s> {
    pub surreal_life_area: &'s SurrealLifeArea,
}

impl<'s> LifeArea<'s> {
    pub fn new(surreal_life_area: &'s SurrealLifeArea) -> Self {
        Self { surreal_life_area }
    }

    pub fn summary(&self) -> &str {
        &self.surreal_life_area.summary
    }
}
