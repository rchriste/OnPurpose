use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use crate::new_mode::NewMode;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealMode {
    pub(crate) id: Option<Thing>,
    pub(crate) name: String,
    pub(crate) version: u32,
    pub(crate) parent: Option<Thing>,
    pub(crate) urgency_scope: Vec<SurrealScope>,
    pub(crate) importance_scope: Vec<SurrealScope>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealModeVersion0 {
    pub(crate) id: Option<Thing>,
    pub(crate) name: String,
    pub(crate) version: u32,
    pub(crate) parent: Option<Thing>,
}

impl From<SurrealModeVersion0> for SurrealMode {
    fn from(version0: SurrealModeVersion0) -> Self {
        SurrealMode {
            id: version0.id,
            name: version0.name,
            version: version0.version,
            parent: version0.parent,
            urgency_scope: vec![],
            importance_scope: vec![],
        }
    }
}

impl From<NewMode> for SurrealMode {
    fn from(new_mode: NewMode) -> Self {
        SurrealMode {
            id: None,
            name: new_mode.name,
            version: 1,
            parent: new_mode.parent,
            urgency_scope: vec![],
            importance_scope: vec![],
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealScope {
    pub(crate) in_scope: Vec<Thing>,
    pub(crate) out_of_scope: Vec<Thing>,
}

impl SurrealMode {
    pub(crate) const TABLE_NAME: &'static str = "modes";
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use crate::data_storage::surrealdb_layer::{
        data_layer_commands::{data_storage_connect_to_db, data_storage_endless_loop},
        surreal_mode::{self, SurrealModeVersion0},
        surreal_tables::SurrealTables,
    };

    #[tokio::test]
    async fn surreal_mode_upgrade_from_version0_to_version1() {
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle = tokio::spawn(async move {
            // data_storage_start_and_run(receiver, "mem://").await
            let db = data_storage_connect_to_db("mem://").await;

            //Setup the test scenario by saving out a Version0 entry that will need to be upgraded
            let mut version0 = SurrealModeVersion0 {
                id: None,
                name: "Test Mode".to_string(),
                version: 0,
                parent: None,
            };
            let created: SurrealModeVersion0 = db
                .create(surreal_mode::SurrealMode::TABLE_NAME)
                .content(version0.clone())
                .await
                .unwrap()
                .into_iter()
                .next()
                .unwrap();

            version0.id = created.id.clone();
            assert_eq!(version0, created);

            data_storage_endless_loop(db, receiver).await
        });

        let surreal_tables = SurrealTables::new(&sender).await.expect("test failed");

        assert!(!surreal_tables.surreal_modes.is_empty());

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }
}
