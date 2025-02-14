pub mod show;
pub mod competition;
pub mod dressage_test;
pub mod ground_jury_member;
pub mod judge;
pub mod position;
pub mod starter;
pub mod competitor;
pub mod scoresheet;
pub mod penalties;
pub mod user;

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct SurrealId {
	tb: String,
	id: SurrealActualId,
}
impl std::fmt::Debug for SurrealId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let SurrealActualId::String(ref id) = self.id;
		write!(f, "{}:{}", &self.tb, id)
	}
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
enum SurrealActualId {
	String(String),
}

impl SurrealId {
	pub fn id(&self) -> String {
		let SurrealActualId::String(ref id) = self.id;
		id.to_string()
	}
	pub fn make(tb: &str, id: &str) -> Self {
		Self{tb: tb.to_string(), id: SurrealActualId::String(id.to_string())}
	}
}