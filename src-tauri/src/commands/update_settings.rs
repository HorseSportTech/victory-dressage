use crate::state::ManagedApplicationState;
use crate::templates::error::screen_error;
use crate::templates::settings::button_freestyle_mode;
use hypertext::Renderable;

use super::replace_director::{ReplaceDirector, ResponseDirector};

#[tauri::command]
pub async fn toggle_freestyle_mode(
    state: tauri::State<'_, ManagedApplicationState>,
) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Cannot access judge preferences due to poisoned lock"))?;
    app_state.auto_freestyle = !app_state.auto_freestyle;
    Ok(ReplaceDirector::with_target(
        "#freestyle-mode-btn",
        button_freestyle_mode(app_state.auto_freestyle).render(),
    ))
}
