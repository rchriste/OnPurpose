use crate::surrealdb_layer::surreal_routine::SurrealRoutine;

use super::life_area::LifeArea;

pub struct Routine<'s> {
    pub surreal_routine: &'s SurrealRoutine,
    pub parent_life_area: &'s LifeArea<'s>,
}

impl<'s> Routine<'s> {
    pub fn new(surreal_routine: &'s SurrealRoutine, parent_life_area: &'s LifeArea<'s>) -> Self {
        Self {
            surreal_routine,
            parent_life_area,
        }
    }

    pub fn summary(&self) -> &str {
        &self.surreal_routine.summary
    }
}
