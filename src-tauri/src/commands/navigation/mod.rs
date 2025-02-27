use crate::{commands::replace_director::ResponseDirector, domain::show::Shows, state::ManagedApplicationState, templates::error::screen_error, traits::{Entity, Storable}};
use super::super::state::application_page::ApplicationPage;



#[tauri::command]
pub async fn page_x_judge_login(
	state: tauri::State<'_,ManagedApplicationState>,
	handle: tauri::AppHandle,
) -> ResponseDirector {
  	super::super::templates::choose_judge::choose_judge(state, handle).await
}

#[tauri::command]
pub async fn page_x_welcome(
	state: tauri::State<'_,ManagedApplicationState>,
	handle: tauri::AppHandle,
) -> ResponseDirector {
	// TODO: update state and store show list
  	super::super::templates::welcome::welcome(state, handle).await
}

#[tauri::command]
pub async fn page_x_preferences(
	state: tauri::State<'_,ManagedApplicationState>,
	handle: tauri::AppHandle,
) -> ResponseDirector {
	match super::super::templates::preferences::get_preferences(state.clone(), handle).await {
		Ok(page) => {
			state.write().unwrap().page = ApplicationPage::Preferences;
			Ok(page)
		},
		Err(err) => Err(err),
	}
}
#[tauri::command]
pub async fn page_x_settings(
	state: tauri::State<'_,ManagedApplicationState>,
	handle: tauri::AppHandle,
) -> ResponseDirector {
	match super::super::templates::settings::get_settings(state.clone(), handle).await {
		Ok(page) => {
			state.write().unwrap().page = ApplicationPage::Settings;
			Ok(page)
		},
		Err(err) => Err(err),
	}
}

#[tauri::command]
pub async fn page_x_competition_list(
	state: tauri::State<'_,ManagedApplicationState>,
	handle: tauri::AppHandle,
	id: String,
) -> ResponseDirector {
	{
		if state.read().or_else(|_|{state.clear_poison();state.read()})
			.map_err(|_|screen_error("Cannot access show due to a poisoned lock"))?
			.show.as_ref().is_none_or(|x|x.get_id() != id) {
			let shows = Shows::get(&handle, "shows")
				.map_err(|_|screen_error("Cannot find shows to navigate"))?;
			
			state.write().or_else(|_|{state.clear_poison();state.write()})
				.map_err(|_|screen_error("Cannot access show due to a poisoned lock"))?
				.show = shows.0.into_iter().find(|x|x.get_id() == id);
		}
	}
  	super::super::templates::competition_list::competition_list(state, handle, id).await
}

#[tauri::command]
pub async fn page_x_scoresheet(
	state: tauri::State<'_,ManagedApplicationState>,
	_handle: tauri::AppHandle,
	id: String,
) -> ResponseDirector {
	// TODO: update state
	let competition = state.read()
		.map_err(|_|screen_error("Poisoned lock fetching scoresheet"))?
		.show.as_ref()
		.and_then(|x| {
			x.competitions.iter()
				.find(|c| c.id.id() == id)
				.cloned()
		})
		.ok_or_else(||screen_error("Competition not found for scoresheet"))?;


	let starter = competition.starters.iter()
		.find(|x| !x.status.is_finished())
		.or_else(||competition.starters.first());
	{
		let mut app = state.write()
			.or_else(|_| {
				state.clear_poison();
				state.write()
			})
			.map_err(|_| screen_error("Failed to write starter to state due to poisoned lock"))?;
		app.competition = Some(competition.clone());
		app.starter = starter.cloned();
		if let Some(s) = starter {
			app.page = ApplicationPage::Scoresheet(s.id.clone())
		}
	}

	crate::templates::scoresheet::scoresheet(state).await
}

#[tauri::command]
pub async fn page_x_current(
	state: tauri::State<'_,ManagedApplicationState>,
	handle: tauri::AppHandle,
) -> ResponseDirector {
	let application_page = {
		let app_state = state.read()
			.map_err(|_|screen_error("Poisoned lock trying to access current page"))?;
		app_state.page.clone()
	};
	match application_page {
		ApplicationPage::Login=> crate::templates::login::login(state, handle).await,
		ApplicationPage::LoginJudge => crate::templates::choose_judge::choose_judge(state, handle).await,
		ApplicationPage::Welcome => crate::templates::welcome::welcome(state, handle).await,
		ApplicationPage::CompetitionList => crate::templates::competition_list::competition_list(state, handle, String::from("TODO")).await,
		ApplicationPage::Scoresheet(_) => crate::templates::scoresheet::scoresheet(state).await,
		ApplicationPage::Settings => crate::templates::settings::get_settings(state, handle).await,
		ApplicationPage::Preferences => crate::templates::preferences::get_preferences(state, handle).await,
		ApplicationPage::FinalResult => todo!(),
		ApplicationPage::Error => Err(screen_error("Unspecified Error")),
	}
}