use crate::{
    commands::{
        alert_manager::{AlertManager, AlertType},
        replace_director::emit_page,
    },
    templates::{html_elements, scoresheet::warnings::get_warnings},
};
use hypertext::{rsx, Renderable};

use crate::{
    commands::replace_director::{ReplaceDirector, ResponseDirector},
    state::ManagedApplicationState,
    templates::error::screen_error,
};

const TARGET: &'static str = "#button-equipment";
const ALERT_TARGET: &'static str = "#alerts-and-warnings";
#[tauri::command]
pub fn toggle_equipment(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Cannot toggle blood due to a poisoned lock"))?;
    let position = app_state
        .competition
        .as_ref()
        .and_then(|x| x.get_position())
        .unwrap_or_default();

    if let Some(s) = app_state.scoresheet_mut() {
        let val = s.warning_manager.equipement.toggle(&position);
        alert_manager.set(AlertType::Equipment, &position, val);
        emit_page(&app, ALERT_TARGET, get_warnings(alert_manager));
        return Ok(ReplaceDirector::with_target(
            TARGET,
            rsx! {@if val{<span data-active>"Active"</span>}" Equipment"}.render(),
        ));
    }

    Ok(ReplaceDirector::with_target(
        TARGET,
        rsx! {{format!("Equipment")}}.render(),
    ))
}

