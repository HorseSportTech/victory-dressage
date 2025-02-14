#[derive(serde::Serialize, Clone, Debug)]
pub struct ReplaceDirector {
	pub target: Option<String>,
	pub content: String,
}
impl ReplaceDirector {
	pub fn with_target(target: &str, content: hypertext::Rendered<String>) -> Self {
		Self{target:Some(target.to_string()), content: content.0}
	}
	pub fn page(content: hypertext::Rendered<String>) -> Self {
		Self {target:Some(String::from("#application")), content:content.0}
	}
	pub fn none() -> Self {
		Self{target:None, content: String::new()}
	}
}

pub type ResponseDirector = Result<ReplaceDirector, ReplaceDirector>;