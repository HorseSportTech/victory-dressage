use std::collections::HashMap;

use crate::domain::{position::Position, starter::StarterResult};

//TODO: Need to account for how other judges can be prefilled into this
// when it is being initially loaded.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Warnings {
    pub blood: PositionedWarning,
    pub lameness: PositionedWarning,
    pub equipement: PositionedWarning,
    pub meeting: PositionedWarning,

    pub status: HashMap<String, PositionedWarning>,

    pub errors: HashMap<u8, PositionedWarning>,
    pub tech_penalties: HashMap<u8, PositionedWarning>,
    pub art_penalties: HashMap<u8, PositionedWarning>,
}

impl Default for Warnings {
    fn default() -> Self {
        Self {
            blood: PositionedWarning::default(),
            lameness: PositionedWarning::default(),
            equipement: PositionedWarning::default(),
            meeting: PositionedWarning::default(),

            status: {
                let mut map = HashMap::new();
                map.insert(
                    StarterResult::InProgress(0).to_string(),
                    PositionedWarning::default(),
                );
                map
            },

            errors: HashMap::new(),
            tech_penalties: HashMap::new(),
            art_penalties: HashMap::new(),
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Default, Clone, Debug)]
pub struct PositionedWarning {
    k: bool,
    e: bool,
    h: bool,
    c: bool,
    m: bool,
    b: bool,
    f: bool,
}
impl PositionedWarning {
    pub fn get(&self, pos: Position) -> bool {
        match pos {
            Position::K => self.k,
            Position::E => self.e,
            Position::H => self.h,
            Position::C => self.c,
            Position::M => self.m,
            Position::B => self.b,
            Position::F => self.f,
        }
    }
    pub fn set(&mut self, pos: Position, value: bool) {
        match pos {
            Position::K => self.k = value,
            Position::E => self.e = value,
            Position::H => self.h = value,
            Position::C => self.c = value,
            Position::M => self.m = value,
            Position::B => self.b = value,
            Position::F => self.f = value,
        }
    }
    pub fn toggle(&mut self, pos: &Position) -> bool {
        match pos {
            Position::K => {
                self.k = !self.k;
                self.k
            }
            Position::E => {
                self.e = !self.e;
                self.e
            }
            Position::H => {
                self.h = !self.h;
                self.h
            }
            Position::C => {
                self.c = !self.c;
                self.c
            }
            Position::M => {
                self.m = !self.m;
                self.m
            }
            Position::B => {
                self.b = !self.b;
                self.b
            }
            Position::F => {
                self.f = !self.f;
                self.f
            }
        }
    }
}

