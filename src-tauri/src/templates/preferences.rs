use hypertext::{rsx, rsx_dyn, GlobalAttributes, Renderable};

use super::icons;
use crate::{
    commands::{
        log_out,
        replace_director::{ReplaceDirector, ResponseDirector},
    },
    state::ManagedApplicationState,
    templates::{html_elements, TxAttributes},
};

use super::error::screen_error;

pub async fn get_preferences(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    let user = state
        .read()
        .or_else(|_| {
            state.clear_poison();
            state.read()
        })
        .map_err(|_| screen_error("Cannot access judge preferences due to poisoned lock"))?
        .user
        .clone();

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
					>{&icons::BACK_ARROW}" Back"</button>
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
							{if judge.prefs.comment_last {rsx_dyn!{
								<option value="false">"Comment first (most judges)"</option>
								<option value="true" selected>"Mark first (some judges)"</option>
							}} else {rsx_dyn!{
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
							{if judge.prefs.hide_trend {rsx_dyn!{
								<option value="false">"Shown (default)"</option>
								<option value="true" selected>Hidden</option>
							}} else {rsx_dyn!{
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
							{if judge.prefs.manually_sign {rsx_dyn!{
								<option value="false">"Automatically sign scoresheets"</option>
								<option value="true" selected>"Manually sign each sheet"</option>
							}} else {rsx_dyn!{
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
							>{icons::EDIT}"Change"</button>
						</div>
						<dialog id="signature-dialog">
                            <form method="dialog">
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
                                    style="border-color:var(--error);background: var(--error)"
                                    type="button"
									onclick="signature_refresh(event)"
								>{&icons::TRASH}"Clear"</button>
								<button
									class="btn"
									onclick="signature_refresh(event)"
								>{icons::CLOSE}"Cancel"</button>
								<div id="signature-dialog-message"></div>
								<button
									class="btn"
									style="margin-inline-start:auto"
									tx-command="save_signature"
								>{icons::TICK}"Ok"</button>
							</div>
                            </form>
						</dialog>
					</div>
				</section>
			</main>
		}.render()
	))
}

