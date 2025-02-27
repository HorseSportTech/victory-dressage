use tauri_plugin_store::StoreExt;

use crate::{state::ManagedApplicationState, STORE_URI};

pub trait Entity {
	fn key(&self) -> String;
	fn get_id(&self) -> String;
}
pub trait Storable 
	where Self: Sized + serde::de::DeserializeOwned + serde::ser::Serialize + Clone + Entity
{
	fn set(self, handle: &tauri::AppHandle) -> Result<(), tauri_plugin_store::Error> {
		let value = serde_json::to_value(&self)
			.map_err(|err| tauri_plugin_store::Error::Serialize(Box::new(err)))?;
		handle.store(STORE_URI)?.set(self.key(), value);
		Ok(())
	}
	fn get(handle: &tauri::AppHandle, key: &str) -> Result<Self, tauri_plugin_store::Error> {
		let json_value = handle.store(STORE_URI)?.get(key)
			.ok_or_else(|| tauri_plugin_store::Error::Io(std::io::Error::from(std::io::ErrorKind::NotFound)))?;
		serde_json::from_value::<Self>(json_value)
			.map_err(|err| tauri_plugin_store::Error::Deserialize(Box::new(err)))
	}
}


pub trait Fetchable
	where Self: Sized + serde::de::DeserializeOwned {
	async fn fetch(state:tauri::State<'_, ManagedApplicationState>) -> Result<Vec<Self>, tauri_plugin_http::Error>;
	async fn select(state:tauri::State<'_, ManagedApplicationState>, id: &str) -> Result<Self, tauri_plugin_http::Error>;
}