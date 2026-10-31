import { gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type {
    CtrController,
    Instrument,
    MidiFilter,
    MidiKeyTransform,
    MidiVelocityTransform
} from '$lib/generated/models';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';
import type { ControllerSlot } from './synths.svelte';

let slotsState = $state({} as Record<string, ControllerSlot>);
let globalTranspositionState = $state(0);
let fetching = false;

export function instrumentSlots() {
    return slotsState;
}

export function instrumentGlobalTransposition() {
    return globalTranspositionState;
}

export function resetInstruments() {
    slotsState = {};
    globalTranspositionState = 0;
}

function loadControllers(instruments: Record<string, Instrument>) {
    const slots: Record<string, ControllerSlot> = {};
    for (const [instrumentId, instrument] of Object.entries(instruments)) {
        for (const [controllerId, controller] of Object.entries(instrument.controllers) as [
            string,
            CtrController
        ][]) {
            if (controller.type !== 'MidiInput') {
                continue;
            }
            const key = Number(instrumentId) * 65_536 + Number(controllerId);
            slots[String(key)] = {
                ...controller.data,
                instrument_id: Number(instrumentId),
                controller_id: Number(controllerId),
                synth_id: Number(instrumentId)
            };
        }
    }
    slotsState = slots;
}

export async function fetchInstruments() {
    if (fetching) {
        return;
    }
    fetching = true;
    try {
        const [instruments, transposition] = await Promise.all([
            api.instrumentList(),
            api.globalTransposition()
        ]);
        loadControllers(instruments.data);
        globalTranspositionState = transposition.data;
    } catch (error) {
        resetInstruments();
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetching = false;
    }
}

export async function fetchInstrumentSlots() {
    loadControllers((await request(api.instrumentList)).data);
}

function responseError(status: number, data: unknown): never {
    if (status === 500) {
        reportGatewayError(status);
    }
    if (typeof data === 'string' && data.trim()) {
        throw new Error(data);
    }
    throw new Error(`Controller operation failed (${status})`);
}

async function request<T>(operation: () => Promise<T>): Promise<T> {
    try {
        return await operation();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }
}

export function updateInstrumentSlot(id: number, update: Partial<ControllerSlot>) {
    const slot = slotsState[String(id)];
    if (slot) {
        slotsState = { ...slotsState, [id]: { ...slot, ...update } };
    }
}

export function updateInstrumentGlobalTransposition(value: number) {
    globalTranspositionState = value;
}

async function replaceController(id: number, update: Partial<ControllerSlot>) {
    const slot = slotsState[String(id)];
    if (!slot) {
        throw new Error('Controller does not exist');
    }
    const {
        synth_id: _,
        instrument_id: instrumentId,
        controller_id: controllerId,
        ...data
    } = { ...slot, ...update };
    const response = await request(() =>
        api.controllerReplace(instrumentId, controllerId, { type: 'MidiInput', data })
    );
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, update);
}

export async function addInstrument(instrumentId?: number) {
    if (instrumentId !== undefined) {
        const response = await request(() => api.controllerAppend(instrumentId));
        if (response.status !== 200) {
            responseError(response.status, response.data);
        }
        await fetchInstrumentSlots();
        return instrumentId * 65_536 + response.data;
    }
    const response = await request(() =>
        api.instrumentPush({ type: 'Dummy', data: { max_voices: 64 } })
    );
    if (response.status !== 200) {
        responseError(response.status, response.data);
    }
    await fetchInstrumentSlots();
    return response.data;
}

export async function removeInstrument(id: number) {
    const slot = slotsState[String(id)];
    if (!slot) {
        throw new Error('Controller does not exist');
    }
    const response = await request(() =>
        api.controllerRemove(slot.instrument_id, slot.controller_id)
    );
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    const next = { ...slotsState };
    delete next[String(id)];
    slotsState = next;
}

export async function setInstrumentEnabled(id: number, enabled: boolean) {
    await replaceController(id, { enabled });
}

export async function setInstrumentSynth(id: number, synthId: number | null) {
    if (synthId !== slotsState[String(id)]?.instrument_id) {
        throw new Error('Controllers are components of instruments and cannot be reassigned');
    }
}

export async function setInstrumentKeyTransform(id: number, key_transform: MidiKeyTransform) {
    await replaceController(id, { key_transform });
}

export async function setInstrumentVelocityTransform(
    id: number,
    velocity_transform: MidiVelocityTransform
) {
    await replaceController(id, { velocity_transform });
}

export async function setInstrumentChannelFilter(id: number, channel_filter: MidiFilter) {
    await replaceController(id, { channel_filter });
}

export async function setInstrumentCcFilter(id: number, cc_filter: MidiFilter) {
    await replaceController(id, { cc_filter });
}

export async function setInstrumentIgnoreKeyTransform(id: number, ignore_key_transform: boolean) {
    await replaceController(id, { ignore_key_transform });
}

export async function setInstrumentIgnoreVelocityTransform(
    id: number,
    ignore_velocity_transform: boolean
) {
    await replaceController(id, { ignore_velocity_transform });
}

export async function setInstrumentIgnorePitchBend(id: number, ignore_pitch_bend: boolean) {
    await replaceController(id, { ignore_pitch_bend });
}

export async function setInstrumentIgnoreProgramChange(id: number, ignore_program_change: boolean) {
    await replaceController(id, { ignore_program_change });
}

export async function setInstrumentIgnoreChannelAftertouch(
    id: number,
    ignore_channel_aftertouch: boolean
) {
    await replaceController(id, { ignore_channel_aftertouch });
}

export async function setInstrumentIgnorePolyphonicAftertouch(
    id: number,
    ignore_polyphonic_aftertouch: boolean
) {
    await replaceController(id, { ignore_polyphonic_aftertouch });
}

export async function setInstrumentUsesGlobalTransposition(
    id: number,
    use_global_transposition: boolean
) {
    await replaceController(id, { use_global_transposition });
}

export async function fetchInstrumentName(id: number) {
    await fetchInstrumentSlots();
    return slotsState[String(id)]?.name;
}

export async function setInstrumentName(id: number, name: string) {
    await replaceController(id, { name });
}

export async function setInstrumentGlobalTransposition(value: number) {
    const response = await request(() => api.setGlobalTransposition(value));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentGlobalTransposition(value);
    return value;
}
