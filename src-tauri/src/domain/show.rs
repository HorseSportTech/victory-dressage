use crate::{
    commands::fetch::{fetch, Method},
    state::{ManagedApplicationState, UserType},
    traits::{Entity, Fetchable, Storable},
};

use super::{competition::Competition, SurrealId};

const API_URL: &str = env!("API_URL");

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Show {
    pub id: SurrealId,
    pub name: String,
    pub venue: String,
    #[serde(default)]
    pub competitions: Vec<Competition>,
}

impl crate::traits::Storable for Show {}
impl crate::traits::Entity for Show {
    fn key(&self) -> String {
        format!("{}:{}", self.id.tb, self.id.id())
    }
    fn get_id(&self) -> String {
        self.id.id()
    }
}
impl Fetchable for Show {
    async fn fetch(
        state: tauri::State<'_, ManagedApplicationState>,
    ) -> Result<Vec<Self>, tauri_plugin_http::Error> {
        let judge_id = {
            crate::state::ApplicationState::refresh(&state)
                .await
                .map_err(|_| tauri_plugin_http::Error::RequestCanceled)?;
            match state.read().expect("Not poisoned").user {
                UserType::Judge(ref judge, _) => judge.id.id(),
                _ => return Err(tauri_plugin_http::Error::RequestCanceled),
            }
        };

        let shows = fetch(Method::Post, &format!("{API_URL}show"), state)
            .await
            .body(format!("\"{judge_id}\""))
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Self>>()
            .await
            .inspect_err(|err| eprintln!("{err:?}"))?;

        Ok(shows)
    }
    async fn select(
        state: tauri::State<'_, ManagedApplicationState>,
        id: &str,
    ) -> Result<Self, tauri_plugin_http::Error> {
        crate::state::ApplicationState::refresh(&state)
            .await
            .map_err(|_| tauri_plugin_http::Error::RequestCanceled)?;

        let json_text = fetch(Method::Get, &format!("{API_URL}show/{id}"), state)
            .await
            .send()
            .await
            .inspect_err(|err| eprintln!("Response -> {err:?}"))?
            .error_for_status()
            .inspect_err(|err| eprintln!("Status -> {err:?}"))?
            .text()
            .await
            .inspect_err(|err| eprintln!("Text Content -> {err:?}"))?;
        let show = serde_json::from_str(&json_text)
            .inspect_err(|err| eprintln!("Parse -> {err:?} \n{json_text}"))?;
        // eprintln!("{:?}", &show[1190..1229]);
        Ok(show)
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(transparent)]
pub struct Shows(pub Vec<Show>);
impl Storable for Shows {}
impl Entity for Shows {
    fn key(&self) -> String {
        String::from("shows")
    }
    fn get_id(&self) -> String {
        String::from("shows")
    }
}

