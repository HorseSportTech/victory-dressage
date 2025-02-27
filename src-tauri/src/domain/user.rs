use super::SurrealId;


#[derive(serde::Deserialize, Clone, Debug)]
pub struct IntitialUser {
	pub username: String,
	pub email: String,
	#[serde(default)]
	pub refresh_token: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct User {
	pub id: SurrealId,
	pub username: String,
	pub email: String,
	#[serde(default)]
	pub refresh_token: Option<String>,
}



#[derive(serde::Deserialize)]
pub struct TokenClaims {
	pub user_id: ulid::Ulid,
	pub role: UserRole,
	pub username: String,
}

#[derive(serde::Deserialize)]
pub enum UserRole {
	Admin,
    Official,
    User,
    Scorer,
    ShowOffice,
}

impl crate::traits::Storable for User{}
impl crate::traits::Entity for User {
	fn key(&self) -> String {format!("{}:{}", self.id.tb, self.id.id())}
	fn get_id(&self) -> String {self.id.id()}
}