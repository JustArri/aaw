import type {
    AudioHostOutputs,
    AudioOutputStreamInfo,
    AudioRecordingInfo,
    OutputStreamLatency,
    OutputStreamSampleRate
} from '$lib/generated/models';
import * as api from '$lib/generated/api';
import { gatewayErrorByStatus, gatewayNotResponding } from '$lib/error';
import { untrack } from 'svelte';
import { gatewayOnline, reportGatewayError, reportGatewayOffline } from './gateway.svelte';

let hostOutputsState = $state([] as AudioHostOutputs[]);
let outputStreamState = $state(null as AudioOutputStreamInfo | null);
let recordingState = $state(null as AudioRecordingInfo | null);
let fetchingOutputs = false;
let fetchingOutputStream = false;
let fetchingRecording = false;
let settingOutputStream = false;
let settingRecording = false;

export function audioHostOutputs() {
    return hostOutputsState;
}

export function audioOutputStream() {
    return outputStreamState;
}

export function audioRecording() {
    return recordingState;
}

export function autoUpdateAudio() {
    $effect(() => {
        if (!gatewayOnline()) {
            return;
        }
        void untrack(fetchAudioHostOutputs);
        void untrack(fetchAudioOutputStream);
        void untrack(fetchAudioRecording);
    });
}

export async function fetchAudioRecording() {
    if (fetchingRecording) {
        return;
    }
    fetchingRecording = true;

    let response;

    try {
        response = await api.audioRecording();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetchingRecording = false;
    }

    switch (response.status) {
        case 200:
            recordingState = response.data;
            break;
        default:
            reportGatewayError(response.status);
            throw gatewayErrorByStatus(response.status, response.data);
    }
}

async function updateAudioRecording(request: () => Promise<{ status: number; data: unknown }>) {
    if (settingRecording) {
        return;
    }
    settingRecording = true;

    let response;

    try {
        response = await request();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        settingRecording = false;
    }

    if (response.status !== 204) {
        reportGatewayError(response.status as 400 | 500);
        throw gatewayErrorByStatus(
            response.status as 400 | 500,
            typeof response.data === 'string' ? response.data : undefined
        );
    }

    await fetchAudioRecording();
}

export async function startAudioRecording() {
    await updateAudioRecording(() => api.startAudioRecording());
}

export async function setAudioRecordingPaused(paused: boolean) {
    await updateAudioRecording(() => api.setAudioRecordingPaused(paused));
}

export async function stopAudioRecording() {
    await updateAudioRecording(() => api.stopAudioRecording());
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
