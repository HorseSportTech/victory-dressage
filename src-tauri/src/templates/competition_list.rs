use hypertext::{rsx, rsx_move, GlobalAttributes, RenderIterator, Renderable};
use tauri::Emitter;
use super::{error::screen_error, html_elements, TxAttributes};
use crate::{commands::{replace_director::{ReplaceDirector, ResponseDirector}, PAGE_UPDATE}, domain::{competition::Competition, show::Show}, state::ManagedApplicationState, traits::{Entity, Fetchable, Storable}};

#[tauri::command]
pub async fn competition_list(
	state: tauri::State<'_, ManagedApplicationState>,
	handle: tauri::AppHandle,
	id: String,
) -> ResponseDirector {
	{
		let (stored_competitions, show_name) = state.read()
			.map_err(|_| screen_error("Bad state - show missing"))?
			.show.as_ref().and_then(|x|{
				println!("{x:?}");
				Some((x.competitions.clone(), x.name.clone()))
			})
				.ok_or_else(||screen_error("Bad state - show name missing"))?;

	
		handle.emit(PAGE_UPDATE, ReplaceDirector::page(
			rsx!{
				<main id="page--competition-list">
				<header style="color:var(--theme); background:var(--background); border-block-end:0.2rem solid var(--theme)">
					<h1>{ show_name }</h1>
					<button class="back-button" tx-goto="welcome">Back</button>
				</header>
				<section>
					<h2>"Competition List"<div class="spinner"></div></h2>
					<ul id="competition-list"><div class="loading">
					{render_list(stored_competitions)}
					</div></ul>
				</section>
				<style>
					r#"
					#page--competition-list {
						display:grid;
						block-size:100lvh;
						inline-size:100lvw;
						overflow:clip;
						grid: 6rem 1fr / 1fr;
						color:var(--theme);
						& header {
							padding: 0 1rem;
							align-content:center;
							& h1 {
								overflow-x:hidden;
								white-space:nowrap;
								text-overflow:ellipsis;
								margin:0;
							}
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
							padding-inline:1rem;
							overflow-x:clip;
							overflow-y:scroll;
							& ul {
								padding-inline:1rem;
								gap: 0.2rem;
								display:flex;
								flex-direction:column;
							}
						}
					}
					"#
				</style>
				</main>
			}.render()
		)).ok();
	}


	match Show::select(state.clone(), &id).await {
		Ok(show) => {
			state.write()
				.or_else(|_| {
					state.clear_poison();
					state.write()
				}).expect("Should be clear by now or we have a major problem")
				.show = Some(show.clone());
			show.clone().set(&handle).ok();
			println!("{state:?}");
			Ok(
				ReplaceDirector::with_target(
					"#competition-list",
					render_list(show.competitions).render()
				)
			)
		}
		Err(err) => Err(screen_error(err.to_string().as_str())),
	}
}

fn render_list(competitions: Vec<Competition>) -> impl for<'a> FnOnce(&'a mut std::string::String) {
	competitions.into_iter().map(|x| rsx_move!{
		<li
			tx-goto="scoresheet"
			tx-id=x.get_id()
			style="color: var(--theme); background: var(--background); display: grid; grid: min-content min-content/ 1fr min-content; padding: 0.5rem 1rem;"
		>
			<h3 
				style="margin:0; overflow-x:hidden; white-space:nowrap; text-overflow:ellipsis;"
			>{ &x.name }</h3>
			<div style="grid-row: 2 / 3;">Starts at { format!(" {}", x.start_time.format("%H:%M")) }</div>
			<div
				style="grid-row:1/3; grid-column:2/3; border:0.2rem solid var(--theme);
				align-items:center; border-radius:0.7rem; justify-content:center; display:flex;
				block-size:3rem; align-self:center; inline-size:3rem;font-size:2rem; font-weight:500"
			>{x.jury.first().and_then(|j|Some(j.position.to_string())).unwrap_or_default()}</div>
		</li>
	}).render_all()
}