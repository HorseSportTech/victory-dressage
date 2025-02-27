use hypertext::{html_elements, rsx, GlobalAttributes, Renderable};

use crate::{commands::replace_director::ResponseDirector, state::ManagedApplicationState, templates::choose_judge};

use super::{fetch::{fetch, Method}, replace_director::ReplaceDirector};


#[tauri::command]
pub async fn search_for_judge(
	state: tauri::State<'_, ManagedApplicationState>,
	value: &str,
) -> ResponseDirector {
	let target:&'static str = "#judge-list";
	if let Ok(res) = fetch(Method::Get, &format!("{}judge", dotenv_codegen::dotenv!("API_URL")), state).await
		.query(&[("term", value)])
		.send()
		.await {
		match res.error_for_status() {
			Ok(res) => match res.json().await {
				Ok(judges) => return Ok(ReplaceDirector::with_target(
					target,
					rsx!{{choose_judge::judge_list(judges)}}.render()
				)),
				Err(err) => eprintln!("{err:?}")
			}
			Err(err) => eprintln!("{err:?}")
		}
	}	
	Ok(ReplaceDirector::with_target(
		target,
		rsx!{
			<div style="background:red;color:white;border-radius:var(--corner-size)">No judges found</div>
		}.render()
	))

}