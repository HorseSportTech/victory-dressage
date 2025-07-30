use hypertext::{html_elements, rsx, rsx_move, GlobalAttributes, Renderable};

use crate::{
    commands::replace_director::ResponseDirector, domain::judge::Judge,
    state::ManagedApplicationState, templates::choose_judge,
};

use super::{
    fetch::{fetch, Method},
    replace_director::{PageLocation, ReplaceDirector},
};
const TARGET: &PageLocation = &PageLocation::JudgeList;

#[tauri::command]
pub async fn search_for_judge(
    state: tauri::State<'_, ManagedApplicationState>,
    value: &str,
) -> ResponseDirector {
    async {
        let res = fetch(Method::Get, concat!(env!("API_URL"), "judge"), &state)
            .query(&[("term", value)])
            .send()
            .await?
            .error_for_status()?;

        let judges: Vec<Judge> = res.json().await?;

        Ok(ReplaceDirector::with_target(
            TARGET,
            rsx_move! {{choose_judge::judge_list(judges.clone())}}.render(),
        ))
    }
    .await
    .or_else(|_: tauri_plugin_http::reqwest::Error| {
        Ok(ReplaceDirector::with_target(
            TARGET,
            rsx! {<div
                    style="background:red;color:white;border-radius:var(--corner-size)"
                >"No judges found"</div>
            }
            .render(),
        ))
    })
}
