import { gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import { unreachable } from '$lib/util';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';
import { clearActiveMidiNotes } from './midi-events.svelte';
import type { EngineMetricsSnapshot } from '$lib/generated/models';

let metricsState = $state(null as EngineMetricsSnapshot | null);
let fetchingMetrics = false;
let resettingMetrics = false;

export function engineMetrics() {
    return metricsState;
}

export async function fetchEngineMetrics() {
    if (fetchingMetrics) {
        return;
    }
    fetchingMetrics = true;

    let response;

    try {
        response = await api.engineMetrics();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetchingMetrics = false;
    }

    if (response.status !== 200) {
        reportGatewayError(response.status);
        throw gatewayErrorByStatus(response.status);
    }

    metricsState = response.data;
}

export async function resetEngineMetrics() {
    if (resettingMetrics) {
        return;
    }
    resettingMetrics = true;

    let response;

    try {
        response = await api.resetEngineMetrics();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        resettingMetrics = false;
    }

    if (response.status !== 204) {
        reportGatewayError(response.status);
        throw gatewayErrorByStatus(response.status);
    }

    await fetchEngineMetrics();
}

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
