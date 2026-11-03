import { gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type {
    CtrEngineMidiConfiguration,
    EngineMidiConfigurationInfo,
    StoredEngineMidiConfiguration
} from '$lib/generated/models';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';
import { fetchEngineMidiAppControls } from './midi-app-controls.svelte';
import { fetchMidiInputs } from './midi-inputs.svelte';

let configurationsState = $state<EngineMidiConfigurationInfo[]>([]);
let activeState = $state<CtrEngineMidiConfiguration | null>(null);

function responseError(status: 500, data?: unknown) {
    reportGatewayError(status);
    return gatewayErrorByStatus(status, data === undefined ? undefined : String(data));
}

async function request<T>(run: () => Promise<T>) {
    try {
        return await run();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }
}

export function engineMidiConfigurations() {
    return configurationsState;
}

export function activeEngineMidiConfiguration() {
    return activeState;
}

export async function fetchEngineMidiConfigurations() {
    const [listResponse, activeResponse] = await request(() =>
        Promise.all([api.listEngineMidiConfigurations(), api.activeEngineMidiConfiguration()])
    );
    configurationsState = listResponse.data;
    activeState = activeResponse.data;
}

export async function refreshEngineMidiConfigurations() {
    const response = await request(() => api.refreshEngineMidiConfigurations());
    if (response.status !== 204) {
        throw responseError(response.status, response.data);
    }
    await fetchEngineMidiConfigurations();
}

export async function setActiveEngineMidiConfigurationName(name: string) {
    const response = await request(() => api.setActiveEngineMidiConfigurationName(name));
    if (response.status !== 204) {
        throw responseError(response.status, response.data);
    }
    if (activeState) {
        activeState = { ...activeState, name };
    }
}

export async function createNewActiveEngineMidiConfiguration() {
    const response = await request(() => api.newActiveEngineMidiConfiguration());
    if (response.status !== 204) {
        return;
    }
    await Promise.all([
        fetchEngineMidiConfigurations(),
        fetchEngineMidiAppControls(),
        fetchMidiInputs()
    ]);
}

export async function saveActiveEngineMidiConfiguration() {
    const response = await request(() => api.saveActiveEngineMidiConfiguration());
    if (response.status !== 200) {
        throw responseError(response.status, response.data);
    }
    await fetchEngineMidiConfigurations();
    return response.data;
}

export async function loadActiveEngineMidiConfiguration(configurationId: string) {
    const response = await request(() => api.loadActiveEngineMidiConfiguration(configurationId));
    if (response.status !== 204) {
        throw responseError(response.status, response.data);
    }
    await Promise.all([
        fetchEngineMidiConfigurations(),
        fetchEngineMidiAppControls(),
        fetchMidiInputs()
    ]);
}

export async function deleteEngineMidiConfiguration(configurationId: string) {
    const response = await request(() => api.deleteEngineMidiConfiguration(configurationId));
    if (response.status !== 204) {
        throw responseError(response.status, response.data);
    }
    configurationsState = configurationsState.filter(
        (configuration) => configuration.id !== configurationId
    );
    if (activeState?.storage?.id === configurationId) {
        activeState = { ...activeState, storage: null };
    }
}

export async function exportActiveEngineMidiConfiguration() {
    const response = await request(() => api.exportActiveEngineMidiConfiguration());
    return response.data;
}

export async function downloadEngineMidiConfiguration(configurationId: string) {
    const response = await request(() => api.readEngineMidiConfiguration(configurationId));
    if (response.status !== 200) {
        const errorData: unknown = response.data;
        const message = errorData instanceof Blob ? await errorData.text() : String(errorData);
        throw responseError(response.status, message);
    }
    return response.data;
}

export async function importActiveEngineMidiConfiguration(
    configuration: StoredEngineMidiConfiguration
) {
    const response = await request(() => api.importActiveEngineMidiConfiguration(configuration));
    if (response.status !== 204) {
        throw responseError(response.status, response.data);
    }
    await Promise.all([
        fetchEngineMidiConfigurations(),
        fetchEngineMidiAppControls(),
        fetchMidiInputs()
    ]);
}

export async function addEngineMidiConfiguration(configuration: StoredEngineMidiConfiguration) {
    const response = await request(() => api.writeNewEngineMidiConfiguration(configuration));
    if (response.status !== 200) {
        throw responseError(response.status, response.data);
    }
    await fetchEngineMidiConfigurations();
    return response.data;
}
