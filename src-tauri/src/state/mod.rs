use application_page::ApplicationPage;
use jsonwebtoken::DecodingKey;
use tauri_plugin_http::reqwest;

pub mod application_page;

use crate::{domain::{competition::Competition, judge::Judge, scoresheet::Scoresheet, show::Show, starter::Starter, user::{IntitialUser, TokenClaims, User, UserRole}, SurrealId}, traits::{Entity, Storable}};
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApplicationState {
	pub user: UserType,
	pub show: Option<Show>,
	pub competition: Option<Competition>,
	pub starter: Option<Starter>,
	pub page: ApplicationPage,
	// pub battery: Battery,
	// pub socket: WebSocket,//??????
}
pub type ManagedApplicationState = std::sync::RwLock<ApplicationState>;


impl ApplicationState {
	pub fn new() -> Self {
		Self {
			user: UserType::NotAuthorised,
			show: None,
			competition: None,
			starter: None,
			page: ApplicationPage::Login,
		}
	}
	pub async fn restore() -> Self {
		todo!()
	}
	pub fn username(&self) -> String {
		match self.user {
			UserType::Judge(_, ref user) | UserType::Admin(ref user) => user.user.username.to_string(),
			_ => String::new(),
		}
	}
	pub fn token(&self) -> String {
		match self.user {
			UserType::Judge(_, ref user) | UserType::Admin(ref user) => user.token.to_string(),
			_ => String::new(),
		}
	}
	pub async fn authorise(&mut self, email: &str, password: &str) {
		let mut headers = reqwest::header::HeaderMap::new();
		headers.insert(reqwest::header::CONTENT_TYPE, "Application/json".parse().unwrap());
		let user = reqwest::Client::new()
			.post("http://server.victory-hst.au/public/login")
			.body(format!("{{\"email\":\"{email}\",\"password\":\"{password}\"}}"))
			.headers(headers)
			.send()
			.await.unwrap()
			.json::<InitialTokenUser>().await.unwrap();


		let token_user:TokenUser = user.try_into().expect("Can authorize");
		self.user = UserType::Admin(token_user);
	}

	pub fn scoresheet(&mut self) -> Option<&mut Scoresheet> {
		self.starter.as_mut()?.scoresheets
			.first_mut()
	}

	pub fn get_judge(&mut self) -> Option<&mut Judge> {
		match &mut self.user {
			UserType::Judge(judge, _) => Some(judge),
			_ => None,
		}
	}
}
impl Storable for ApplicationState{}
impl Entity for ApplicationState {
	fn key(&self) -> String {String::from("state")}
	fn id(&self) -> String {String::from("state")}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum UserType {
	Judge(Judge, TokenUser),
	Admin(TokenUser),
	NotAuthorised,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct TokenUser {
	pub token: String,
	pub user: User,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct InitialTokenUser {
	pub token: String,
	pub user: IntitialUser,
}
impl TryFrom<InitialTokenUser> for TokenUser {
	type Error = String;

	fn try_from(other: InitialTokenUser) -> Result<Self, Self::Error> {
		let Ok(claims) = decode_token(&other.token) else {
			return Err(String::from("Cannot decode"))
		};
		Ok(TokenUser {
			token: other.token,
			user: User {
				id: SurrealId::make("user", claims.claims.user_id.to_string().as_str()),
				username: other.user.username,
				email: other.user.email,
			}
		})
	}
}

const API_KEY: &'static str = dotenv_codegen::dotenv!("API_KEY");

#[derive(PartialEq)]
pub enum UserRoleTag {
	Judge,
	Admin,
	NotAuthorised,
}

impl TokenUser {
	// pub async fn into_usertype(self) -> UserType {
	// 	let Ok(data) = self.decode_token()
	// 	else {
	// 		return UserType::NotAuthorised
	// 	};
	// 	if self.user.username != data.claims.username {
	// 		return UserType::NotAuthorised
	// 	}
	
	// 	match data.claims.role {
	// 		UserRole::Official => {

	// 		},
	// 		UserRole::Scorer | UserRole::ShowOffice | UserRole::Admin => UserType::Admin(self),
	// 		_ => UserType::NotAuthorised,
	// 	}
	// }
	pub fn get_role_for_user(self) -> UserRoleTag {
		let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512);
		let Ok(data) = jsonwebtoken::decode::<TokenClaims>(
			&self.token,
			&DecodingKey::from_secret(API_KEY.as_bytes()),
			&validation
		)
		else {
			return UserRoleTag::NotAuthorised
		};
		if self.user.username != data.claims.username {
			return UserRoleTag::NotAuthorised
		}

		match data.claims.role {
			UserRole::Official => UserRoleTag::Judge,
			UserRole::Scorer | UserRole::ShowOffice | UserRole::Admin => UserRoleTag::Admin,
			_ => UserRoleTag::NotAuthorised,
		}
	}
}


pub fn decode_token(token: &str) -> Result<jsonwebtoken::TokenData<TokenClaims>, jsonwebtoken::errors::Error> {
	let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512);
	jsonwebtoken::decode::<TokenClaims>(
		&token,
		&DecodingKey::from_secret(API_KEY.as_bytes()),
		&validation
	)
}