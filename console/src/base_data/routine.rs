use crate::surrealdb_layer::surreal_routine::SurrealRoutine;

pub(crate) struct Routine<'s> {
    pub(crate) surreal_routine: &'s SurrealRoutine,
}

impl<'s> Routine<'s> {
    pub(crate) fn new(surreal_routine: &'s SurrealRoutine) -> Self {
        Self { surreal_routine }
    }

    pub(crate) fn summary(&self) -> &str {
        &self.surreal_routine.summary
    }
}
