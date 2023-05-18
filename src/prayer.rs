#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Prayer {
    Fajr = 0,
    Sunrise,
    Dhuhr,
    Asr,
    Maghrib,
    Isha,
    FajrNextDay,
}

impl From<u8> for Prayer {
    fn from(value: u8) -> Self {
        match value {
            0 => Prayer::Fajr,
            1 => Prayer::Sunrise,
            2 => Prayer::Dhuhr,
            3 => Prayer::Asr,
            4 => Prayer::Maghrib,
            5 => Prayer::Isha,
            6 => Prayer::FajrNextDay,
            _ => Prayer::Fajr,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RemainingTime {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub next_prayer: Prayer,
}
