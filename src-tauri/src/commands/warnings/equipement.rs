use hypertext::{rsx, Renderable};
use crate::templates::html_elements;

use crate::{commands::replace_director::{ReplaceDirector, ResponseDirector}, state::ManagedApplicationState, templates::error::screen_error};

const TARGET: &'static str = "#button-equipment";
#[tauri::command]
pub fn toggle_equipment(
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
		let val = s.warning_manager.equipement.toggle(position);
		return Ok(ReplaceDirector::with_target(
			TARGET,
			rsx!{@if val{<span data-active>"Active"</span>}" Equipment"}.render()
		))
	}

	Ok(ReplaceDirector::with_target(TARGET,
		rsx!{{format!("Equipment")}}.render()
	))
}