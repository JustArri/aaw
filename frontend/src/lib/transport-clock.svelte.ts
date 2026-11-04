import { onDestroy, onMount } from 'svelte';
import * as api from '$lib/generated/api';
import { activeProject } from '$lib/state/active-project.svelte';
import { audioOutputStream } from '$lib/state/audio.svelte';

const ticksPerQuarterNote = 960;

export class TransportClock {
    tick = $state(0);
    synchronized = $state(false);

    private anchorTick = 0;
    private anchorTime = 0;
    private previousPlaying = false;
    private previousTempo = 120;
    private syncing = false;
    private animationFrame: number | undefined;
    private syncInterval: ReturnType<typeof setInterval> | undefined;

    constructor() {
        onMount(() => {
            this.anchorTime = performance.now();
            this.animationFrame = requestAnimationFrame((now) => this.update(now));
            this.syncInterval = setInterval(() => void this.sync(), 500);
        });
        onDestroy(() => {
            if (this.animationFrame !== undefined) cancelAnimationFrame(this.animationFrame);
            clearInterval(this.syncInterval);
        });
    }

    private estimate(now: number, tempo: number) {
        const output = audioOutputStream();
        const outputLatencyTicks = output
            ? (output.buffer_size / output.sample_rate / 60) * tempo * ticksPerQuarterNote
            : 0;
        return Math.max(
            0,
            this.anchorTick +
                ((now - this.anchorTime) / 60_000) * tempo * ticksPerQuarterNote -
                outputLatencyTicks
        );
    }

    private update(now: number) {
        const transport = activeProject()?.transport;
        if (transport?.playing) {
            if (!this.previousPlaying) {
                this.anchorTick = 0;
                this.anchorTime = now;
                this.synchronized = false;
                void this.sync();
            } else if (transport.tempo_bpm !== this.previousTempo) {
                this.anchorTick = this.estimate(now, this.previousTempo);
                this.anchorTime = now;
            }
            this.tick = this.estimate(now, transport.tempo_bpm);
        } else {
            this.tick = 0;
            this.synchronized = false;
        }
        this.previousPlaying = transport?.playing ?? false;
        this.previousTempo = transport?.tempo_bpm ?? this.previousTempo;
        this.animationFrame = requestAnimationFrame((nextNow) => this.update(nextNow));
    }

    private async sync() {
        if (!activeProject()?.transport.playing || this.syncing) return;
        this.syncing = true;
        const requestStarted = performance.now();
        try {
            const response = await api.transportPosition();
            const responseReceived = performance.now();
            if (response.status === 200) {
                this.anchorTick = response.data;
                this.anchorTime = (requestStarted + responseReceived) / 2;
                this.synchronized = true;
            }
        } catch {
            // Retry while continuing from the last synchronized position.
        } finally {
            this.syncing = false;
        }
    }
}
