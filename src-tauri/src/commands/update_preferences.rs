use crate::{commands::replace_director::ResponseDirector, state::{ManagedApplicationState, UserType}, templates::error::screen_error};

use super::{log_out, replace_director::ReplaceDirector};





#[tauri::command]
pub async fn update_comment_first(
	state: tauri::State<'_, ManagedApplicationState>,
	handle: tauri::AppHandle,
	value: &str,
) -> ResponseDirector {
	{
		let bool_value = value == "true";
		let mut app_state = state.write()
			.or_else(|_| {
				state.clear_poison();
				state.write()
			})
			.map_err(|_| screen_error("Cannot update value due to a poisoned lock"))?;
		match app_state.user {
			UserType::Judge(ref mut judge, _) => {
				judge.prefs.comment_last = bool_value;
				println!("{:?}", judge);
				return Ok(ReplaceDirector::none())
			}
			_ => ()
		}
	}
	return log_out::log_out(state.clone(), handle).await
}

#[tauri::command]
pub async fn update_show_trend(
	state: tauri::State<'_, ManagedApplicationState>,
	handle: tauri::AppHandle,
	value: &str,
) -> ResponseDirector {
	{
		let bool_value = value == "true";
		let mut app_state = state.write()
			.or_else(|_| {
				state.clear_poison();
				state.write()
			})
			.map_err(|_| screen_error("Cannot update value due to a poisoned lock"))?;
		match app_state.user {
			UserType::Judge(ref mut judge, _) => {
				judge.prefs.hide_trend = bool_value;
				return Ok(ReplaceDirector::none())
			}
			_ => ()
		}
	}
	return log_out::log_out(state.clone(), handle).await
}

#[tauri::command]
pub async fn update_auto_sign(
	state: tauri::State<'_, ManagedApplicationState>,
	handle: tauri::AppHandle,
	value: &str,
) -> ResponseDirector {
	{
		let bool_value = value == "true";
		let mut app_state = state.write()
			.or_else(|_| {
				state.clear_poison();
				state.write()
			})
			.map_err(|_| screen_error("Cannot update value due to a poisoned lock"))?;
		match app_state.user {
			UserType::Judge(ref mut judge, _) => {
				judge.prefs.manually_sign = bool_value;
				return Ok(ReplaceDirector::none())
			}
			_ => ()
		}
	}
	return log_out::log_out(state.clone(), handle).await
}