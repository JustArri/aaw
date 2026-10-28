import { gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import { unreachable } from '$lib/util';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';
import { clearActiveMidiNotes } from './midi-events.svelte';

export async function panicEngine() {
    let response;

    try {
        response = await api.silence();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }

    switch (response.status) {
        case 204:
            clearActiveMidiNotes();
            return;
        case 500:
            reportGatewayError(response.status);
            throw gatewayErrorByStatus(response.status, response.data);
        default:
            unreachable(response);
    }
}
