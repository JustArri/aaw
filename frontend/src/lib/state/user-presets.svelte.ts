import { gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type { StoredUserPresets } from '$lib/generated/models';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';

export type EditableUserPreset = { name: string; instruments: number[] };
export type EditableUserPresets = { version: number; entries: EditableUserPreset[] };

const EMPTY_USER_PRESETS: EditableUserPresets = { version: 1, entries: [] };

function fromStored(presets: StoredUserPresets): EditableUserPresets {
    return {
        version: presets.version,
        entries: presets.entries.map((preset) => ({
            name: preset.name,
            instruments: preset.controllers.map(
                (address) => address.instrument_id * 65_536 + address.controller_id
            )
        }))
    };
}

function toStored(presets: EditableUserPresets): StoredUserPresets {
    return {
        version: presets.version,
        entries: presets.entries.map((preset) => ({
            name: preset.name,
            controllers: preset.instruments.map((id) => ({
                instrument_id: Math.floor(id / 65_536),
                controller_id: id % 65_536
            }))
        }))
    };
}

export function cloneUserPresets(presets: EditableUserPresets): EditableUserPresets {
    return {
        version: presets.version,
        entries: presets.entries.map((preset) => ({
            name: preset.name,
            instruments: [...preset.instruments]
        }))
    };
}

let userPresetsState = $state<EditableUserPresets>(cloneUserPresets(EMPTY_USER_PRESETS));
export const userPresetsDraft = $state<EditableUserPresets>(cloneUserPresets(EMPTY_USER_PRESETS));
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

function replaceDraft(presets: EditableUserPresets) {
    const copy = cloneUserPresets(presets);
    userPresetsDraft.version = copy.version;
    userPresetsDraft.entries = copy.entries;
}

export function initializeUserPresetsDraft(presets: EditableUserPresets) {
    if (!draftInitialized) {
        replaceDraft(presets);
        draftInitialized = true;
    }
}

export function resetUserPresetsDraft(presets: EditableUserPresets = userPresetsState) {
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
        userPresetsState = fromStored((await request(api.userPresets)).data);
        return userPresetsState;
    } finally {
        fetching = false;
    }
}

export async function saveUserPresets(presets: EditableUserPresets) {
    const response = await request(() => api.setUserPresets(toStored(presets)));
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
