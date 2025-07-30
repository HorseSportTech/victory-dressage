use battery::State;

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum VirtualDeviceBattery {
    Charging(f32),
    Discharging(f32),
    #[default]
    Unknown,
    Error,
}

#[cfg(target_os = "ios")]
impl From<swift_bat::BatteryState> for VirtualDeviceBattery {
    fn from(value: swift_bat::BatteryState) -> Self {
        match value {
            swift_bat::BatteryState::Charging(val) => Self::Charging(val * 100.0),
            swift_bat::BatteryState::Discharging(val) => Self::Discharging(val * 100.0),
            swift_bat::BatteryState::Unknown => Self::Unknown,
            swift_bat::BatteryState::Error => Self::Error,
        }
    }
}

impl VirtualDeviceBattery {
    pub fn new() -> Self {
        VirtualDeviceBattery::Error
    }
    pub fn check(&mut self) {
        if Self::inner_check(self).is_err() {
            *self = Self::Error;
        };
    }
    fn inner_check(battery_state: &mut Self) -> Result<(), battery::Error> {
        if cfg!(target_os = "ios") {
            #[cfg(target_os = "ios")]
            {
                let bat = swift_bat::get_battery_state();
                *battery_state = bat.into();
            }
        } else {
            let manager = battery::Manager::new()?;
            let mut batteries = manager.batteries()?;
            let mut level: f32;
            while let Some(Ok(battery)) = batteries.next() {
                level = battery.state_of_charge().into();
                *battery_state = match battery.state() {
                    State::Charging => Self::Charging(level * 100.0),
                    State::Discharging => Self::Discharging(level * 100.0),
                    State::Full => Self::Charging(100.0),
                    State::Unknown => Self::Unknown,
                    _ => Self::Error,
                };
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for VirtualDeviceBattery {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Charging(level) => write!(f, "Battery Charging, Level {level:.0}%"),
            Self::Discharging(level) => write!(f, "Battery Discharging, Level {level:.0}%"),
            Self::Unknown => write!(f, "Battery in unknown state"),
            Self::Error => write!(f, "Error with battery"),
        }
    }
}
