use super::{error::screen_error, html_elements, TxAttributes};
use crate::{
    commands::{
        replace_director::{ReplaceDirector, ResponseDirector},
        PAGE_UPDATE,
    },
    domain::{competition::Competition, show::Show},
    state::ManagedApplicationState,
    traits::{Entity, Fetchable, Storable},
};
use hypertext::{rsx, rsx_move, GlobalAttributes, Lazy, Raw, Renderable};
use tauri::Emitter;

#[tauri::command]
pub async fn competition_list(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    id: String,
) -> ResponseDirector {
    {
        let (stored_competitions, show_name) = state
            .read()
            .map_err(|_| screen_error("Bad state - show missing"))?
            .show
            .as_ref()
            .and_then(|x| Some((x.competitions.clone(), x.name.clone())))
            .ok_or_else(|| screen_error("Bad state - show name missing"))?;
        let stored_competitions_template = render_list(stored_competitions);

        handle.emit(PAGE_UPDATE, ReplaceDirector::page(
			rsx!{
				<main id="page--competition-list">
				<header style="color:var(--theme); background:var(--background); border-block-end:0.2rem solid var(--theme)">
					<h1>{ &show_name }</h1>
					<button class="back-button" tx-goto="welcome">Back</button>
				</header>
				<section>
					<h2>"Competition List"<div class="spinner"></div></h2>
					<ul id="competition-list"><div class="loading">
					{ &stored_competitions_template }
					</div></ul>
				</section>
				</main>
			}.render()
		)).ok();
    }

    match Show::select(state.clone(), &id).await {
        Ok(show) => {
            state
                .write()
                .or_else(|_| {
                    state.clear_poison();
                    state.write()
                })
                .expect("Should be clear by now or we have a major problem")
                .show = Some(show.clone());
            show.clone().set(&handle).ok();

            Ok(ReplaceDirector::with_target(
                "#competition-list",
                render_list(show.competitions).render(),
            ))
        }
        Err(err) => Err(screen_error(err.to_string().as_str())),
    }
}

fn render_list(competitions: Vec<Competition>) -> Lazy<impl Fn(&mut std::string::String)> {
    rsx_move! {
        @for x in competitions.iter() {
            <li
                tx-goto="scoresheet"
                tx-id=x.get_id()
                style="color: var(--theme); background: var(--background); display: grid; grid: min-content min-content/ 1fr min-content; padding: 0.5rem 1rem;"
            >
                <h3
                    style="margin:0; overflow-x:hidden; white-space:nowrap; text-overflow:ellipsis;"
                >{ &x.name }</h3>
                <div style="grid-row: 2 / 3;color:var(--foreground);opacity:.8">
                    Starts at { format!(" {}", x.start_time.format("%H:%M")) }
                    @if let Some(ref arena) = x.arena {
                        {Raw("&nbsp;")}|{Raw("&nbsp;")}"Arena: "{&arena.name}
                    }
                </div>
                <div
                    style="grid-row:1/3; grid-column:2/3; border:0.2rem solid var(--theme);
					align-items:center; border-radius:0.7rem; justify-content:center; display:flex;
					block-size:3rem; align-self:center; inline-size:3rem;font-size:2rem; font-weight:500"
                >{x.jury.first().and_then(|j|Some(j.position.to_string())).unwrap_or_default()}</div>
            </li>
        }
    }
}
