use tauri::Manager;
use tauri_plugin_store::StoreExt;

use crate::{commands::replace_director::{ReplaceDirector, ResponseDirector}, domain::SurrealId, templates::error::screen_error, STORE_URI};

use super::ManagedApplicationState;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ApplicationPage {
	Login,
	LoginJudge,
	Welcome,
	CompetitionList,
	Scoresheet(SurrealId),
	Settings,
	Preferences,
	FinalResult,
	Error,
}
impl ApplicationPage {

	pub fn set_location<'a>(self, handle: &'a tauri::AppHandle) -> ResponseDirector {
	
		let state = handle.state::<ManagedApplicationState>();
		let mut app_state = state.write().or_else(|_|{
				state.clear_poison();
				state.write()
			})
			.map_err(|_| screen_error("Error writing new data to app state"))?;
		app_state.page = self;

		if let Ok(e) = handle.store(STORE_URI) {
			e.set(
				"state", 
				serde_json::to_value((*app_state).clone()).ok()
			)

		};

		Ok(ReplaceDirector::none())
	}
}
