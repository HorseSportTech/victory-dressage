use std::sync::Mutex;

use crate::domain::position::Position;
use crate::domain::starter::{Starter, StarterResult};
use hypertext::{html_elements, GlobalAttributes};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AlertManager(Mutex<Vec<Alert>>);

#[derive(serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
pub struct Alert {
    r#type: AlertType,
    k: bool,
    e: bool,
    h: bool,
    c: bool,
    m: bool,
    b: bool,
    f: bool,
}
impl Alert {
    pub fn new(r#type: AlertType) -> Self {
        Self {
            r#type,
            k: false,
            e: false,
            h: false,
            c: false,
            m: false,
            b: false,
            f: false,
        }
    }
    pub fn is_empty(&self) -> bool {
        !(self.k || self.e || self.h || self.c || self.m || self.b || self.f)
    }
    pub fn toggle(&mut self, position: &Position) -> bool {
        match position {
            Position::K => self.k = !self.k,
            Position::E => self.e = !self.e,
            Position::H => self.h = !self.h,
            Position::C => self.c = !self.c,
            Position::M => self.m = !self.m,
            Position::B => self.b = !self.b,
            Position::F => self.f = !self.f,
        }
        self.is_empty()
    }
    pub fn set(&mut self, position: &Position, value: bool) -> bool {
        match position {
            Position::K => self.k = value,
            Position::E => self.e = value,
            Position::H => self.h = value,
            Position::C => self.c = value,
            Position::M => self.m = value,
            Position::B => self.b = value,
            Position::F => self.f = value,
        }
        self.is_empty()
    }
    pub fn fmt<'a>(&'a self) -> hypertext::Lazy<impl Fn(&mut String) + 'a> {
        hypertext::rsx! {
            <li class="alert-line">
                <div class="key">{ format!("{}",self.r#type) }</div>
                { get_position_checkoff(&Position::K, self.k) }
                { get_position_checkoff(&Position::E, self.e) }
                { get_position_checkoff(&Position::H, self.h) }
                { get_position_checkoff(&Position::C, self.c) }
                { get_position_checkoff(&Position::M, self.m) }
                { get_position_checkoff(&Position::B, self.b) }
                { get_position_checkoff(&Position::F, self.f) }
            </li>
        }
    }
}
fn get_position_checkoff<'a>(
    position: &'a Position,
    include: bool,
) -> hypertext::Lazy<impl Fn(&mut String) + 'a> {
    hypertext::rsx_move! {
        <label class=format!("position-{}", position.to_string().to_lowercase())>{position.to_string()}
            @if include {
                <input type="checkbox" disabled checked/>
            } @else {
                <input type="checkbox" disabled />
            }
        </label>
    }
}
impl AlertManager {
    /// Toggles a control and the returns a bool relating to whether to show
    /// the alert manager or not (if it is empty)
    pub fn toggle(&self, r#type: AlertType, position: &Position) -> bool {
        let visible = if let Ok(mut list) = self.0.lock() {
            let item = list.iter_mut().find(|item| item.r#type == r#type);
            if let Some(item) = item {
                item.toggle(position)
            } else {
                let mut item = Alert::new(r#type);
                let empty = item.toggle(position);
                list.push(item);
                empty
            }
        } else {
            false
        };
        self.filter();
        visible
    }
    pub fn set(&self, r#type: AlertType, position: &Position, value: bool) -> bool {
        let visible = if let Ok(mut list) = self.0.lock() {
            let item = list.iter_mut().find(|item| item.r#type == r#type);
            if let Some(item) = item {
                item.set(position, value)
            } else {
                let mut item = Alert::new(r#type);
                let empty = item.set(position, value);
                list.push(item);
                empty
            }
        } else {
            false
        };
        self.filter();
        visible
    }
    pub fn filter(&self) {
        let mut list = self.0.lock().expect("Must be able to get list");
        *list = list.drain(..).filter(|x| !x.is_empty()).collect::<Vec<_>>();
    }
    pub fn fmt<'a>(&'a self) -> hypertext::Lazy<impl Fn(&mut String) + use<'a>> {
        let list = self.0.lock().map_or(vec![], |x| (*x).clone());
        hypertext::rsx_move! {
            <ul class="alert-list">
                @for x in list.iter() { {x.fmt()} }
            </ul>
        }
    }
    pub fn get_length(&self) -> usize {
        self.0.lock().map_or(0, |x| x.len())
    }
    pub fn new() -> Self {
        Self(Mutex::new(Vec::new()))
    }

    pub fn merge_starter(&self, starter: &Starter) {
        let mut list = self.0.lock().expect("Must be able to get list");
        *list = starter.warnings.clone();
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Default, Clone, Debug)]
pub enum AlertType {
    ErrorOfCourse(u8),
    TechnicalPenalty(u8),
    ArtisticPenalty(u8),
    #[default]
    Meeting,
    Blood,
    Lameness,
    Equipment,
    Status(StarterResult),
}
impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AlertType::*;
        use StarterResult::*;
        write!(
            f,
            "{}",
            match self {
                ErrorOfCourse(n) => format!("Error {n}"),
                TechnicalPenalty(n) => format!("Tech. {n}"),
                ArtisticPenalty(n) => format!("Art. {n}"),
                Meeting => "Meeting".to_string(),
                Blood => "Blood".to_string(),
                Lameness => "Lameness".to_string(),
                Equipment => "Equipment".to_string(),
                Status(s) => match s {
                    Eliminated(_) => "Elim".to_string(),
                    Withdrawn => "WD".to_string(),
                    NoShow => "No Show".to_string(),
                    Retired => "Ret.".to_string(),
                    _ => String::new(),
                },
            }
        )
    }
}
