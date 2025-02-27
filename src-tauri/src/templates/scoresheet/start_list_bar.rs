use hypertext::{rsx, rsx_move, GlobalAttributes, Raw, RenderIterator, Renderable};

use super::super::{html_elements, TxAttributes};
use crate::{
    domain::{
        ground_jury_member::{GroundJuryMember, JuryAuthority},
        show::Show,
        starter::Starter,
        SurrealId,
    },
    traits::Entity,
};

pub fn start_list_bar<'b>(
    show: &'b Show,
    start_list: &'b Vec<Starter>,
    judge: &'b GroundJuryMember,
    current_starter: &'b SurrealId,
) -> impl for<'a> FnOnce(&'a mut std::string::String) + use<'b> {
    let finished_starters: Vec<&Starter> = start_list
        .iter()
        .filter(|x| x.status.is_finished())
        .collect();
    let upcoming_starters: Vec<&Starter> = start_list
        .iter()
        .filter(|x| !x.status.is_finished())
        .collect();
    let separator = finished_starters.len() > 0 && upcoming_starters.len() > 0;
    let show_final = match upcoming_starters.len() == 0 {
        true if judge.authority == JuryAuthority::Chief => FinalButton::ActiveFinalize,
        true => FinalButton::ActiveView,
        false if judge.authority == JuryAuthority::Chief => FinalButton::DisabledFinalize,
        false => FinalButton::DisabledView,
    };
    let finished = list(finished_starters, true, current_starter);
    let to_come = list(upcoming_starters, false, current_starter);
    rsx_move! {<aside>
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
            class="start-list"
            onclick="event.target==this && this.close()"
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
                            tx-id=show.get_id()
                            style="background:var(--theme);
							border:1px solid color-mix(in srgb, var(--theme) 95%, black);
							font-size:var(--text-info); border-radius:var(--corner-size);
							flex: 1 1 100%; color:white;"
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
                <div style="padding:calc(2 * var(--padding)); font-size:var(--text" class="starters-list">
                    <ul style="margin:0; padding:0">
                    {finished}
                    </ul>
                    {if separator {
                    Some(rsx!{<hr style="border: 0.1rem solid var(--theme);margin: 0.5rem 0;"/>})
                    } else {None}}
                    <ul style="margin:0; padding:0">
                    {to_come}
                    </ul>
                    {show_final.to_string()}
                </div>
            </div>
            <style>{Raw(r#"dialog.start-list {
			
			& .starters-list {
				& li {padding-block:0.1rem}
				& li:has(.done) {background-color: color-mix(in hsl, var(--theme), transparent 85%);}
				& li + li {
					border-top: 0 !important;
				}
				& button {
					margin: 0;
					block-size: min-content;
					background: transparent;
					box-sizing: border-box;
					border: 0;
					padding: 0 0.1rem 0 0;
					inline-size:100%;
					display:block
				}
				& .starter-select {
					display:grid;
					column-gap:0.2rem;
					inline-size:100%;
					grid:auto auto / 1rem 1fr max-content;
					font-size:var(--text-info);
					align-items:center;
				}
				& .done-icon {
					grid-area:1/1 / 3/2;
					writing-mode:vertical-lr;
					min-height:2rem;
					font-weight: 700;
					align-content: center;
					border-radius: var(--corner-size);
					padding-inline: 0.1rem;
					font-size:0.6rem;
					&.done {
						background: forestgreen;
						color: white;
					}
				}
				& button.finalize {
					background:var(--theme);
					color:white;
					border:0;
					border-radius:var(--corner-size);
					padding:0.5rem;
					margin-block-start:0.5rem;
					&:disabled {
						background:grey;
					}
				}
				li.selected {
					background-image: linear-gradient(90deg, lightyellow 10%, transparent 50%);
				}
			}
		}"#)}
            </style>
        </dialog>
    </aside>}
}

fn list<'b>(
    starters: Vec<&'b Starter>,
    finished: bool,
    current_starter: &'b SurrealId,
) -> impl for<'a> FnOnce(&'a mut std::string::String) + use<'b> {
    starters.into_iter().map(move |x| rsx_move!{
		<li
			class=format!("{}", if x.id == *current_starter {"selected"} else {""})
			style=format!("padding:0.1rem; margin:0; display:block;border:1px solid;{}",
				if finished {"border-color:var(--theme);"} else {"border-color:gainsboro;"}
			)>
			<button tx-command="choose_starter" tx-id=x.get_id()>
				<div class="starter-select">
					{Raw(if x.status.is_finished() {
						rsx_move!{<div class="done-icon done">{format!("{}", x.status.list_abbreviation())}</div>}.render()
					} else {
						rsx_move!{<div class="done-icon">{format!("{}", x.status.list_abbreviation())}</div>}.render()
					})}
					<div style="text-align:left;text-overflow:ellipsis; white-space:nowrap; overflow:clip">{format!("{} {}", x.competitor.first_name, x.competitor.last_name)}</div>
					<div style="text-align:left;text-overflow:ellipsis; white-space:nowrap; overflow:clip">{x.score_or_number()}</div>
					<div style="text-align:left;text-overflow:ellipsis; white-space:nowrap; overflow:clip">{&x.competitor.horse_name}</div>
					<div style="text-align:left;text-overflow:ellipsis; white-space:nowrap; overflow:clip; text-align:right">{x.time_or_rank()}</div>
				</div>
			</button>
		</li>
	}).render_all()
}

enum FinalButton {
    ActiveFinalize,
    ActiveView,
    DisabledFinalize,
    DisabledView,
}

impl FinalButton {
    fn to_string<'b>(&'b self) -> impl for<'a> FnOnce(&'a mut std::string::String) + use<'b> {
        let res = match self {
            Self::ActiveFinalize | Self::DisabledFinalize => "Finalise competition result",
            Self::ActiveView | Self::DisabledView => "See competition result",
        };

        return rsx_move! {{Raw(
            match self {
                Self::ActiveFinalize|Self::ActiveView =>rsx!{
                    <button class="finalize">{res}</button>
                }.render(),
                _ => rsx!{
                    <button class="finalize" disabled>{res}</button>
                }.render()
            }
        )}};
    }
}

