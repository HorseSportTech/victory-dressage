use hypertext::{rsx, GlobalAttributes, Renderable};

use crate::{commands::{log_out, replace_director::{ReplaceDirector, ResponseDirector}}, state::ManagedApplicationState, templates::{TxAttributes, html_elements}};

use super::error::screen_error;

pub async fn get_preferences(
	state: tauri::State<'_, ManagedApplicationState>,
	handle: tauri::AppHandle,
) -> ResponseDirector {
	let user = state.read()
		.or_else(|_|{state.clear_poison();state.read()})
		.map_err(|_|screen_error("Cannot access judge preferences due to poisoned lock"))?
		.user.clone();

	let judge = match user {
		crate::state::UserType::Judge(judge, _token_user) => judge,
		_ => return log_out::log_out(state.clone(), handle).await,
	};

	Ok(ReplaceDirector::page(
		rsx!{
			<main id="page--preferences">
				<header style="inline-size: 100%">
					<h1 style="margin:0">{format!("Preferences for {} {}", judge.first_name, judge.last_name)}</h1>
					<button
						class="back-button"
						tx-goto="welcome"
					>"◂ Back"</button>
				</header>
				<section style="overflow-y:scroll">

					<h2>Scoresheet behavior settings</h2>
					<div class="settings-line">
						<label class="option-selector">
						<div class="label">Input order</div>
						<div class="selector-down-arrow">
						<select
							tx-command="update_comment_first"
							tx-trigger="change"
							value=judge.prefs.comment_last
						>
							{if judge.prefs.comment_last {rsx!{
								<option value="false">"Comment first (most judges)"</option>
								<option value="true" selected>"Mark first (some judges)"</option>
							}} else {rsx!{
								<option value="false" selected>"Comment first (most judges)"</option>
								<option value="true">"Mark first (some judges)"</option>
							}}}
						</select>
						</div>
						</label>

						<label class="option-selector">
						<div class="label">Display your trend</div>
						<div class="selector-down-arrow">
						<select
							tx-command="update_show_trend"
							tx-trigger="change"
							value=judge.prefs.hide_trend
						>
							{if judge.prefs.hide_trend {rsx!{
								<option value="false">"Shown (default)"</option>
								<option value="true" selected>Hidden</option>
							}} else {rsx!{
								<option value="false" selected>"Shown (default)"</option>
								<option value="true">Hidden</option>
							}}}
						</select>
						</div>
						</label>

						<label class="option-selector">
						<div class="label">Scoresheet signing</div>
						<div class="selector-down-arrow">
						<select
							tx-command="update_auto_sign"
							tx-trigger="change"
							value=judge.prefs.manually_sign
							disabled
						>
							{if judge.prefs.manually_sign {rsx!{
								<option value="false">"Automatically sign scoresheets"</option>
								<option value="true" selected>"Manually sign each sheet"</option>
							}} else {rsx!{
								<option value="false" selected>"Automatically sign scoresheets"</option>
								<option value="true">"Manually sign each sheet"</option>
							}}}
						</select>
						</div>
						</label>
					</div>

					<h2 style="margin:2.5rem 0 0.5rem">Signature</h2>
					<div class="signature-wrapper">
						<div class="signature">
							<svg viewBox="0 0 200 100" id="signature-image">
								<path
									d=judge.signature
									style="fill:none; stroke-width:2px; stroke: blue"
								></path>
							</svg>
						</div>
						<div style="text-align:center; flex:1 0 0%;">
							<button
								class="btn"
								onclick="document.querySelector('#signature-dialog').showModal()"
							>"Change"</button>
						</div>
						<dialog id="signature-dialog">
							<script src="/src/draw_signature.ts" defer></script>
							<h1 style="color:var(--theme)">"Please sign"</h1>
							<div class="box-signature" style="inline-size:100%;aspect-ratio:2 / 1;">
								<canvas
									style="width:100%; height: 100%"
									width="1000"
									height="500"
									onpointerdown="signature_startDraw(event)"
									onpointermove="signature_continueDraw(event)"
									onpointerup="signature_endDraw(event)"
									onpointerleave="signature_endDraw(event)"
								>
								</canvas>
							</div>
							<div style="display:flex; flex-direction:row; margin-block-start:0.5rem; gap:0.5rem">
								<button
									class="btn"
									onclick="signature_refresh(event)"
								>"Clear"</button>
								<button
									class="btn"
									onclick="signature_refresh(event);document.querySelector('#signature-dialog').close()"
								>"Cancel"</button>
								<div id="signature-dialog-message"></div>
								<button
									class="btn"
									style="margin-inline-start:auto"
									tx-command="save_signature"
								>"Ok"</button>
							</div>
						</dialog>
					</div>
				</section>
				<style>{hypertext::Raw(r#"
				#page--preferences {
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
					.back-button {
						font-size: var(--text-input);
						padding: 0.2rem 1rem;
						border-radius: var(--corner-size);
						border: 1px solid var(--theme);
						background: var(--theme);
						color: white;
					}
					& .btn {
						font-size: var(--text-input);
						padding: 0.5rem 1rem;
						border-radius: var(--corner-size);
						border: 1px solid var(--theme);
						background: var(--theme);
						color: white;
					}
					& section {
						padding-inline:2rem;
						overflow-x: clip;
					}
					& .settings-line {
						display:flex;
						flex-direction:row;gap:0.5rem;
						block-size:2rem;
					}
					& .option-selector {
						flex: 1 1 100%;
						display:flex;
						flex-direction:column;
						& .label {
							color:white;
						}
						& div {
							font-size:var(--text-info);
							box-sizing:border-box;
						}
					}
					& .signature-wrapper {
						display:flex;
					}
					& .signature {
						background:color-mix(in hsl, var(--theme) 45%, white 80%);
						border:0.1rem solid;
						block-size:14rem;
						aspect-ratio: 2 / 1;
					}
					& #signature-dialog {
						inline-size:calc(100% - 4rem);
						margin-inline:auto;
						border:0;
						border-radius:var(--corner-size);
						background:color-mix(in hsl, white 15%, var(--background));
					}
					& .box-signature {
						background: color-mix(in hsl, white 35%, var(--theme) 20%);
						position:relative;
						&::after {
							border: 1px solid var(--theme);
							border-width: 1px 0;
							position: absolute;
							content: '';
							width: 100%;
							height: 60%;
							top: 20%;
							left:0;
							pointer-events:none;
						}
					}
				}
				"#)}</style>
			</main>
		}.render()
	))
}