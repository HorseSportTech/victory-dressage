use hypertext::{rsx, rsx_static, Renderable};

use crate::{
    commands::{
        alert_manager::{AlertManager, AlertType},
        replace_director::{emit_page, ReplaceDirector, ResponseDirector},
    },
    state::ManagedApplicationState,
    templates::{error::screen_error, html_elements, scoresheet::warnings::get_warnings},
};

const TARGET: &'static str = "#button-blood";
const ALERT_TARGET: &'static str = "#alerts-and-warnings";
#[tauri::command]
pub fn toggle_blood(
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
        let val = s.warning_manager.blood.toggle(&position);
        alert_manager.set(AlertType::Blood, &position, val);
        emit_page(&app, ALERT_TARGET, get_warnings(alert_manager));

        return Ok(ReplaceDirector::with_target(
            TARGET,
            rsx! {@if val{<span data-active>"Active"</span>}" Blood"}.render(),
        ));
    }

    Ok(ReplaceDirector::with_target(
        TARGET,
        hypertext::Rendered(rsx_static! {"Blood"}.0.to_string()),
    ))
}
