use hypertext::{rsx_move, GlobalAttributes};

use crate::{domain::{dressage_test::DressageTest, scoresheet::Scoresheet, starter::Starter}, templates::scoresheet::{artistic_row, errors_row, status_selection, technical_row}};
use super::super::{TxAttributes, html_elements};

pub fn warnings_bar<'b>(
	test: &'b DressageTest,
	starter: &'b Starter,
	scoresheet: &'b Scoresheet,
) -> impl for<'a> FnOnce(&'a mut std::string::String,) + use<'b> {
rsx_move!{<aside>
	<button
		tx-open="#warnings-menu"
		style="background:var(--theme); position:fixed; left:0.5rem; bottom:0.5rem;
			block-size: 2rem; inline-size: 2rem;
			border:1px solid color-mix(in srgb, var(--theme) 92%, black);
			list-style: none; border-radius: 50%; align-content:center"
		>
		<svg
			style="margin:0; fill:white; fill-rule:evenodd; width:95%; height:95%"
			viewBox="0 0 36 36"
			preserveAspectRatio="xMinYMid meet"
		>
			<path d="M18,1 L36,35 H0 L18,1 M18,6 L31,32 H5L18,6z M16,31H20V27H16z M16,25H20L21,11H15z"></path>
		</svg>
	</button>
	<dialog
		style="position:fixed;box-sizing:border-box; width: 40vw; margin-top: 5rem; margin-left:0;
		background:var(--background); height: calc(100vh - 7rem);
		border: none; outline:none; padding:var(--padding)"
		id="warnings-menu"
		onclick="event.target==this && this.close()"
	>
	<div style="block-size:100%; background:var(--foreground); padding:0.5rem; box-sizing:border-box">
	<fieldset style="border:1px solid var(--background); border-radius:var(--corner-size)">
		<legend>"Judges’ Signalling System"</legend>
		<div 
			class="dialog-header"
			style="display:grid; inline-size:100%; aspect-ratio: 5/3; grid: 1fr 1fr/1fr 1fr; gap:0.1rem"
		>
			<button id="button-blood" tx-command="toggle_blood">"Blood"</button>
			<button id="button-lameness" tx-command="toggle_lameness">"Lameness"</button>
			<button id="button-equipement" tx-command="toggle_equipement">"Tack"</button>
			<button id="button-meeting" tx-command="toggle_meeting">"Meeting"</button>
		</div>
		</fieldset>

		<fieldset style="border:1px solid var(--background)">
		<legend>"Status"</legend>
		<div>
			<label for="competitor-status">"Competitor Status"</label>
			<div class="selector-down-arrow" id="status-selector">
				{status_selection(&starter.status)}
			</div>
		</div>
		</fieldset>

		<fieldset style="border:1px solid var(--background)">
		<legend>"Penalties"</legend>
		<div
			style="color:var(--theme); gap:0.5rem; flex-direction:column; display:flex;"
		>
			<div
				style="display:flex; flex-direction:row; flex-wrap:wrap"
				id="penalties-errors">
				{errors_row(test.errors_of_course.len() > 0, scoresheet.errors)}
			</div>
			<div
				style="display:flex; flex-direction:row; flex-wrap:wrap"
				id="penalties-technical">
				{technical_row(test.technical_penalties.len() > 0, scoresheet.tech_penalties)}
			</div>
			<div style="display:flex; flex-direction:row; flex-wrap:wrap"
				id="penalties-artistic">
				{artistic_row(test.artistic_penalties.len() > 0, scoresheet.art_penalties)}
			</div>
		</div>
		</fieldset>
	</div>
	</dialog>
</aside>}
}