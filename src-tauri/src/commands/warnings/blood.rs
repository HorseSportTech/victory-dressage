use hypertext::{rsx, Renderable};

use crate::{commands::replace_director::{ReplaceDirector, ResponseDirector}, state::ManagedApplicationState, templates::{html_elements, error::screen_error}};

const TARGET: &'static str = "#button-blood";
#[tauri::command]
pub fn toggle_blood(
	state: tauri::State<'_, ManagedApplicationState>,
) -> ResponseDirector {
	let mut app_state = state.write()
		.or_else(|_| {
			state.clear_poison();
			state.write()
		})
		.map_err(|_|screen_error("Cannot toggle blood due to a poisoned lock"))?;
	let position = app_state.competition.as_ref().and_then(|x| x.get_position())
		.unwrap_or_default();

	if let Some(s) = app_state.scoresheet() {
		let val = s.warning_manager.blood.toggle(position);
		return Ok(ReplaceDirector::with_target(
			TARGET,
			rsx!{@if val{<span data-active>"Active"</span>}" Blood"}.render()
			
		))
	}

	Ok(ReplaceDirector::with_target(TARGET,
		rsx!{{format!("Blood")}}.render()
	))
}