import type { Notification, ProjectChange } from '$lib/generated/models';
import * as api from '$lib/generated/api';
import { notificationStream } from '$lib/notification-stream';
import { recordMidiEvent } from '$lib/state/midi-events.svelte';
import {
    fetchInstrumentSlots,
    updateInstrumentGlobalTransposition,
    updateInstrumentSlot
} from '$lib/state/instruments.svelte';
import {
    fetchSynthInstrumentSlots,
    fetchSynths,
    removeSynthFromState,
    updateFluidSynthProgram,
    updateSynthEffects,
    updateSynthInstrumentSlot,
    updateSynthName,
    updateSynthParameters
} from '$lib/state/synths.svelte';
import { fetchUserPresets } from '$lib/state/user-presets.svelte';
import { unreachable } from '$lib/util';
import { gatewayOnline, reportGatewayOffline } from './gateway.svelte';
import {
    fetchActiveProject,
    updateActiveProjectEffects,
    updateActiveProjectName,
    updateActiveProjectStorage,
    updateActiveProjectRevision
} from './active-project.svelte';

let abortCtr = null as AbortController | null;

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
        case 'Effects':
            updateActiveProjectEffects(change.data.effects);
            break;
        case 'Synth':
            if (change.data.removed) {
                removeSynthFromState(change.data.id);
            } else {
                await fetchSynths();
            }
            break;
        case 'SynthSoundfontProgram':
            updateFluidSynthProgram(change.data.id, change.data.program);
            break;
        case 'SynthKind':
            updateSynthParameters(change.data.id, change.data.parameters);
            break;
        case 'SynthName':
            updateSynthName(change.data.id, change.data.name);
            break;
        case 'SynthEffects':
            updateSynthEffects(change.data.id, change.data.effects);
            break;
        case 'GlobalTransposition':
            updateInstrumentGlobalTransposition(change.data.transposition);
            break;
        case 'Instrument':
            await Promise.all([fetchInstrumentSlots(), fetchSynthInstrumentSlots()]);
            break;
        case 'InstrumentSynth':
            updateInstrumentSlot(change.data.id, { synth_id: change.data.synth_id ?? null });
            updateSynthInstrumentSlot(change.data.id, {
                synth_id: change.data.synth_id ?? null
            });
            break;
        case 'InstrumentEnabled':
            updateInstrumentSlot(change.data.id, { enabled: change.data.flag });
            updateSynthInstrumentSlot(change.data.id, { enabled: change.data.flag });
            break;
        case 'InstrumentIgnoreKeyTransform':
            updateInstrumentSlot(change.data.id, { ignore_key_transform: change.data.flag });
            break;
        case 'InstrumentKeyTransform':
            updateInstrumentSlot(change.data.id, { key_transform: change.data.transform });
            break;
        case 'InstrumentIgnoreVelocityTransform':
            updateInstrumentSlot(change.data.id, { ignore_velocity_transform: change.data.flag });
            break;
        case 'InstrumentVelocityTransform':
            updateInstrumentSlot(change.data.id, { velocity_transform: change.data.transform });
            break;
        case 'InstrumentChannelFilter':
            updateInstrumentSlot(change.data.id, { channel_filter: change.data.filter });
            break;
        case 'InstrumentCcFilter':
            updateInstrumentSlot(change.data.id, { cc_filter: change.data.filter });
            break;
        case 'InstrumentIgnorePitchBend':
            updateInstrumentSlot(change.data.id, { ignore_pitch_bend: change.data.flag });
            break;
        case 'InstrumentIgnoreProgramChange':
            updateInstrumentSlot(change.data.id, { ignore_program_change: change.data.flag });
            break;
        case 'InstrumentIgnoreChannelAftertouch':
            updateInstrumentSlot(change.data.id, {
                ignore_channel_aftertouch: change.data.flag
            });
            break;
        case 'InstrumentIgnorePolyphonicAftertouch':
            updateInstrumentSlot(change.data.id, {
                ignore_polyphonic_aftertouch: change.data.flag
            });
            break;
        case 'InstrumentUseGlobalTransposition':
            updateInstrumentSlot(change.data.id, {
                use_global_transposition: change.data.flag
            });
            break;
        case 'InstrumentName':
            updateInstrumentSlot(change.data.id, { name: change.data.name });
            updateSynthInstrumentSlot(change.data.id, { name: change.data.name });
            break;
        case 'UserPresets':
            await fetchUserPresets();
            break;
        default:
            unreachable(type);
    }
}
