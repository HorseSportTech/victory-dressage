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
        return !(self.k || self.e || self.h || self.c || self.m || self.b || self.f);
    }
    pub fn toggle(&mut self, position: Position) -> bool {
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
    pub fn fmt<'a>(&'a self) -> hypertext::Lazy<impl Fn(&mut String) + 'a> {
        hypertext::rsx! {
            <li class="alert-line">
                <div class="row">{ format!("{}",self.r#type) }</div>
                <div class="position-k"><input type="checkbox" value=self.k/></div>
                <div class="position-e"><input type="checkbox" value=self.e/></div>
                <div class="position-h"><input type="checkbox" value=self.h/></div>
                <div class="position-c"><input type="checkbox" value=self.c/></div>
                <div class="position-m"><input type="checkbox" value=self.m/></div>
                <div class="position-b"><input type="checkbox" value=self.b/></div>
                <div class="position-f"><input type="checkbox" value=self.f/></div>
            </li>
        }
    }
}
impl AlertManager {
    pub fn toggle(&self, r#type: AlertType, position: Position) -> bool {
        if let Ok(mut list) = self.0.lock() {
            for item in list.iter_mut() {
                if item.r#type == r#type {
                    return item.toggle(position);
                }
            }
            let mut new = Alert::new(r#type);
            let empty = new.toggle(position);
            list.push(new);
            return empty;
        }
        true
    }
    pub fn filter(&mut self) {
        let mut list = self.0.lock().expect("Must be able to get list");
        *list = list
            .drain(..)
            .into_iter()
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();
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

    pub fn from_starter(&self, starter: &Starter) {
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
                Meeting => format!("Meeting"),
                Blood => format!("Blood"),
                Lameness => format!("Lameness"),
                Equipment => format!("Equipment"),
                Status(s) => match s {
                    Eliminated(_) => format!("Elim"),
                    Withdrawn => format!("WD"),
                    NoShow => format!("No Show"),
                    Retired => format!("Ret."),
                    _ => format!(""),
                },
            }
        )
    }
}
