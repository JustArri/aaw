import type { OutputStreamLatency, OutputStreamSampleRate } from '$lib/generated/models';
import type { LatencyMode, SampleRateMode } from '$lib/content/audio';
import { notify } from '$lib/push-notifications.svelte';
import {
    audioHostOutputs,
    audioOutputStream,
    fetchAudioHostOutputs,
    setAudioOutputStream
} from '$lib/state/audio.svelte';

export class AudioSection {
    selectedHostId = $state('');
    selectedOutputId = $state('');
    latencyMode = $state<LatencyMode>('Automatic');
    customBufferSize = $state(256);
    sampleRateMode = $state<SampleRateMode>('Automatic');
    sampleRate = $state(48_000);
    applying = $state(false);
    refreshingOutputs = $state(false);
    outputsRefreshed = $state(false);
    hosts = $derived(audioHostOutputs());
    stream = $derived(audioOutputStream());
    selectedHost = $derived(this.hosts.find((entry) => entry.host.id === this.selectedHostId));
    selectedOutput = $derived(
        this.selectedHost?.outputs.find((output) => output.id === this.selectedOutputId)
    );
    activeHost = $derived(this.hosts.find((entry) => entry.host.id === this.stream?.host_id));
    activeOutput = $derived(
        this.activeHost?.outputs.find((output) => output.id === this.stream?.output_id)
    );

    constructor() {
        $effect(() => {
            if (this.hosts.length === 0) {
                this.selectedHostId = '';
                this.selectedOutputId = '';
            } else if (!this.hosts.some((entry) => entry.host.id === this.selectedHostId)) {
                const streamHost = this.hosts.find(
                    (entry) => entry.host.id === this.stream?.host_id
                );
                this.selectedHostId = streamHost?.host.id ?? this.hosts[0].host.id;
            }
        });
        $effect(() => {
            const outputs = this.selectedHost?.outputs ?? [];
            if (outputs.length === 0) {
                this.selectedOutputId = '';
            } else if (!outputs.some((output) => output.id === this.selectedOutputId)) {
                const streamOutput = outputs.find((output) => output.id === this.stream?.output_id);
                const defaultOutput = outputs.find((output) => output.is_default);
                this.selectedOutputId = streamOutput?.id ?? defaultOutput?.id ?? outputs[0].id;
            }
        });
    }

    private selectedLatency(): OutputStreamLatency {
        return this.latencyMode === 'CustomBufferSize'
            ? {
                  type: 'CustomBufferSize',
                  data: { value: Math.max(1, Math.round(this.customBufferSize)) }
              }
            : { type: this.latencyMode };
    }

    private selectedSampleRate(): OutputStreamSampleRate {
        return this.sampleRateMode === 'Automatic'
            ? { type: 'Automatic' }
            : { type: 'Fixed', data: { value: Math.max(1, Math.round(this.sampleRate)) } };
    }

    async applyOutput() {
        if (!this.selectedHostId || !this.selectedOutputId || this.applying) return;
        this.applying = true;
        try {
            await setAudioOutputStream(
                this.selectedHostId,
                this.selectedLatency(),
                this.selectedOutputId,
                this.selectedSampleRate()
            );
        } catch (error) {
            notify.error(
                error instanceof Error ? error.message : 'Could not apply the audio output.',
                { title: 'Apply audio output' }
            );
        } finally {
            this.applying = false;
        }
    }

    async refreshOutputs() {
        if (this.refreshingOutputs) return;
        this.refreshingOutputs = true;
        this.outputsRefreshed = false;
        try {
            await fetchAudioHostOutputs();
            this.outputsRefreshed = true;
            setTimeout(() => (this.outputsRefreshed = false), 1200);
        } catch (error) {
            notify.error(
                error instanceof Error ? error.message : 'Could not refresh audio outputs.',
                { title: 'Refresh audio outputs' }
            );
        } finally {
            this.refreshingOutputs = false;
        }
    }
}
