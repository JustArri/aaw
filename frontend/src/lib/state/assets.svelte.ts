import { gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type { Assets } from '$lib/generated/models';
import { SynthKind, type SynthKind as SynthKindType } from './synths.svelte';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';

let assetsState = $state<Assets>({ fluid: [], sfizz: [], clap_instruments: [] });
let fetching = false;
let refreshing = false;

export function assets() {
    return assetsState;
}

export function assetsForSynth(kind: SynthKindType) {
    switch (kind) {
        case SynthKind.Dummy:
            return [];
        case SynthKind.Fluid:
            return assetsState.fluid;
        case SynthKind.Sfizz:
            return assetsState.sfizz;
        case SynthKind.Clap:
            return [];
    }
}

export function clapPlugins() {
    return assetsState.clap_instruments;
}

export function resetAssets() {
    assetsState = { fluid: [], sfizz: [], clap_instruments: [] };
}

export async function fetchAssets() {
    if (fetching) {
        return;
    }
    fetching = true;

    let response;

    try {
        response = await api.assets();
    } catch (error) {
        resetAssets();
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetching = false;
    }

    switch (response.status) {
        case 200:
            assetsState = response.data;
            break;
        case 500:
            resetAssets();
            reportGatewayError(response.status);
            throw gatewayErrorByStatus(response.status);
    }
}

export async function refreshAssets() {
    if (refreshing) {
        return;
    }
    refreshing = true;

    let response;

    try {
        response = await api.refreshAssets();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        refreshing = false;
    }

    switch (response.status) {
        case 204:
            await fetchAssets();
            break;
        case 500:
            reportGatewayError(response.status);
            throw gatewayErrorByStatus(response.status);
    }
}
