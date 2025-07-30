use hypertext::Renderable as _;

use crate::{
    commands::replace_director::{PageLocation, ReplaceDirector, ResponseDirector},
    state::ManagedApplicationState,
    templates::{error::screen_error, scoresheet::start_list_bar::get_starters_list},
};

#[tauri::command]
pub async fn filter_starters(
    state: tauri::State<'_, ManagedApplicationState>,
    value: Option<String>,
) -> ResponseDirector {
    let output = state
        .read_async(move |app_state| {
            let competition = app_state
                .competition()
                .ok_or_else(|| screen_error("Competition Not Found"))?;

            let current_starter_id = &app_state
                .starter()
                .ok_or_else(|| screen_error("Starter not found"))?
                .id;
            let judge = competition
                .jury
                .first()
                .ok_or_else(|| screen_error("Judge not found"))?;

            Ok(get_starters_list(&competition.starters, current_starter_id, judge, value).render())
        })
        .await??;

    Ok(ReplaceDirector::with_target(
        &PageLocation::StartersList,
        output,
    ))
}
