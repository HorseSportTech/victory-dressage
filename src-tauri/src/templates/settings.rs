use hypertext::{rsx, GlobalAttributes, Renderable};

use crate::{commands::replace_director::{ReplaceDirector, ResponseDirector}, state::ManagedApplicationState, templates::{TxAttributes, html_elements}};


pub async fn get_settings(
	_state: tauri::State<'_, ManagedApplicationState>,
	_handle: tauri::AppHandle,
) -> ResponseDirector {

	Ok(ReplaceDirector::page(
		rsx!{
			<main id="page--settings">
				<header style="inline-size: 100%">
					<h1 style="margin:0">Application Settings</h1>
					<button
						class="back-button"
						tx-goto="welcome"
					>"◂ Back"</button>
				</header>
				<section style="overflow-y:scroll">

					<div>
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
				<style>{hypertext::Raw(r#"
				#page--settings {
					color:var(--theme);
					inline-size: 100lvw;
					block-size:100lvh;
					overflow:hidden;
					display:grid;
					grid:min-content 1fr / 1fr;

					& header {
						background:var(--background);
						border-block-end:0.2rem solid var(--theme);
						padding-inline: 1rem;
						block-size:6rem;
						align-content:center;
					}
					& .settings-button {
						font-size: var(--text-input);
						padding: 0.5rem 1rem;
						border-radius: var(--corner-size);
						border: 1px solid var(--theme);
						background: var(--theme);
						color: white;
					}
					& .back-button {
						font-size: var(--text-input);
						padding: 0.2rem 1rem;
						border-radius: var(--corner-size);
						border: 1px solid var(--theme);
						background: var(--theme);
						color: white;
					}
					& section {
						padding-inline:2rem;
						overflow-x: clip;
					}
				}
				"#)}</style>
			</main>
		}.render()
	))
}