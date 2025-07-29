use crate::templates::html_elements;
use hypertext::{rsx_static, Rendered};

use crate::{
    commands::{
        alert_manager::{AlertManager, AlertType},
        replace_director::{emit_page, PageLocation, ReplaceDirector, ResponseDirector},
    },
    state::ManagedApplicationState,
    templates::scoresheet::warnings::get_warnings,
};

const TARGET: &PageLocation = &PageLocation::ButtonBlood;
const ALERT_TARGET: &PageLocation = &PageLocation::AlertsAndWarnings;
#[tauri::command]
pub fn toggle_blood(
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
                let val = s.warning_manager.blood.toggle(&position);
                alert_manager.set(AlertType::Blood, &position, val);
                emit_page(&app, ALERT_TARGET, get_warnings(alert_manager));
                val
            } else {
                false
            }
        })? {
            true => Rendered(
                rsx_static! {<span data-active>"Active"</span>" Blood"}
                    .0
                    .to_string(),
            ),
            false => Rendered(rsx_static! {"Blood"}.0.to_string()),
        },
    ))
}
