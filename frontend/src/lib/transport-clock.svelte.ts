import { onDestroy, onMount } from 'svelte';
import { activeProject } from '$lib/state/active-project.svelte';
import { audioOutputStream } from '$lib/state/audio.svelte';

const ticksPerQuarterNote = 960;
type TransportPositionListener = (tick: number, receivedAt: number) => void;

const transportPositionListeners = new Set<TransportPositionListener>();

export function updateTransportPosition(tick: number) {
    const receivedAt = performance.now();
    for (const listener of transportPositionListeners) listener(tick, receivedAt);
}

export class TransportClock {
    tick = $state(0);
    synchronized = $state(false);

    private anchorTick = 0;
    private anchorTime = 0;
    private previousPlaying = false;
    private previousTempo = 120;
    private animationFrame: number | undefined;
    private readonly positionListener: TransportPositionListener = (tick, receivedAt) => {
        if (!activeProject()?.transport.playing) return;
        this.anchorTick = tick;
        this.anchorTime = receivedAt;
        this.synchronized = true;
    };

    constructor() {
        onMount(() => {
            this.anchorTime = performance.now();
            transportPositionListeners.add(this.positionListener);
            this.animationFrame = requestAnimationFrame((now) => this.update(now));
        });
        onDestroy(() => {
            if (this.animationFrame !== undefined) cancelAnimationFrame(this.animationFrame);
            transportPositionListeners.delete(this.positionListener);
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
}
