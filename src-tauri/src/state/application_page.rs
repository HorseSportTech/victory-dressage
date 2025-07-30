use tauri::Manager;
use tauri_plugin_store::StoreExt;

use crate::{
    commands::replace_director::{ReplaceDirector, ResponseDirector},
    domain::SurrealId,
    STATE, STORE_URI,
};

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
    pub fn set_location(self, handle: &tauri::AppHandle) -> ResponseDirector {
        let state = handle.state::<ManagedApplicationState>();
        state.write(|app_state| {
            app_state.page = self;

            if let Ok(e) = handle.store(STORE_URI) {
                e.set(STATE, serde_json::to_value((*app_state).clone()).ok())
            };
        })?;

        Ok(ReplaceDirector::none())
    }
}
