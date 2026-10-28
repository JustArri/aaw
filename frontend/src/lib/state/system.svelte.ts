import { GatewayError, gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type { Preferences } from '$lib/generated/models';
import { unreachable } from '$lib/util';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';

let preferencesState = $state(null as Preferences | null);
let fetchingPreferences = false;
let settingPreferences = false;

export function preferences() {
    return preferencesState;
}

export function resetPreferences() {
    preferencesState = null;
}

export async function fetchPreferences() {
    if (fetchingPreferences) {
        return preferencesState;
    }
    fetchingPreferences = true;

    try {
        const response = await api.preferences();
        preferencesState = response.data;
        return preferencesState;
    } catch (error) {
        resetPreferences();
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetchingPreferences = false;
    }
}

export async function updatePreferences(value: Preferences) {
    if (settingPreferences) {
        return;
    }
    settingPreferences = true;

    let response;
    try {
        response = await api.setPreferences(value);
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        settingPreferences = false;
    }

    switch (response.status) {
        case 204:
            preferencesState = { ...value };
            break;
        case 400:
            throw new GatewayError('bad-request');
        default:
            unreachable(response);
    }
}

async function runSystemAction(action: () => Promise<api.systemRebootResponse>) {
    let response;

    try {
        response = await action();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }

    switch (response.status) {
        case 202:
            return;
        case 403:
            throw new GatewayError('forbidden');
        case 500:
            reportGatewayError(response.status);
            throw gatewayErrorByStatus(response.status);
        default:
            unreachable(response);
    }
}

export async function rebootSystem() {
    await runSystemAction(() => api.systemReboot());
}

export async function shutDownSystem() {
    await runSystemAction(() => api.systemShutdown());
}
