use hypertext::{html_elements, rsx, rsx_move, GlobalAttributes, Renderable};

use crate::{
    commands::replace_director::ResponseDirector, domain::judge::Judge,
    state::ManagedApplicationState, templates::choose_judge,
};

use super::{
    fetch::{fetch, Method},
    replace_director::ReplaceDirector,
};
const TARGET: &str = "#judge-list";

#[tauri::command]
pub async fn search_for_judge(
    state: tauri::State<'_, ManagedApplicationState>,
    value: &str,
) -> ResponseDirector {
    if let Ok(res) = fetch(Method::Get, &format!("{}judge", env!("API_URL")), state)
        .await
        .query(&[("term", value)])
        .send()
        .await
    {
        match res.error_for_status() {
            Ok(res) => match res.json::<Vec<Judge>>().await {
                Ok(judges) => {
                    return Ok(ReplaceDirector::with_target(
                        TARGET,
                        rsx_move! {{choose_judge::judge_list(judges.clone())}}.render(),
                    ))
                }
                Err(err) => eprintln!("{err:?}"),
            },
            Err(err) => eprintln!("{err:?}"),
        }
    }
    Ok(ReplaceDirector::with_target(
		TARGET,
		rsx!{
			<div style="background:red;color:white;border-radius:var(--corner-size)">No judges found</div>
		}.render()
	))
}

