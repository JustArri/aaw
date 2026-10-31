pub use ctr::*;
pub use rt::*;
pub use stored::*;

mod ctr;
mod rt;
mod stored;

pub fn prepare(stored: StoredController) -> (CtrController, RtController) {
    match stored {
        StoredController::MidiInput(controller) => {
            let (ctr, rt) = prepare_midi_input(controller);
            (CtrController::MidiInput(ctr), RtController::MidiInput(rt))
        }
    }
}

pub fn default_controller() -> (CtrController, RtController) {
    let controller = CtrMidiInputController::default();

    (
        CtrController::MidiInput(controller),
        RtController::MidiInput(RtMidiInputController::default()),
    )
}

fn prepare_midi_input(
    stored: StoredMidiInputController,
) -> (CtrMidiInputController, RtMidiInputController) {
    let rt = RtMidiInputController {
        enabled: stored.enabled,
        ignore_key_transform: stored.ignore_key_transform,
        key_transform: stored.key_transform.compile(),
        ignore_velocity_transform: stored.ignore_velocity_transform,
        velocity_transform: stored.velocity_transform.compile(),
        channel_mask: stored.channel_filter.as_u16(),
        cc_mask: stored.cc_filter.as_u128(),
        ignore_pitch_bend: stored.ignore_pitch_bend,
        ignore_program_change: stored.ignore_program_change,
        ignore_channel_aftertouch: stored.ignore_channel_aftertouch,
        ignore_polyphonic_aftertouch: stored.ignore_polyphonic_aftertouch,
        use_global_transposition: stored.use_global_transposition,
    };
    let ctr = CtrMidiInputController {
        enabled: stored.enabled,
        ignore_key_transform: stored.ignore_key_transform,
        key_transform: stored.key_transform,
        ignore_velocity_transform: stored.ignore_velocity_transform,
        velocity_transform: stored.velocity_transform,
        channel_filter: stored.channel_filter,
        cc_filter: stored.cc_filter,
        ignore_pitch_bend: stored.ignore_pitch_bend,
        ignore_program_change: stored.ignore_program_change,
        ignore_channel_aftertouch: stored.ignore_channel_aftertouch,
        ignore_polyphonic_aftertouch: stored.ignore_polyphonic_aftertouch,
        use_global_transposition: stored.use_global_transposition,
        name: stored.name,
    };
    (ctr, rt)
}
