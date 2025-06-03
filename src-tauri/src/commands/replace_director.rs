use crate::commands::PAGE_UPDATE;
#[derive(serde::Serialize, Clone, Debug)]
pub struct ReplaceDirector {
    pub target: Option<String>,
    pub content: String,
    #[serde(skip_serializing_if = "std::ops::Not::not", rename = "outerHTML")]
    pub outer_html: bool,
}
impl ReplaceDirector {
    pub fn with_target(target: &str, content: hypertext::Rendered<String>) -> Self {
        Self {
            target: Some(target.to_string()),
            content: content.0,
            outer_html: false,
        }
    }
    pub fn page(content: hypertext::Rendered<String>) -> Self {
        Self {
            target: Some(String::from("#application")),
            content: content.0,
            outer_html: false,
        }
    }
    pub fn none() -> Self {
        Self {
            target: None,
            content: String::new(),
            outer_html: false,
        }
    }
    pub fn with_target_outer(target: &str, content: hypertext::Rendered<String>) -> Self {
        Self {
            target: Some(target.to_string()),
            content: content.0,
            outer_html: true,
        }
    }
}

pub type ResponseDirector = Result<ReplaceDirector, ReplaceDirector>;

pub fn emit_page(app: &tauri::AppHandle, name: &str, html: hypertext::Lazy<impl Fn(&mut String)>) {
    tauri::Emitter::emit(
        app,
        PAGE_UPDATE,
        ReplaceDirector::with_target(name, hypertext::Renderable::render(&html)),
    )
    .ok();
}
pub fn emit_page_outer(
    app: &tauri::AppHandle,
    name: &str,
    html: hypertext::Lazy<impl Fn(&mut String)>,
) {
    tauri::Emitter::emit(
        app,
        PAGE_UPDATE,
        ReplaceDirector::with_target_outer(name, hypertext::Renderable::render(&html)),
    )
    .ok();
}

pub fn emit_page_prerendered(
    app: &tauri::AppHandle,
    name: &str,
    html: hypertext::Rendered<String>,
) {
    tauri::Emitter::emit(app, PAGE_UPDATE, ReplaceDirector::with_target(name, html)).ok();
}
