use std::fmt::Display;

use async_recursion::async_recursion;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{life_area::LifeArea, routine::Routine},
    menu::top_menu::present_top_menu,
    surrealdb_layer::{surreal_tables::SurrealTables, DataLayerCommands},
};

pub(crate) enum LifeAreaItem<'e> {
    ExistingRoutine(&'e Routine<'e>),
    NewLifeArea,
    NewRoutine(&'e LifeArea<'e>),
}

impl Display for LifeAreaItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifeAreaItem::ExistingRoutine(routine) => write!(f, "{}", routine.summary()),
            LifeAreaItem::NewLifeArea => write!(f, "New Life Area"),
            LifeAreaItem::NewRoutine(life_area) => {
                write!(f, "New Routine in {}", life_area.summary())
            }
        }
    }
}

impl<'e> LifeAreaItem<'e> {
    fn make_list(
        routines: &'e [Routine<'e>],
        life_areas: &'e [LifeArea<'e>],
    ) -> Vec<LifeAreaItem<'e>> {
        let mut list = Vec::new();
        for routine in routines {
            list.push(Self::ExistingRoutine(routine));
        }
        for life_area in life_areas {
            list.push(Self::NewRoutine(life_area));
        }
        list.push(Self::NewLifeArea);
        list
    }
}

#[async_recursion]
pub(crate) async fn change_routine(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let raw_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let life_areas = raw_tables.make_life_areas();
    let routines = raw_tables.make_routines();
    let list = LifeAreaItem::make_list(&routines, &life_areas);

    let selection = Select::new("", list).prompt();

    match selection {
        Ok(LifeAreaItem::ExistingRoutine(routine)) => {
            todo!(
                "TODO: Implement editing existing routine {}",
                routine.summary()
            );
        }
        Ok(LifeAreaItem::NewLifeArea) => {
            todo!("TODO: Implement creating new life area");
        }
        Ok(LifeAreaItem::NewRoutine(life_area)) => {
            todo!(
                "TODO: Implement creating new routine in {}",
                life_area.summary()
            );
        }
        Err(InquireError::OperationCanceled) => present_top_menu(send_to_data_storage_layer).await,
        Err(_) => {
            todo!("TODO: Implement cancelling");
        }
    }
    todo!()
}
