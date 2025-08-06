use std::sync::Arc;

use tauri::Runtime;
use tauri_plugin_store::StoreExt;

use crate::commands::signature::Signature;
use crate::debug;
use crate::domain::show::{Show, Shows};
use crate::domain::SurrealId;

use super::application_state::ApplicationId;
use super::ApplicationState;

impl Storable for ApplicationState {
    type Key = ();
    const KEY: &str = "STATE";

    fn store(&self, handle: &tauri::AppHandle) {
        let store = get_store_helper(handle);
        if let Some(ref show) = self.show {
            show.store(handle);
        }
        _set_helper(store, Self::KEY, self);
    }
}

impl Storable for Show {
    type Key = SurrealId;
    const KEY: &str = "";

    fn store(&self, handle: &tauri::AppHandle) {
        _set_helper(get_store_helper(handle), &self.id.to_string(), self);
    }

    fn retrieve_key(handle: &tauri::AppHandle, key: Self::Key) -> Option<Self> {
        _get_helper(get_store_helper(handle), &key.to_string())
    }

    fn retrieve(_handle: &tauri::AppHandle) -> Option<Self> {
        unimplemented!("Use version with key")
    }
    fn delete_stored(&self, handle: &tauri::AppHandle) {
        let store = get_store_helper(handle);
        store.delete(self.id.to_string());
    }
}
impl Storable for Shows {
    type Key = ();
    const KEY: &str = "SHOWS";
}
impl Storable for Signature {
    type Key = ();
    const KEY: &str = "TEMP_SIGNATURE";
}
impl Storable for ApplicationId {
    type Key = ();
    const KEY: &str = "APPLICATION_ID";
}

pub trait Storable: serde::Serialize + serde::de::DeserializeOwned + Sized {
    type Key;
    const KEY: &str;
    fn store(&self, handle: &tauri::AppHandle) {
        debug!(dim, "Store {}", Self::KEY);
        _set_helper(get_store_helper(handle), Self::KEY, self);
    }
    fn retrieve_key(_handle: &tauri::AppHandle, _key: Self::Key) -> Option<Self> {
        unimplemented!("No key to use")
    }
    fn retrieve(handle: &tauri::AppHandle) -> Option<Self> {
        _get_helper(get_store_helper(handle), Self::KEY)
    }
    fn delete_stored(&self, handle: &tauri::AppHandle) {
        let store = get_store_helper(handle);
        store.delete(Self::KEY);
    }
}

fn get_store_helper<R: Runtime>(handle: &tauri::AppHandle<R>) -> Arc<tauri_plugin_store::Store<R>> {
    StoreExt::store(handle, env!("STORE_URI")).expect("Must get the store")
}
fn _get_helper<V: serde::de::DeserializeOwned, R: Runtime>(
    store: Arc<tauri_plugin_store::Store<R>>,
    key: &str,
) -> Option<V> {
    store
        .get(key)
        .and_then(|val| serde_json::from_value(val).ok())
}

fn _set_helper<V: serde::Serialize, R: Runtime>(
    store: Arc<tauri_plugin_store::Store<R>>,
    key: &str,
    object: V,
) {
    let value = serde_json::to_value(object).expect("This must parse");
    store.set(key, value);
}
