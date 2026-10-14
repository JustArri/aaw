use crate::{midi::event::MidiEvent, rt_channels::NotifiedRtSharedProducer};
use midir::{
    ConnectError, ConnectErrorKind, InitError, MidiInput, MidiInputConnection, PortInfoError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{info, warn};

const LOG_TARGET: &str = "midi-runtime";

#[derive(Debug, Error)]
pub enum InputSlotError {
    #[error("slot already disconnected")]
    AlreadyDisconnected,

    #[error("slot already connected")]
    AlreadyConnected,

    #[error("slot not connected")]
    NotConnected,

    #[error("slot not disconnected")]
    NotDisconnected,

    #[error("invalid slot id: {0}")]
    InvalidId(usize),

    #[error("failed to connect input: {0}")]
    Connect(ConnectErrorKind),

    #[error("failed to get MIDI port information: {0}")]
    PortInfo(#[from] PortInfoError),
}

pub enum InputSlot {
    Connected(ConnectedInputSlot),
    Disconnected(DisconnectedInputSlot),
    Empty,
}

pub struct ConnectedInputSlot {
    inner: MidiInputConnection<()>,
    senders: Vec<NotifiedRtSharedProducer<MidiEvent>>,
    name: String,
    port_id: String,
}

impl ConnectedInputSlot {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn port_id(&self) -> &String {
        &self.port_id
    }

    pub fn disconnect(self) -> DisconnectedInputSlot {
        DisconnectedInputSlot {
            inner: self.inner.close().0,
            senders: self.senders,
            name: self.name,
        }
    }
}

pub struct DisconnectedInputSlot {
    inner: MidiInput,
    senders: Vec<NotifiedRtSharedProducer<MidiEvent>>,
    name: String,
}

impl DisconnectedInputSlot {
    pub fn new(
        name: impl Into<String>,
        senders: Vec<NotifiedRtSharedProducer<MidiEvent>>,
    ) -> Result<Self, InitError> {
        let name = name.into();

        Ok(Self { inner: MidiInput::new(&name)?, senders, name })
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn connect(
        self,
        port_id: &str,
    ) -> Result<ConnectedInputSlot, ConnectError<DisconnectedInputSlot>> {
        let ports = self.inner.ports();
        let Some(port) = ports.iter().find(|&p| p.id() == port_id) else {
            return Err(ConnectError::new(ConnectErrorKind::InvalidPort, self));
        };

        let senders = self.senders.clone();

        let connect_result = self.inner.connect(
            port,
            &self.name,
            move |_, msg, _| {
                if let Some(event) = super::event::parse_midi_message(msg) {
                    for sender in &senders {
                        if let Err(event) = sender.blocking_push(event) {
                            warn!(
                                target: LOG_TARGET,
                                "failed to push midi event: {event:#?}"
                            );
                        }
                    }
                }
            },
            (),
        );

        match connect_result {
            Ok(con) => Ok(ConnectedInputSlot {
                inner: con,
                senders: self.senders,
                name: self.name,
                port_id: port_id.into(),
            }),
            Err(err) => {
                let kind = err.kind();
                let inner = err.into_inner();
                Err(ConnectError::new(
                    kind,
                    DisconnectedInputSlot { inner, senders: self.senders, name: self.name },
                ))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputPortInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAutoConnection {
    pub preferred_slot_id: Option<usize>,
    pub slot_id_fixed: bool,
}

pub struct MidiRuntime {
    inputs: Vec<InputSlot>,
    port_discovery: MidiInput,
    available_ports: Vec<InputPortInfo>,
    input_auto_connections: HashMap<String, InputAutoConnection>,
}

impl MidiRuntime {
    pub fn new(
        input_count: usize,
        client_name: impl Into<String>,
        senders: Vec<NotifiedRtSharedProducer<MidiEvent>>,
        input_auto_connections: HashMap<String, InputAutoConnection>,
    ) -> Result<Self, InitError> {
        let client_name = client_name.into();

        let inputs = (0..input_count)
            .map(|i| {
                DisconnectedInputSlot::new(format!("{client_name}-{i}"), senders.clone())
                    .map(InputSlot::Disconnected)
            })
            .collect::<Result<_, _>>()?;

        let port_discovery = MidiInput::new(&format!("{client_name}-port-discovery"))?;
        let available_ports = Default::default();

        Ok(Self { inputs, port_discovery, available_ports, input_auto_connections })
    }

    pub fn inputs(&self) -> &Vec<InputSlot> {
        &self.inputs
    }

    pub fn connect_input(&mut self, slot_id: usize, port_id: &str) -> Result<(), InputSlotError> {
        let slot = self.inputs.get_mut(slot_id).ok_or(InputSlotError::InvalidId(slot_id))?;
        connect_input_slot(slot, port_id)
    }

    pub fn disconnect_input(&mut self, slot_id: usize) -> Result<(), InputSlotError> {
        let slot = self.inputs.get_mut(slot_id).ok_or(InputSlotError::InvalidId(slot_id))?;
        disconnect_input_slot(slot)
    }

    pub fn port_id(&self, slot_id: usize) -> Result<&str, InputSlotError> {
        let slot = self.inputs.get(slot_id).ok_or(InputSlotError::InvalidId(slot_id))?;

        match slot {
            InputSlot::Connected(slot) => Ok(&slot.port_id),
            InputSlot::Disconnected(_) => Err(InputSlotError::NotConnected),
            InputSlot::Empty => unreachable!("temporary empty input slot leaked"),
        }
    }

    pub fn refresh_input_ports(&mut self) -> Result<(), PortInfoError> {
        let mut current = HashMap::new();

        for port in self.port_discovery.ports() {
            let id = port.id();
            let name = self.port_discovery.port_name(&port)?;
            current.insert(id.clone(), InputPortInfo { id, name });
        }

        self.available_ports = self
            .port_discovery
            .ports()
            .into_iter()
            .map(|port| {
                let id = port.id();
                let name = self.port_discovery.port_name(&port)?;
                Ok(InputPortInfo { id, name })
            })
            .collect::<Result<_, _>>()?;

        Ok(())
    }

    pub fn update_input_connections(&mut self) -> Result<(), InputSlotError> {
        self.disconnect_non_existent_inputs()?;
        self.try_autoconnect_inputs()?;
        Ok(())
    }

    pub fn available_input_ports(&self) -> &Vec<InputPortInfo> {
        &self.available_ports
    }

    pub fn input_auto_connections(&self) -> &HashMap<String, InputAutoConnection> {
        &self.input_auto_connections
    }

    pub fn input_auto_connections_mut(&mut self) -> &mut HashMap<String, InputAutoConnection> {
        &mut self.input_auto_connections
    }

    fn disconnect_non_existent_inputs(&mut self) -> Result<(), InputSlotError> {
        for slot_id in 0..self.inputs.len() {
            if let Ok(port_id) = self.port_id(slot_id)
                && !is_input_port_available(&self.available_ports, port_id)
            {
                info!(
                    target: LOG_TARGET,
                    "auto disconnecting midi input: {} from slot {}", port_id, slot_id
                );
            } else {
                continue;
            }
            self.disconnect_input(slot_id)?;
        }
        Ok(())
    }

    fn try_autoconnect_inputs(&mut self) -> Result<(), InputSlotError> {
        for (port_id, state) in &mut self.input_auto_connections {
            if input_port_connected(&self.inputs, port_id)
                || !is_input_port_available(&self.available_ports, port_id)
            {
                continue;
            }

            let Some((slot_id, slot)) = auto_slot_mut(&mut self.inputs, state) else {
                info!(
                    target: LOG_TARGET,
                    "no available slots for auto-connect of port: {}", port_id
                );
                continue;
            };

            connect_input_slot(slot, port_id)?;
            info!(
                target: LOG_TARGET,
                "auto connected midi input: {} to slot {}", port_id, slot_id
            );
        }

        Ok(())
    }
}

fn is_input_port_available(available_ports: &[InputPortInfo], port_id: &str) -> bool {
    available_ports.iter().find(|p| p.id == port_id).is_some()
}

fn input_port_connected(inputs: &[InputSlot], port_id: &str) -> bool {
    inputs
        .iter()
        .any(|slot| matches!(slot, InputSlot::Connected(slot) if slot.port_id() == port_id))
}

fn input_slot_connected(inputs: &[InputSlot], slot_id: usize) -> bool {
    inputs
        .iter()
        .enumerate()
        .any(|(id, slot)| id == slot_id && matches!(slot, InputSlot::Connected(_)))
}

fn auto_slot_mut<'a>(
    inputs: &'a mut [InputSlot],
    state: &InputAutoConnection,
) -> Option<(usize, &'a mut InputSlot)> {
    let slot_id = match state.preferred_slot_id {
        Some(id) if !input_slot_connected(inputs, id) => id,
        Some(_) if state.slot_id_fixed => return None,
        _ => inputs.iter().position(|slot| matches!(slot, InputSlot::Disconnected(_)))?,
    };

    Some((slot_id, inputs.get_mut(slot_id)?))
}

fn connect_input_slot(slot: &mut InputSlot, port_id: &str) -> Result<(), InputSlotError> {
    let current = std::mem::replace(slot, InputSlot::Empty);

    let (next, result) = match current {
        InputSlot::Connected(slot) => {
            (InputSlot::Connected(slot), Err(InputSlotError::AlreadyConnected))
        }

        InputSlot::Disconnected(slot) => match slot.connect(port_id) {
            Ok(slot) => (InputSlot::Connected(slot), Ok(())),
            Err(error) => {
                let kind = error.kind();
                let disconnected = error.into_inner();

                (InputSlot::Disconnected(disconnected), Err(InputSlotError::Connect(kind)))
            }
        },

        InputSlot::Empty => unreachable!("temporary empty input slot leaked"),
    };

    *slot = next;
    result
}

fn disconnect_input_slot(slot: &mut InputSlot) -> Result<(), InputSlotError> {
    let current = std::mem::replace(slot, InputSlot::Empty);

    let (next, result) = match current {
        InputSlot::Connected(slot) => (InputSlot::Disconnected(slot.disconnect()), Ok(())),

        InputSlot::Disconnected(slot) => {
            (InputSlot::Disconnected(slot), Err(InputSlotError::AlreadyDisconnected))
        }

        InputSlot::Empty => unreachable!("temporary empty input slot leaked"),
    };

    *slot = next;
    result
}
