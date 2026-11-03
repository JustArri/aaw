import type { ControllerAddress, Notification, ProjectChange } from '$lib/generated/models';
import * as api from '$lib/generated/api';
import { notificationStream } from '$lib/notification-stream';
import { clearActiveMidiNotes, recordMidiEvent } from '$lib/state/midi-events.svelte';
import {
    fetchInstrumentSlots,
    updateInstrumentGlobalTransposition,
    updateInstrumentSlot
} from '$lib/state/instruments.svelte';
import {
    fetchSynthInstrumentSlots,
    fetchSynths,
    removeSynthFromState,
    updateClapParameter,
    updateFluidSynthProgram,
    updateSynthEffects,
    updateSynthGain,
    updateSynthInstrumentSlot,
    updateSynthName,
    updateSynthParameters
} from '$lib/state/synths.svelte';
import { fetchUserPresets } from '$lib/state/user-presets.svelte';
import { updateEngineMidiAppControls } from '$lib/state/midi-app-controls.svelte';
import { unreachable } from '$lib/util';
import { gatewayOnline, reportGatewayOffline } from './gateway.svelte';
import {
    fetchActiveProject,
    updateActiveProjectGlobalEffects,
    updateActiveProjectGlobalGain,
    updateActiveProjectMidiAppControls,
    updateActiveProjectName,
    updateActiveProjectStorage,
    updateActiveProjectTransport,
    updateActiveProjectRevision
} from './active-project.svelte';

let abortCtr = null as AbortController | null;

function controllerStateId(address: ControllerAddress) {
    return address.instrument_id * 65_536 + address.controller_id;
}

export function autoSubscribeServerNotifications() {
    $effect(() => {
        if (!gatewayOnline()) {
            return;
        }

        abortCtr = new AbortController();
        receiveAndHandleServerNotifications(abortCtr.signal);

        return () => abortCtr?.abort();
    });
}

async function receiveAndHandleServerNotifications(abortSignal: AbortSignal) {
    try {
        for await (const notification of subscribeServerNotifications(abortSignal)) {
            await handleServerNotification(notification);
        }
    } catch (e) {
        clearActiveMidiNotes();
        reportGatewayOffline();
        console.error(e);
    }
}

function subscribeServerNotifications(signal?: AbortSignal) {
    const url = new URL(api.getNotificationsUrl(), window.location.href);
    url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
    return notificationStream(url, signal);
}

function handleServerNotification(notification: Notification) {
    const type = notification.type;

    switch (type) {
        case 'MidiEvent':
            recordMidiEvent(notification.data);
            break;
        case 'ProjectChange':
            handleProjectChange(notification.data);
            break;
        case 'EngineMidiAppControls':
            updateEngineMidiAppControls(notification.data);
            break;
        default:
            unreachable(type);
    }
}

async function handleProjectChange(change: ProjectChange) {
    const type = change.type;

    console.log('project change', change);
    updateActiveProjectRevision(change.revision);

    switch (type) {
        case 'Full':
            await fetchActiveProject();
            break;
        case 'Storage':
            updateActiveProjectStorage(change.data.storage ?? null);
            break;
        case 'Name':
            updateActiveProjectName(change.data.name);
            break;
        case 'GlobalEffects':
            updateActiveProjectGlobalEffects(change.data.effects);
            break;
        case 'GlobalGain':
            updateActiveProjectGlobalGain(change.data.gain);
            break;
        case 'Instrument':
            if (change.data.removed) {
                removeSynthFromState(change.data.id);
            } else {
                await fetchSynths();
            }
            break;
        case 'InstrumentSoundfontProgram':
            updateFluidSynthProgram(change.data.id, change.data.program);
            break;
        case 'InstrumentKind':
            updateSynthParameters(change.data.id, change.data.kind);
            break;
        case 'InstrumentName':
            updateSynthName(change.data.id, change.data.name);
            break;
        case 'InstrumentEffects':
            updateSynthEffects(change.data.id, change.data.effects);
            break;
        case 'InstrumentGain':
            updateSynthGain(change.data.id, change.data.gain);
            break;
        case 'InstrumentClapParameter':
            updateClapParameter(change.data.id, change.data.parameter_id, change.data.value);
            break;
        case 'GlobalTransposition':
            updateInstrumentGlobalTransposition(change.data.transposition);
            break;
        case 'Controller':
            await Promise.all([fetchInstrumentSlots(), fetchSynthInstrumentSlots()]);
            break;
        case 'ControllerEnabled':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                enabled: change.data.flag
            });
            updateSynthInstrumentSlot(controllerStateId(change.data.address), {
                enabled: change.data.flag
            });
            break;
        case 'ControllerSourceFilter':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                source_filter: change.data.filter
            });
            updateSynthInstrumentSlot(controllerStateId(change.data.address), {
                source_filter: change.data.filter
            });
            break;
        case 'ControllerIgnoreKeyTransform':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                ignore_key_transform: change.data.flag
            });
            break;
        case 'ControllerKeyTransform':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                key_transform: change.data.transform
            });
            break;
        case 'ControllerIgnoreVelocityTransform':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                ignore_velocity_transform: change.data.flag
            });
            break;
        case 'ControllerVelocityTransform':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                velocity_transform: change.data.transform
            });
            break;
        case 'ControllerChannelFilter':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                channel_filter: change.data.filter
            });
            break;
        case 'ControllerCcFilter':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                cc_filter: change.data.filter
            });
            break;
        case 'ControllerIgnorePitchBend':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                ignore_pitch_bend: change.data.flag
            });
            break;
        case 'ControllerIgnoreProgramChange':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                ignore_program_change: change.data.flag
            });
            break;
        case 'ControllerIgnoreChannelAftertouch':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                ignore_channel_aftertouch: change.data.flag
            });
            break;
        case 'ControllerIgnorePolyphonicAftertouch':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                ignore_polyphonic_aftertouch: change.data.flag
            });
            break;
        case 'ControllerUseGlobalTransposition':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                use_global_transposition: change.data.flag
            });
            break;
        case 'ControllerName':
            updateInstrumentSlot(controllerStateId(change.data.address), {
                name: change.data.name
            });
            updateSynthInstrumentSlot(controllerStateId(change.data.address), {
                name: change.data.name
            });
            break;
        case 'UserPresets':
            await fetchUserPresets();
            break;
        case 'MidiAppControls':
            updateActiveProjectMidiAppControls(change.data.controls);
            break;
        case 'Transport':
            updateActiveProjectTransport(change.data.transport);
            break;
        default:
            unreachable(type);
    }
}
