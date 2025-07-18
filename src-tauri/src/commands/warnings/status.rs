use hypertext::Renderable;
use serde::Deserialize;

use crate::{
	commands::replace_director::{ReplaceDirector, ResponseDirector},
	domain::starter::StarterResult, state::ManagedApplicationState,
	templates::{error::screen_error, scoresheet::status_selection}
};

#[tauri::command]
pub fn change_competitor_status(
	state: tauri::State<'_, ManagedApplicationState>,
	value: WrappedStatus,
) -> ResponseDirector {
	let WrappedStatus(value) = value;
	let app_state = state.write()
		.or_else(|_|{state.clear_poison(); state.write()})
		.map_err(|_|screen_error("Could not increase error due to poisoned lock"))?;

	let mut starter = app_state.starter.clone()
		.ok_or_else(||screen_error("Could not increase error due to poisoned lock"))?;

	starter.status = value;
	Ok(ReplaceDirector::with_target(
		"#status-selector", 
		status_selection(starter.status).render()
	))
}
#[derive(serde::Serialize)]
#[serde(transparent)]
pub struct WrappedStatus(StarterResult);
impl<'de> Deserialize<'de> for WrappedStatus {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de> {
		let str = String::deserialize(deserializer)?;
		Ok(match str.as_str() {
			"Eliminated" => WrappedStatus(StarterResult::Eliminated(String::from(""))),
			"Withdrawn" => WrappedStatus(StarterResult::Withdrawn),
			"NoShow" => WrappedStatus(StarterResult::NoShow),
			"Retired" => WrappedStatus(StarterResult::Retired),
			"Placed" => WrappedStatus(StarterResult::Placed(0)),
			"NotPlaced" => WrappedStatus(StarterResult::NotPlaced(0)),
			_ => WrappedStatus(StarterResult::InProgress(0)),
		})
	}
}