use hypertext::{rsx_static, Renderable};

use crate::commands::alert_manager::{AlertManager, AlertType};
use crate::commands::replace_director::{emit_page, PageLocation};
use crate::templates::html_elements;
use crate::templates::scoresheet::warnings::get_warnings;
use crate::{
    commands::replace_director::{ReplaceDirector, ResponseDirector},
    state::ManagedApplicationState,
};

const TARGET: &PageLocation = &PageLocation::ButtonMeeting;
const ALERT_TARGET: &PageLocation = &PageLocation::AlertsAndWarnings;
#[tauri::command]
pub fn toggle_meeting(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    Ok(ReplaceDirector::with_target(
        TARGET,
        match state.write(|app_state| {
            let position = app_state
                .competition()
                .and_then(|x| x.get_position())
                .unwrap_or_default();

            if let Some(s) = app_state.scoresheet_mut() {
                let val = s.warning_manager.meeting.toggle(&position);
                alert_manager.set(AlertType::Meeting, &position, val);
                emit_page(&app, ALERT_TARGET, get_warnings(alert_manager));
                val
            } else {
                false
            }
        })? {
            true => rsx_static! {<span data-active>"Active"</span>" Meeting"}.render(),
            false => hypertext::Rendered(rsx_static! {"Meeting"}.0.to_string()),
        },
    ))
}
