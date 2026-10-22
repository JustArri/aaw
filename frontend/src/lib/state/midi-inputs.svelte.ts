import { gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type { MidiInputAutoConnection, MidiInputSlot, MidiPort } from '$lib/generated/models';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';

let portsState = $state([] as MidiPort[]);
let slotsState = $state([] as MidiInputSlot[]);
let autoConnectionsState = $state([] as MidiInputAutoConnection[]);
let fetching = false;

export function midiInputPorts() {
    return portsState;
}

export function midiInputSlots() {
    return slotsState;
}

export function midiInputAutoConnections() {
    return autoConnectionsState;
}

export function resetMidiInputs() {
    portsState = [];
    slotsState = [];
    autoConnectionsState = [];
}

export async function fetchMidiInputs() {
    if (fetching) {
        return;
    }
    fetching = true;

    let responses;

    try {
        responses = await Promise.all([
            api.midiInputPorts(),
            api.midiInputInstruments(),
            api.midiInputAutoConnections()
        ]);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetching = false;
    }

    const [portsResponse, slotsResponse, autoConnectionsResponse] = responses;

    if (portsResponse.status !== 200) {
        resetMidiInputs();
        reportGatewayError(portsResponse.status);
        throw gatewayErrorByStatus(portsResponse.status);
    }

    if (slotsResponse.status !== 200) {
        resetMidiInputs();
        reportGatewayError(slotsResponse.status);
        throw gatewayErrorByStatus(slotsResponse.status);
    }

    portsState = portsResponse.data;
    slotsState = slotsResponse.data;
    autoConnectionsState = autoConnectionsResponse.data;
}

export async function connectMidiInput(slotId: number, portId: string) {
    let response;

    try {
        response = await api.midiConnectInputSlot(slotId, { port_id: portId });
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }

    switch (response.status) {
        case 204:
            slotsState = slotsState.map((slot, index) =>
                index === slotId ? { type: 'Connected', data: { port_id: portId } } : slot
            );
            break;
        default:
            reportGatewayError(response.status);
            throw gatewayErrorByStatus(response.status, response.data);
    }
}

export async function disconnectMidiInput(slotId: number) {
    let response;

    try {
        response = await api.midiDisconnectInputSlot(slotId);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }

    switch (response.status) {
        case 204:
            slotsState = slotsState.map((slot, index) =>
                index === slotId ? { type: 'Disconnected' } : slot
            );
            break;
        default:
            reportGatewayError(response.status);
            throw gatewayErrorByStatus(response.status, response.data);
    }
}

export async function setMidiInputAutoConnection(connection: MidiInputAutoConnection) {
    try {
        await api.midiInsertInputAutoConnection(connection);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }

    autoConnectionsState = [
        ...autoConnectionsState.filter((entry) => entry.port_id !== connection.port_id),
        connection
    ];
}

export async function deleteMidiInputAutoConnection(portId: string) {
    try {
        await api.midiDeleteInputAutoConnection({ port_id: portId });
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }

    autoConnectionsState = autoConnectionsState.filter((entry) => entry.port_id !== portId);
}
