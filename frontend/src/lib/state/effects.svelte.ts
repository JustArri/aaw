import * as api from '$lib/generated/api';
import type { Effects, EffectsTuning } from '$lib/generated/models';
import { gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';

async function request<T>(operation: () => Promise<T>): Promise<T> {
    try {
        return await operation();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }
}

function responseError(status: 404 | 500, data: unknown): never {
    if (status === 500) {
        reportGatewayError(status);
        throw gatewayErrorByStatus(500, typeof data === 'string' ? data : undefined);
    }
    throw new Error(typeof data === 'string' && data.trim() ? data : 'Synth does not exist');
}

let tuningState = $state<EffectsTuning | null>(null);
let fetchingTuning = false;

export function effectsTuning() {
    return tuningState;
}

export async function fetchEffectsTuning() {
    if (fetchingTuning || tuningState) {
        return tuningState;
    }
    fetchingTuning = true;
    try {
        const response = await request(api.effectsTuning);
        if (response.status !== 200) {
            responseError(response.status, response.data);
        }
        tuningState = response.data;
        return tuningState;
    } finally {
        fetchingTuning = false;
    }
}

export async function setGlobalEffects(effects: Effects) {
    const response = await request(() => api.setEffects(effects));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
}

export async function setSynthEffects(id: number, effects: Effects) {
    const response = await request(() => api.synthSetEffects(id, effects));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
}
