use crate::commands::PAGE_UPDATE;
use crate::debug;
#[derive(serde::Serialize, Clone, Debug)]
pub struct ReplaceDirector {
    pub target: Option<String>,
    pub content: String,
    #[serde(skip_serializing_if = "std::ops::Not::not", rename = "outerHTML")]
    pub outer_html: bool,
}
impl ReplaceDirector {
    pub fn with_target(target: &PageLocation, content: hypertext::Rendered<String>) -> Self {
        Self {
            target: target.shref(),
            content: content.0,
            outer_html: false,
        }
    }
    pub fn page(content: hypertext::Rendered<String>) -> Self {
        Self {
            target: PageLocation::Application.shref(),
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
    pub fn with_target_outer(target: &PageLocation, content: hypertext::Rendered<String>) -> Self {
        Self {
            target: target.shref(),
            content: content.0,
            outer_html: true,
        }
    }
}

pub type ResponseDirector = Result<ReplaceDirector, ReplaceDirector>;

pub fn emit_page(
    app: &tauri::AppHandle,
    name: &PageLocation,
    html: hypertext::Lazy<impl Fn(&mut String)>,
) {
    tauri::Emitter::emit(
        app,
        PAGE_UPDATE,
        ReplaceDirector::with_target(name, hypertext::Renderable::render(&html)),
    )
    .ok();
}
pub fn emit_page_outer(
    app: &tauri::AppHandle,
    name: &PageLocation,
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
    name: &PageLocation,
    html: hypertext::Rendered<String>,
) {
    tauri::Emitter::emit(app, PAGE_UPDATE, ReplaceDirector::with_target(name, html)).ok();
}

#[derive(serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PageLocation {
    HeaderTrend,
    TotalScore,
    ConfirmMarks,
    WarningsMenu,
    StartlistMenu,
    StartersList,
    ShowList,
    CompetitionList,
    ButtonLameness,
    ButtonBlood,
    ButtonEquipment,
    ButtonMeeting,
    Scoresheet,
    PenaltiesErrors,
    PenaltiesTechnical,
    PenaltiesArtistic,
    FinalRemark,
    AlertsAndWarnings,
    LoginButton,
    StatusSelector,
    TestTimeCountdown,
    MusicCountdown,
    NormalCountdown,
    EmailButton,
    Application,
    SignatureImage,
    SignatureDialogMessage,
    EmailLabel,
    PasswordLabel,
    FreestyleModeBtn,
    JudgeList,
    Any(String),
}
impl PageLocation {
    pub fn href(&self) -> String {
        match self {
            Self::Any(value) => value.to_string(),
            _ => {
                let mut str = serde_json::to_string(self).unwrap();
                debug_assert!(str.starts_with('"') && str.ends_with('"'));

                // Change first byte
                unsafe {
                    let bytes = str.as_bytes_mut();
                    bytes[0] = b'#';
                }

                str.truncate(str.len() - 1);
                str
            }
        }
    }
    pub fn shref(&self) -> Option<String> {
        Some(self.href())
    }
    pub fn id(&self) -> String {
        match self {
            Self::Any(_) => unimplemented!(),
            _ => {
                let mut str = serde_json::to_string(self).unwrap();
                debug_assert!(str.starts_with('"') && str.ends_with('"'));

                let len = str.len();
                unsafe {
                    let bytes = str.as_bytes_mut();
                    bytes.copy_within(1..len - 1, 0); // shift left, remove first and last
                }
                str.truncate(len - 2);
                str
            }
        }
    }
}
