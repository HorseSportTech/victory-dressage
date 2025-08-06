use super::{error::screen_error, html_elements, TxAttributes};
use crate::{
    commands::{
        replace_director::{PageLocation, ReplaceDirector, ResponseDirector},
        PAGE_UPDATE,
    },
    domain::{competition::Competition, show::Show},
    state::{store::Storable, ManagedApplicationState},
    traits::{Entity, Fetchable},
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
            .read_async(|app_state| {
                let show = app_state
                    .show
                    .as_ref()
                    .ok_or_else(|| screen_error("Bad state - show missing"))?;
                Ok((show.competitions.clone(), show.name.to_string()))
            })
            .await??;
        let stored_competitions_template = render_list(stored_competitions);

        handle.emit(PAGE_UPDATE, ReplaceDirector::page(
			rsx!{
				<main id="page--competition-list">
				<header style="color:var(--theme); background:var(--background); border-block-end:0.2rem solid var(--theme)">
					<h1 id="show-name">{ &show_name }</h1>
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

    match Show::select(&state, &id).await {
        Ok(show) => {
            let show2 = show.clone();
            state.write_async(move |a| a.show = Some(show2)).await?;
            show.store(&handle);

            Ok(ReplaceDirector::with_target(
                &PageLocation::CompetitionList,
                render_list(show.competitions).render(),
            ))
        }
        Err(err) => Err(screen_error(err.to_string().as_str())),
    }
}

pub fn render_list(competitions: Vec<Competition>) -> Lazy<impl Fn(&mut std::string::String)> {
    rsx_move! {
        @for x in competitions.iter() {
            {competition_listing(x)}
        }
    }
}
fn competition_listing<'a>(x: &'a Competition) -> Lazy<impl Fn(&mut String) + use<'a>> {
    rsx! {
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
            >{x.jury.first().map(|j|j.position.to_string()).unwrap_or_default()}</div>
        </li>
    }
}
