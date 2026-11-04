<script lang="ts">
    import type { MusicalTransport } from '$lib/generated/models';
    import { TransportClock } from '$lib/transport-clock.svelte';

    interface Props {
        transport: MusicalTransport;
    }

    let { transport }: Props = $props();

    const clock = new TransportClock();

    let beatCount = $derived(Math.max(1, Math.round(transport.beats_per_bar)));
    let ticksPerBeat = $derived((960 * 4) / transport.beat_unit);
    let activeBeat = $derived(
        transport.playing && clock.synchronized
            ? Math.floor(clock.tick / ticksPerBeat) % beatCount
            : -1
    );
</script>

<div
    class="flex max-w-32 min-w-12 flex-1 items-center"
    role="meter"
    aria-label={transport.playing
        ? `Beat ${activeBeat + 1} of ${beatCount}`
        : `Transport stopped · ${beatCount} beats per bar`}
    aria-valuemin="1"
    aria-valuemax={beatCount}
    aria-valuenow={transport.playing ? activeBeat + 1 : undefined}
    title={transport.playing ? `Beat ${activeBeat + 1} of ${beatCount}` : 'Transport stopped'}>
    <div
        class="grid h-3 w-full items-center gap-px"
        style:grid-template-columns={`repeat(${beatCount}, minmax(0, 1fr))`}>
        {#each Array.from({ length: beatCount }).keys() as beat}
            <span
                class={`rounded-full transition-[height,background-color] duration-75 ${beat === activeBeat ? 'bg-accent h-3' : beat < activeBeat ? 'bg-accent/35 h-1.5' : beat === 0 ? 'bg-foreground-muted/55 h-2' : 'bg-border-strong h-1.5'}`}>
            </span>
        {/each}
    </div>
</div>
