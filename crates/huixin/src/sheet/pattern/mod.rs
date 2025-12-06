use self::midi::MidiPattern;

pub mod curve;
pub mod event;
pub mod midi;

#[derive(Debug, PartialEq, Eq)]
pub enum SheetPatternType {
    Midi,
    // Curve,
    // Event,
}

#[derive(Debug)]
pub enum SheetPattern {
    Midi(MidiPattern),
    // Curve(CurvePattern),
    // Event(EventPattern),
}

pub trait SheetPatternTrait {
    fn name_ref(&self) -> &String;
    fn name_mut(&mut self) -> &mut String;
    fn set_name(&mut self, name: String);

    fn icon_ref(&self) -> &String;
    fn icon_mut(&mut self) -> &mut String;
    fn set_icon(&mut self, icon: String);

    fn beats(&self) -> u64;
}

impl SheetPatternTrait for SheetPattern {
    fn name_ref(&self) -> &String {
        match self {
            Self::Midi(pat) => pat.name_ref(),
        }
    }
    fn name_mut(&mut self) -> &mut String {
        match self {
            Self::Midi(pat) => pat.name_mut(),
        }
    }
    fn set_name(&mut self, name: String) {
        match self {
            Self::Midi(pat) => pat.set_name(name),
        }
    }

    fn icon_ref(&self) -> &String {
        match self {
            Self::Midi(pat) => pat.icon_ref(),
        }
    }
    fn icon_mut(&mut self) -> &mut String {
        match self {
            Self::Midi(pat) => pat.icon_mut(),
        }
    }
    fn set_icon(&mut self, icon: String) {
        match self {
            Self::Midi(pat) => pat.set_icon(icon),
        }
    }

    fn beats(&self) -> u64 {
        match self {
            Self::Midi(pat) => pat.beats(),
        }
    }
}
