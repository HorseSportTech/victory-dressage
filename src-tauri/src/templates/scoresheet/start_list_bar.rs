use hypertext::{rsx, rsx_move, GlobalAttributes, Raw, RenderIterator};

use super::super::{html_elements, TxAttributes};
use crate::{domain::{show::Show, starter::Starter}, traits::Entity};

pub fn start_list_bar<'b>(show: &'b Show, start_list: &'b Vec<Starter>) -> impl for<'a> FnOnce(&'a mut std::string::String,) + use<'b> {
rsx!{<aside>
	<button
		tx-open="#startlist-menu"
		style="background:var(--theme); position:fixed; right:0.5rem; bottom:0.5rem;
			block-size: 2rem; inline-size: 2rem;
			border:1px solid color-mix(in srgb, var(--theme) 92%, black);
			list-style: none; border-radius: 50%; align-content:center; padding:0.3rem;"
		>
		<svg viewBox="0 0 10 10" style="margin:0.35rem; stroke-width:2; stroke: white">
			<path d="M0,1H10M0,5H10M0,9H10"></path>
		</svg></button>
	<dialog
		style="position:fixed;box-sizing:border-box; width: 40vw; margin-top: 5rem; margin-right:0;
		background:var(--background); height: calc(100vh - 7rem);
		border: none; outline:none; padding:var(--padding)"
		id="startlist-menu"
	>
		<div 
			class="dialog-header"
			style="background:var(--foreground); border-radius:var(--corner-size);
				block-size: 100%; display:grid;
				grid: min-content 1fr / 1fr; box-sizing: border-box;"
		>
			<div 
				style="box-sizing: border-box; border-block-end: 0.2rem solid var(--theme);
					padding:var(--padding); box-shadow: 0 0.2rem 0.2rem #0003">
				<div
					style="display:flex;flex-direction:row; gap:var(--padding); inline-size: 100%;
					box-sizing:border-box; padding-inline:var(--padding); block-size: 1.9rem"
				>
					<button
						onclick="document.getElementById('startlist-menu').close()"
						style="border-radius:var(--corner-size); background: red; color:white; border:none;
							border:1px solid color-mix(in srgb, red 95%, black);
							outline:none;font-size:2rem;line-height:1.3rem; align-content:center;
							font-weight: 300; display:flex; flex: 0 0 auto; box-sizing: border-box;"
					>{Raw("&times;")}</button>
					<button
						tx-goto="competition_list"
						tx-id=show.id()
						style="background:var(--theme);
							border:1px solid color-mix(in srgb, var(--theme) 95%, black);
							font-size:var(--text-info); border-radius:var(--corner-size);
							flex: 1 1 100%; color:white"
					>"Return to competitions"</button>
				</div>
				<div style="display:flex; width:100%; padding:var(--padding);box-sizing: border-box;">
					<input
						type="search"
						placeholder="Search ID, horse name, athlete name"
						style="flex: 1 1 100%; border-radius:var(--corner-size); box-sizing: border-box;
							outline: none; border: 1px solid var(--theme); font-size: 0.8rem;
							margin-block-start: .8rem; padding:var(--padding);
							min-height:1.8rem"
					>
				</div>
			</div>
			<div style="padding:calc(2 * var(--padding))">
			{list(start_list)}
			</div>
		</div>

	</dialog>
</aside>}
}

fn list<'b>(starters: &'b Vec<Starter>) -> impl for<'a> FnOnce(&'a mut std::string::String,) + use<'b> {
	starters.into_iter().map(|x| rsx_move!{
		<ul>
			<button tx-command="choose-starter" tx-id=x.id()>
				<div style="display:grid; inline-size:100%;grid:1fr max-content / auto auto">
					<div>{format!("{} {}", x.competitor.first_name, x.competitor.last_name)}</div>
					<div>{format!("{:.3}", x.score.unwrap_or_default())}</div>
					<div>{&x.competitor.horse_name}</div>
					<div>{format!("{}", x.start_time)}</div>
				</div>
			</button>
		</ul>
	}).render_all()
}