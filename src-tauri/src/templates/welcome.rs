use super::TxAttributes;
use super::{error::screen_error, html_elements};
use crate::{
    commands::{
        replace_director::{ReplaceDirector, ResponseDirector},
        PAGE_UPDATE,
    },
    domain::show::{Show, Shows},
    state::{ManagedApplicationState, UserType},
    templates::logout::logout_button,
    traits::{Entity, Fetchable, Storable},
};
use hypertext::{rsx, rsx_move, GlobalAttributes, Renderable};
use tauri::Emitter;

pub async fn welcome(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    {
        let user = &state.read().expect("Not poisoned");
        let UserType::Judge(ref judge, _) = user.user else {
            return Err(screen_error(
                "Incorrect authorization. You must be a judge!",
            ));
        };

        let Shows(stored_shows) = Shows::get(&handle, "shows").unwrap_or_else(|_| Shows(vec![]));

        handle.emit(PAGE_UPDATE, ReplaceDirector::page(
			rsx!{
				<main id="page--welcome">
					<header class="header">
						{hypertext::Raw(logout_button())}
						<h1 style="margin:1rem 0 0;">{ format!("Welcome, {} {}", &judge.first_name, &judge.last_name) }</h1>
					</header>
					<section>
						<div style="flex: 1 1 100%; inline-size: 100%; display:flex; flex-direction:row;
							justify-content:end; gap:0.5rem; margin-block:0.2rem;">
							<button class="btn" tx-goto="preferences">Preferences</button>
							<button class="btn" tx-goto="settings">Settings</button>
						</div>
						<h2 style="margin: 0 0 0.5rem">"Upcoming shows"<div class="spinner"></div></h2>
						<ul id="show-list">
							<div class="loading">
								{show_list(stored_shows.clone())}
							</div>
						</ul>
					</section>
				</main>
			}.render(),
		))
		.inspect_err(|err| eprintln!("{err:?}")).ok();
    }

    match Show::fetch(state).await {
        Ok(shows) => {
            Shows(shows.clone()).set(&handle).ok();
            Ok(ReplaceDirector::with_target(
                "#show-list",
                show_list(shows).render(),
            ))
        }
        Err(err) => {
            eprintln!("{err:?}");
            Ok(ReplaceDirector::none())
        }
    }
}

fn show_list(shows: Vec<Show>) -> hypertext::Lazy<impl Fn(&mut String)> {
    rsx_move! {
        @for (x, _) in shows.iter().zip(1..) {
            <li
                tx-goto="competition_list"
                tx-id=x.get_id()
                style="background:var(--background)"
            >
                <div style="color:white">{ &x.name }</div>
                <div style="color:silver">{ &x.venue }</div>
            </li>
        }
    }
}

