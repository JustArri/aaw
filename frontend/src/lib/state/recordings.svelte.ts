import { gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type { RecordingInfo } from '$lib/generated/models';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';

let recordingsState = $state<RecordingInfo[]>([]);

export function recordings() {
    return recordingsState;
}

export async function fetchRecordings() {
    let response;
    try {
        response = await api.listRecordings();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }

    if (response.status === 200) {
        recordingsState = response.data;
        return;
    }
    reportGatewayError(response.status);
    throw gatewayErrorByStatus(response.status, response.data);
}

export async function deleteRecording(path: string) {
    let response;
    try {
        response = await api.deleteRecording({ path });
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }

    if (response.status === 204) {
        recordingsState = recordingsState.filter((recording) => recording.path !== path);
        return;
    }
    if (response.status === 500) {
        reportGatewayError(response.status);
    }
    throw new Error(response.data || `Could not delete recording (${response.status})`);
}
