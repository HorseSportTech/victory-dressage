use crate::{state::{ApplicationState, ManagedApplicationState}, templates::error::screen_error};

use super::replace_director::ResponseDirector;

#[tauri::command]
pub async fn log_out(
	state: tauri::State<'_, ManagedApplicationState>,
	handle: tauri::AppHandle,
) -> ResponseDirector {
	{
		let mut logout = state.write()
			.or_else(|_| {
				state.clear_poison();
				state.write()
			})
			.map_err(|_|screen_error("Cannot log out session"))?;
	
		*logout = ApplicationState::new();
	}

	crate::templates::login::login(state, handle).await
}