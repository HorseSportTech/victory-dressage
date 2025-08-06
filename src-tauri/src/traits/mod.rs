use crate::state::{ManagedApplicationState, StatefulRequestError};

pub trait Entity {
    fn key(&self) -> String;
    fn get_id(&self) -> String;
}

pub trait Fetchable
where
    Self: Sized + serde::de::DeserializeOwned,
{
    async fn fetch(
        state: &tauri::State<'_, ManagedApplicationState>,
    ) -> Result<Vec<Self>, StatefulRequestError>;
    async fn select(
        state: &tauri::State<'_, ManagedApplicationState>,
        id: &str,
    ) -> Result<Self, StatefulRequestError>;
}
