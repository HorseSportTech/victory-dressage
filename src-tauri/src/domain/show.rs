use crate::{commands::fetch::{fetch, Method}, state::{ManagedApplicationState, UserType}, traits::{Entity, Fetchable, Storable}};

use super::{competition::Competition, SurrealId};

const API_URL: &'static str = dotenv_codegen::dotenv!("API_URL");

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Show {
	pub id: SurrealId,
	pub name: String,
	pub venue: String,
	#[serde(default)]
	pub competitions: Vec<Competition>,
}

impl crate::traits::Storable for Show{}
impl crate::traits::Entity for Show {
	fn key(&self) -> String {format!("{}:{}", self.id.tb, self.id.id())}
	fn id(&self) -> String {self.id.id()}
}
impl Fetchable for Show {
	async fn fetch(
		state: tauri::State<'_, ManagedApplicationState>,
	) -> Result<Vec<Self>, tauri_plugin_http::Error> {

		let judge_id = {
			match state.read().expect("Not poisoned").user {
				UserType::Judge(ref judge, _) => judge.id.id(),
				_ => return Err(tauri_plugin_http::Error::RequestCanceled),
			}
		};

		let shows = fetch(Method::Post, &format!("{API_URL}app/show"), state).await
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
		state: tauri::State<'_,ManagedApplicationState>,
		id: &str,
	) -> Result<Self, tauri_plugin_http::Error> {
	
		let show = fetch(Method::Get, &format!("{API_URL}app/show/{id}"), state).await
			.send()
			.await
			.inspect_err(|err| eprintln!("Response -> {err:?}"))?
			.error_for_status()
			.inspect_err(|err| eprintln!("Status -> {err:?}"))?
			.json::<Self>()
			.await
			.inspect_err(|err| eprintln!("Parse -> {err:?}"))?;
		// eprintln!("{:?}", &show[1190..1229]);
		Ok(show)
	}
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(transparent)]
pub struct Shows(pub Vec<Show>);
impl Storable for Shows {}
impl Entity for Shows {
	fn key(&self) -> String {String::from("shows")}
	fn id(&self) -> String {String::from("shows")}
}