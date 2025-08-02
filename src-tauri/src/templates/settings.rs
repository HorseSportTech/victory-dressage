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
					>{icons::BACK_ARROW}" Back"</button>
				</header>
				<section style="padding-top:var(--padding);overflow-y:scroll">

					<div>
                        <div style="margin-block:2rem 0.5rem">"Automatically calculate the total for each movement in freestyles"</div>
                        <div id="freestyle-mode-btn" style="display:flex;gap:0.5rem">
                            {button_freestyle_mode(freestyle_mode)}
                        </div>

						<div style="margin-block:2rem 0.5rem">"Download a log file with all marks and comments from this current session."</div>
						<button
							class="settings-button"
							tx-command="download_file"
						>"Download data file"</button>

						<div style="margin-block:2rem 0.5rem">"Clear all stored data on the device ⚠︎"</div>
                        <div id="clear-data-button" style="display:contents">
                            {clear_data_button(false)}
                            <dialog
                                id="clear-data-button--confirm"
                                style="border-radius:var(--padding); border:1px solid var(--theme); background:var(--background)"
                            >
                                <form method="dialog">
                                    <div style="color:var(--theme);margin-block:1rem 2rem">"Are you sure you want to clear all data?"</div>
                                    <div style="display:flex;gap:0.5rem;justify-content:end">
                                        <button class="settings-button open">"Do not clear"</button>
                                        <button class="settings-button" tx-command="clear_data">"Clear"</button>
                                    </div>
                                </form>
                            </dialog>
                        </div>
					</div>

				</section>
			</main>
		}.render()
	))
}

pub fn button_freestyle_mode(auto: bool) -> hypertext::Lazy<impl Fn(&mut String)> {
    rsx_move! {
        @if auto {
            <button class="settings-button" disabled tx-command="toggle_freestyle_mode">" Auto✓"</button>
            <button class="settings-button open" tx-command="toggle_freestyle_mode">" Traditional "</button>
        } @else {
            <button class="settings-button open" tx-command="toggle_freestyle_mode">" Auto "</button>
            <button class="settings-button" disabled tx-command="toggle_freestyle_mode">" Traditional✓"</button>
        }
    }
}

pub fn clear_data_button(clear: bool) -> hypertext::Lazy<impl Fn(&mut String)> {
    rsx_move! {
        <button
            class="settings-button"
            onclick="document.querySelector('#clear-data-button--confirm')?.showModal()"
            style="background:red; border-color:darkred"
        >@if clear {"Cleared"} @else {"Clear data"}</button>
    }
}
