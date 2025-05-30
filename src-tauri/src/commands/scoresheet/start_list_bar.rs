use hypertext::Renderable as _;

use crate::{
    commands::replace_director::{ReplaceDirector, ResponseDirector},
    state::ManagedApplicationState,
    templates::{error::screen_error, scoresheet::start_list_bar::get_starters_list},
};

#[tauri::command]
pub async fn filter_starters(
    state: tauri::State<'_, ManagedApplicationState>,
    value: Option<String>,
) -> ResponseDirector {
    let app_state = state
        .read()
        .map_err(|_| screen_error("Unexpected poisoned lock"))?;
    let competition = app_state
        .competition
        .as_ref()
        .ok_or_else(|| screen_error("Competition Not Found"))?;

    let current_starter = &app_state
        .starter
        .as_ref()
        .ok_or_else(|| screen_error("Starter not found"))?
        .id;
    let judge = competition
        .jury
        .first()
        .ok_or_else(|| screen_error("Judge not found"))?;

    let output = get_starters_list(&competition.starters, current_starter, judge, value).render();

    Ok(ReplaceDirector::with_target("#starters-list", output))
}
