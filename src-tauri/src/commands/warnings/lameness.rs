use crate::{
    commands::{
        alert_manager::{AlertManager, AlertType},
        replace_director::{emit_page, PageLocation},
    },
    templates::{html_elements, scoresheet::warnings::get_warnings},
};
use hypertext::{rsx_static, Renderable};

use crate::{
    commands::replace_director::{ReplaceDirector, ResponseDirector},
    state::ManagedApplicationState,
};

const TARGET: &PageLocation = &PageLocation::ButtonLameness;
const ALERT_TARGET: &PageLocation = &PageLocation::AlertsAndWarnings;
#[tauri::command]
pub fn toggle_lameness(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    Ok(ReplaceDirector::with_target(
        TARGET,
        match state.write(|app_state| {
            let position = app_state
                .competition
                .as_ref()
                .and_then(|x| x.get_position())
                .unwrap_or_default();

            if let Some(s) = app_state.scoresheet_mut() {
                let val = s.warning_manager.lameness.toggle(&position);
                alert_manager.set(AlertType::Lameness, &position, val);
                emit_page(&app, ALERT_TARGET, get_warnings(alert_manager));
                val
            } else {
                false
            }
        })? {
            true => rsx_static! {<span data-active>"Active"</span>" Lameness"}.render(),
            false => hypertext::Rendered(rsx_static! {"Lameness"}.0.to_string()),
        },
    ))
}
