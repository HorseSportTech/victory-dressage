use crate::{
    state::ManagedApplicationState,
    templates::{self, error::screen_error},
    traits::Entity,
};

use super::alert_manager::AlertManager;
use super::replace_director::ResponseDirector;

#[tauri::command]
pub async fn choose_starter(
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
    _handle: tauri::AppHandle,
    id: String,
) -> ResponseDirector {
    let starter = state
        .write_async(move |app_state| {
            app_state.score_debounces = Default::default();
            let comp = app_state
                .competition()
                .ok_or_else(|| screen_error("Cannot find competition"))?;
            let starter = comp.starters.iter().find(|x| x.get_id() == id).cloned();
            app_state.starter_id = starter.as_ref().map(|x| x.id.clone());
            if app_state.starter_id.is_none() {
                return Err(screen_error("Cannot find Starter for competition"));
            };
            Ok(starter)
        })
        .await??;
    if let Some(ref starter) = starter {
        alert_manager.merge_starter(starter);
    }
    templates::scoresheet::scoresheet(state, alert_manager).await
}
