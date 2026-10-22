import { gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type { StoredUserPresets } from '$lib/generated/models';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';

const EMPTY_USER_PRESETS: StoredUserPresets = { version: 1, entries: [] };

export function cloneUserPresets(presets: StoredUserPresets): StoredUserPresets {
    return {
        version: presets.version,
        entries: presets.entries.map((preset) => ({
            name: preset.name,
            instruments: [...preset.instruments]
        }))
    };
}

let userPresetsState = $state<StoredUserPresets>(cloneUserPresets(EMPTY_USER_PRESETS));
export const userPresetsDraft = $state<StoredUserPresets>(cloneUserPresets(EMPTY_USER_PRESETS));
let fetching = false;
let draftInitialized = false;

export function userPresets() {
    return userPresetsState;
}

export function resetUserPresets() {
    userPresetsState = cloneUserPresets(EMPTY_USER_PRESETS);
    resetUserPresetsDraft(userPresetsState);
    draftInitialized = false;
}

function replaceDraft(presets: StoredUserPresets) {
    const copy = cloneUserPresets(presets);
    userPresetsDraft.version = copy.version;
    userPresetsDraft.entries = copy.entries;
}

export function initializeUserPresetsDraft(presets: StoredUserPresets) {
    if (draftInitialized) {
        return;
    }
    replaceDraft(presets);
    draftInitialized = true;
}

export function resetUserPresetsDraft(presets: StoredUserPresets = userPresetsState) {
    replaceDraft(presets);
    draftInitialized = true;
}

function responseError(status: number, data: unknown): never {
    if (status === 500) {
        reportGatewayError(status);
    }
    if (typeof data === 'string' && data.trim()) {
        throw new Error(data);
    }
    if (status === 404) {
        throw new Error('User preset does not exist');
    }
    throw new Error(`User preset operation failed (${status})`);
}

async function request<T>(operation: () => Promise<T>): Promise<T> {
    try {
        return await operation();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }
}

export async function fetchUserPresets() {
    if (fetching) {
        return userPresetsState;
    }
    fetching = true;
    try {
        const response = await request(api.userPresets);
        userPresetsState = response.data;
        return userPresetsState;
    } finally {
        fetching = false;
    }
}

export async function saveUserPresets(presets: StoredUserPresets) {
    const response = await request(() => api.setUserPresets(presets));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    userPresetsState = cloneUserPresets(presets);
}

export async function activateUserPreset(id: number) {
    const response = await request(() => api.activateUserPreset(id));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
}
