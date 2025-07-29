use hypertext::{rsx, rsx_move, GlobalAttributes, Renderable};

use super::icons;
use crate::{
    commands::replace_director::{ReplaceDirector, ResponseDirector},
    state::ManagedApplicationState,
    templates::{html_elements, TxAttributes},
};

pub async fn get_settings(
    state: tauri::State<'_, ManagedApplicationState>,
    _handle: tauri::AppHandle,
) -> ResponseDirector {
    let freestyle_mode = state.read(|aps| aps.auto_freestyle)?;

    Ok(ReplaceDirector::page(
		rsx!{
			<main id="page--settings">
				<header style="inline-size: 100%">
					<h1 style="margin:0">Application Settings</h1>
					<button
						class="back-button"
						tx-goto="welcome"
					>{icons::BACK_ARROW}"Back"</button>
				</header>
				<section style="overflow-y:scroll">

					<div>
                        <div>"Automatically calculate the total for each movement in freestyles"</div>
                        <div id="freestyle-mode-btn">
                            {button_freestyle_mode(freestyle_mode)}
                        </div>

						<div style="margin-block:2rem 0.5rem">"Download a log file with all marks and comments from this current session."</div>
						<button
							class="settings-button"
							tx-command="download-file"
						>Download data file</button>

						<div style="margin-block:2rem 0.5rem">"Clear all stored data on the device ⚠︎"</div>
						<button
							class="settings-button"
							tx-command="clear-data"
							style="background:red; border-color:darkred"
						>Clear data</button>
					</div>

				</section>
			</main>
		}.render()
	))
}

pub fn button_freestyle_mode(auto: bool) -> hypertext::Lazy<impl Fn(&mut String)> {
    rsx_move! {
        <button class="settings-button" tx-command="toggle_freestyle_mode">@if auto {"Auto"} @else {"Traditional"}</button>
    }
}
