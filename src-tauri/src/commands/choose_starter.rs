use tauri_plugin_store::StoreExt;

use crate::{
    state::ManagedApplicationState, templates::error::screen_error, traits::Entity, STORE_URI,
};

use super::alert_manager::AlertManager;
use super::replace_director::ResponseDirector;

#[tauri::command]
pub async fn choose_starter(
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
    handle: tauri::AppHandle,
    id: String,
) -> ResponseDirector {
    let starter = state
        .write_async(move |app_state| {
            let Some(ref comp) = app_state.competition else {
                return Err(screen_error("Cannot find competition"));
            };
            let starter = comp.starters.iter().find(|x| x.get_id() == id).cloned();
            app_state.starter = starter.clone();
            if app_state.starter.is_none() {
                return Err(screen_error("Cannot find Starter for competition"));
            };
            let store = handle
                .store(STORE_URI)
                .map_err(|e| screen_error(e.to_string().as_str()))?;
            store.set("state", serde_json::to_value((*app_state).clone()).ok());
            Ok(starter)
        })
        .await??;
    if let Some(ref starter) = starter {
        alert_manager.from_starter(starter);
    }
    crate::templates::scoresheet::scoresheet(state, alert_manager).await
}
