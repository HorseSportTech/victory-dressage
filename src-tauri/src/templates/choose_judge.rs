use super::html_elements;
use super::welcome;
use crate::commands::replace_director::{ReplaceDirector, ResponseDirector};
use crate::domain::judge::Judge;
use crate::state::ManagedApplicationState;
use crate::state::UserTypeOnly;
use crate::templates::logout::logout_button;
use hypertext::{rsx, GlobalAttributes, Renderable as _};
use hypertext::{rsx_move, Lazy};

use super::error::screen_error;
use super::TxAttributes;
pub async fn choose_judge(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    match state.read(|app_state| (&app_state.user).into())? {
        UserTypeOnly::Admin => (),
        UserTypeOnly::Judge => return welcome::welcome(state.clone(), handle).await,
        _ => return Err(screen_error("Not authorised")),
    };

    Ok(ReplaceDirector::page(rsx! {
		<div
			style="display:flex; align-items:center;
				justify-content:center; block-size:100vh;
				inline-size:100vw;"
		>
			{hypertext::Raw(logout_button())}
			<div
				style="text-align:center; background:var(--background); border-radius:0.5rem; color:var(--theme);
					padding:1rem 1rem"
			>
				<h2>Judge Selection</h2>
				<input type="search"
					tx-command="search_for_judge"
					tx-trigger="input"
					style="font-size: var(--text-input); padding: 0.2rem; appearance: none; border: 0; border-radius: var(--corner-size); outline: none;"
				>
				<ul
					id="judge-list"
					style="list-style:none; margin-inline-end:-0.4rem; padding:0; height:6rem; overflow-y:scroll"
				></ul>
			</div>
		</div>
	}.render()))
}

pub fn judge_list(judges: Vec<Judge>) -> Lazy<impl Fn(&mut String)> {
    rsx_move! {
        @for x in judges.iter(){
            <li style="margin-block-end:0.1rem">
                <button
                    tx-command="login_judge"
                    tx-id=x.user.as_ref().map(|u|u.id.id())
                    style="width:100%;border:0; background:var(--theme); color:white"
                >
                    <div style="text-transform:uppercase; font-style:bold; font-size:var(--text-info)">{format!("{} {}", x.first_name, x.last_name)}</div>
                    <div style="opacity: 0.6; font-size:0.6rem">{x.user.as_ref().map(|j|j.email.to_string())}</div>
                </button>
            </li>
        }
    }
}
