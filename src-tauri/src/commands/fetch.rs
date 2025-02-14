use tauri::http::header::{AUTHORIZATION, CONTENT_TYPE};
use tauri_plugin_http::reqwest::{self, RequestBuilder};

use crate::state::ManagedApplicationState;

pub async fn fetch(method: Method, url: &str, state: tauri::State<'_, ManagedApplicationState>) -> RequestBuilder {
	method.to_fetch(reqwest::Client::new(), url)
		.header(CONTENT_TYPE, "Application/json")
		.header(AUTHORIZATION, format!("Bearer {}", state.read()
			.expect("Can read state")
			.token()))
		.header("Application-ID", "Victory/Client")
}

pub enum Method {
	Post,
	Get,
	Put,
}

impl Method {
	fn to_fetch(self, client: reqwest::Client, url: &str) -> RequestBuilder {
		match self {
			Method::Post => client.post(url),
			Method::Get => client.get(url),
			Method::Put => client.put(url),
		}
	}
}