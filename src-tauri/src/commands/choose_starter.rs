use tauri_plugin_store::StoreExt;

use crate::{
    state::ManagedApplicationState, templates::error::screen_error, traits::Entity, STORE_URI,
};

use super::alert_manager::AlertManager;
use super::replace_director::ResponseDirector;

#[tauri::command]
pub async fn choose_starter(
    state: tauri::State<'_, ManagedApplicationState>,
    mut alert_manager: tauri::State<'_, AlertManager>,
    handle: tauri::AppHandle,
    id: &str,
) -> ResponseDirector {
    {
        let mut unlocked = state
            .write()
            .or_else(|_| {
                state.clear_poison();
                state.write()
            })
            .map_err(|err| screen_error(&err.to_string()))?;
        let Some(ref comp) = unlocked.competition else {
            return Err(screen_error("Cannot find competition"));
        };
        unlocked.starter = comp.starters.iter().find(|x| x.get_id() == id).cloned();
        if unlocked.starter.is_none() {
            return Err(screen_error("Cannot find Starter for competition"));
        };
        if let Some(ref starter) = unlocked.starter {
            alert_manager.from_starter(starter);
        }
        let store = handle
            .store(STORE_URI)
            .map_err(|e| screen_error(e.to_string().as_str()))?;
        store.set("state", serde_json::to_value((*unlocked).clone()).ok());
    }
    crate::templates::scoresheet::scoresheet(state, alert_manager).await
}
