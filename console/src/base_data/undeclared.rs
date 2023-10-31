use super::item::Item;

pub(crate) struct Undeclared<'s> {
    pub(crate) item: &'s Item<'s>,
}

impl<'s> Undeclared<'s> {
    pub(crate) fn new(item: &'s Item<'s>) -> Self {
        Self { item }
    }

    pub(crate) fn get_item(&'s self) -> &'s Item<'s> {
        self.item
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.item.is_finished()
    }
}
