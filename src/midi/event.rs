use crate::midi::CompiledMidiKeyTransform;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct MidiEvent {
    pub timestamp: Option<u64>,
    pub channel: u8,

    #[serde(flatten)]
    pub kind: MidiEventKind,
}

impl MidiEvent {
    pub fn transformed(
        self,
        transform: &CompiledMidiKeyTransform,
        transposition: i8,
    ) -> Option<Self> {
        let kind = match self.kind {
            MidiEventKind::NoteOn { key, velocity } => {
                MidiEventKind::NoteOn { key: transform.apply(key, transposition)?, velocity }
            }
            MidiEventKind::NoteOff { key, velocity } => {
                MidiEventKind::NoteOff { key: transform.apply(key, transposition)?, velocity }
            }
            MidiEventKind::PolyphonicAftertouch { key, value } => {
                MidiEventKind::PolyphonicAftertouch {
                    key: transform.apply(key, transposition)?,
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

pub fn parse_midi_message(message: &[u8]) -> Option<MidiEvent> {
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

    Some(MidiEvent { timestamp: None, channel, kind })
}
