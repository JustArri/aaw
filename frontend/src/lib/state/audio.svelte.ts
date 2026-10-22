import type {
    AudioHostOutputs,
    AudioOutputStreamInfo,
    OutputStreamLatency,
    OutputStreamSampleRate
} from '$lib/generated/models';
import * as api from '$lib/generated/api';
import { gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import { untrack } from 'svelte';
import { gatewayOnline, reportGatewayError, reportGatewayOffline } from './gateway.svelte';

let hostOutputsState = $state([] as AudioHostOutputs[]);
let outputStreamState = $state(null as AudioOutputStreamInfo | null);
let fetchingOutputs = false;
let fetchingOutputStream = false;
let settingOutputStream = false;

export function audioHostOutputs() {
    return hostOutputsState;
}

export function audioOutputStream() {
    return outputStreamState;
}

export function autoUpdateAudio() {
    $effect(() => {
        if (!gatewayOnline()) {
            return;
        }
        void untrack(fetchAudioHostOutputs);
        void untrack(fetchAudioOutputStream);
    });
}

export function resetAudioHostOutputs() {
    hostOutputsState = [];
}

export async function fetchAudioHostOutputs() {
    if (fetchingOutputs) {
        return;
    }
    fetchingOutputs = true;

    let rsp;

    try {
        rsp = await api.audioHostOutputs();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetchingOutputs = false;
    }

    switch (rsp.status) {
        case 200:
            hostOutputsState = rsp.data;
            break;
        default:
            hostOutputsState = [];
            reportGatewayError(rsp.status);
            throw gatewayErrorByStatus(rsp.status);
    }
}

export async function fetchAudioOutputStream() {
    if (fetchingOutputStream) {
        return;
    }
    fetchingOutputStream = true;

    let rsp;

    try {
        rsp = await api.audioOutputStream();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetchingOutputStream = false;
    }

    switch (rsp.status) {
        case 200:
            outputStreamState = rsp.data;
            break;
    }
}

export async function setAudioOutputStream(
    host_id: string,
    latency: OutputStreamLatency,
    output_id: string,
    sample_rate: OutputStreamSampleRate
) {
    if (settingOutputStream) {
        return;
    }
    settingOutputStream = true;

    let rsp;

    try {
        rsp = await api.setAudioOutputStream({ host_id, latency, output_id, sample_rate });
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        settingOutputStream = false;
    }

    switch (rsp.status) {
        case 200:
            outputStreamState = rsp.data;
            break;
        default:
            outputStreamState = null;
            reportGatewayError(rsp.status);
            throw gatewayErrorByStatus(rsp.status);
    }
}
