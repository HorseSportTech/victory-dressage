use crate::state::ManagedApplicationState;
use crate::templates::error::screen_error;
use crate::templates::settings::button_freestyle_mode;
use hypertext::Renderable;

use super::replace_director::{PageLocation, ReplaceDirector, ResponseDirector};

#[tauri::command]
pub async fn toggle_freestyle_mode(
    state: tauri::State<'_, ManagedApplicationState>,
) -> ResponseDirector {
    let use_auto_freestyle = state
        .write_async(|app_state| {
            app_state.auto_freestyle = !app_state.auto_freestyle;
            app_state.auto_freestyle
        })
        .await
        .map_err(|_| screen_error("Cannot access judge preferences due to poisoned lock"))?;
    Ok(ReplaceDirector::with_target(
        &PageLocation::FreestyleModeBtn,
        button_freestyle_mode(use_auto_freestyle).render(),
    ))
}
