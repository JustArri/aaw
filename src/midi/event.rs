use crate::midi::{CompiledMidiKeyTransform, CompiledMidiVelocityTransform};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct TimedMidiEvent {
    #[schema(required)]
    pub timestamp: Option<u64>,

    pub channel: u8,

    #[serde(flatten)]
    pub kind: MidiEventKind,
}

impl TimedMidiEvent {
    pub fn has_timestamp(&self) -> bool {
        self.timestamp.is_some()
    }

    pub fn split(self) -> (Option<u64>, MidiEvent) {
        (self.timestamp, MidiEvent { channel: self.channel, kind: self.kind })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct MidiEvent {
    pub channel: u8,

    #[serde(flatten)]
    pub kind: MidiEventKind,
}

impl From<TimedMidiEvent> for MidiEvent {
    fn from(value: TimedMidiEvent) -> Self {
        Self { channel: value.channel, kind: value.kind }
    }
}

impl MidiEvent {
    pub fn transformed(
        self,
        key_transform: &CompiledMidiKeyTransform,
        velocity_transform: &CompiledMidiVelocityTransform,
        transposition: i8,
    ) -> Option<Self> {
        let kind = match self.kind {
            MidiEventKind::NoteOn { key, velocity } => MidiEventKind::NoteOn {
                key: key_transform.apply(key, transposition)?,
                velocity: velocity_transform.apply(velocity),
            },
            MidiEventKind::NoteOff { key, .. } => MidiEventKind::NoteOff {
                key: key_transform.apply(key, transposition)?,
                velocity: 0,
            },
            MidiEventKind::PolyphonicAftertouch { key, value } => {
                MidiEventKind::PolyphonicAftertouch {
                    key: key_transform.apply(key, transposition)?,
                    value,
                }
            }
            kind => kind,
        };
        Some(Self { kind, ..self })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum MidiEventKind {
    NoteOn { key: u8, velocity: u8 },
    NoteOff { key: u8, velocity: u8 },
    ControlChange { controller: u8, value: u8 },
    PitchBend { value: i16 },
    ProgramChange { program: u8 },
    ChannelAftertouch { value: u8 },
    PolyphonicAftertouch { key: u8, value: u8 },
}

pub fn parse_midi_message(message: &[u8]) -> Option<TimedMidiEvent> {
    let status = *message.first()?;
    let channel = status & 0x0f;
    let data = &message[1..];

    if data.iter().any(|&byte| byte > 0x7F) {
        return None;
    }

    let kind = match status & 0xf0 {
        0x80 if data.len() == 2 => MidiEventKind::NoteOff { key: data[0], velocity: data[1] },
        0x90 if data.len() == 2 => {
            let key = data[0];
            let velocity = data[1];

            if velocity == 0 {
                MidiEventKind::NoteOff { key, velocity }
            } else {
                MidiEventKind::NoteOn { key, velocity }
            }
        }
        0xa0 if data.len() == 2 => {
            MidiEventKind::PolyphonicAftertouch { key: data[0], value: data[1] }
        }
        0xb0 if data.len() == 2 => {
            MidiEventKind::ControlChange { controller: data[0], value: data[1] }
        }
        0xc0 if data.len() == 1 => MidiEventKind::ProgramChange { program: data[0] },
        0xd0 if data.len() == 1 => MidiEventKind::ChannelAftertouch { value: data[0] },
        0xe0 if data.len() == 2 => {
            let raw = ((data[1] as i16) << 7) | data[0] as i16;
            MidiEventKind::PitchBend { value: raw - 8192 }
        }
        _ => return None,
    };

    Some(TimedMidiEvent { timestamp: None, channel, kind })
}
