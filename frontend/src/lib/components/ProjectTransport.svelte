<script lang="ts">
    import Icon from '@iconify/svelte';
    import * as api from '$lib/generated/api';
    import type { MusicalTransport } from '$lib/generated/models';
    import { errorToUserMessage } from '$lib/error';
    import { notify } from '$lib/push-notifications.svelte';
    import { activeProject, updateActiveProjectTransport } from '$lib/state/active-project.svelte';
    import Button from './Button.svelte';
    import NumericControl from './NumericControl.svelte';
    import SegmentedControl from './SegmentedControl.svelte';
    import TransportBeatVisualizer from './TransportBeatVisualizer.svelte';

    let {
        class: className = '',
        title = 'Project transport',
        subtitle = 'Quarter-note tempo · shared clock',
        showBeatVisualizer = false,
        showConfiguration = true,
        onchange
    }: {
        class?: string;
        title?: string;
        subtitle?: string;
        showBeatVisualizer?: boolean;
        showConfiguration?: boolean;
        onchange?: (transport: MusicalTransport) => void | Promise<void>;
    } = $props();

    const fallback: MusicalTransport = {
        tempo_bpm: 120,
        beats_per_bar: 4,
        beat_unit: 4,
        playing: false
    };
    let transport = $derived(activeProject()?.transport ?? fallback);
    let sending = false;
    let queued: MusicalTransport | null = null;

    async function update(update: Partial<MusicalTransport>) {
        const previous = transport;
        const next = { ...transport, ...update };
        updateActiveProjectTransport(next);
        queued = next;
        if (sending) return;

        sending = true;
        try {
            while (queued) {
                const candidate = queued;
                queued = null;
                const response = await api.setTransport(candidate);
                if (response.status !== 204) {
                    throw new Error(`Transport update failed (${response.status})`);
                }
                await onchange?.(candidate);
            }
        } catch (cause) {
            queued = null;
            updateActiveProjectTransport(previous);
            notify.error(errorToUserMessage(cause), { title: 'Update transport' });
        } finally {
            sending = false;
        }
    }
</script>

<section
    class={`border-border bg-surface-card grid gap-3 rounded-xl border p-3 ${className}`}
    aria-label="Beat">
    <div class="flex items-center justify-between gap-3">
        <div class={showBeatVisualizer ? 'shrink-0' : 'mr-auto'}>
            <h3 class="text-foreground m-0 text-sm font-semibold whitespace-nowrap">
                {title}
            </h3>
            {#if subtitle}
                <p
                    class={`text-foreground-muted m-0 text-xs ${showBeatVisualizer ? 'hidden min-[31.25rem]:block' : ''}`}>
                    {subtitle}
                </p>
            {/if}
        </div>
        {#if showBeatVisualizer}
            <TransportBeatVisualizer {transport} />
        {/if}
        <Button
            variant={transport.playing ? 'danger-soft' : 'primary'}
            size="md"
            class="size-8! min-h-8! px-0! min-[31.25rem]:h-auto! min-[31.25rem]:min-h-10! min-[31.25rem]:w-auto! min-[31.25rem]:px-3.5!"
            aria-label={transport.playing ? 'Stop transport' : 'Start transport'}
            title={transport.playing ? 'Stop transport' : 'Start transport'}
            onclick={() => update({ playing: !transport.playing })}>
            <Icon
                icon={transport.playing ? 'lucide:square' : 'lucide:play'}
                class="size-5! min-[31.25rem]:size-4!" />
            <span class="hidden min-[31.25rem]:inline">
                {transport.playing ? 'Stop' : 'Play'}
            </span>
        </Button>
    </div>
    {#if showConfiguration}
        <div class="grid gap-3 min-[37.5rem]:grid-cols-2 min-[56.25rem]:grid-cols-3">
            <NumericControl
                compact
                label="Tempo"
                value={transport.tempo_bpm}
                unit="BPM"
                smallStep={0.1}
                largeStep={10}
                resetValue={120}
                min={20}
                max={400}
                precision={1}
                onchange={async (value) => {
                    await update({ tempo_bpm: value });
                    return transport.tempo_bpm;
                }} />
            <NumericControl
                compact
                label="Beats per bar"
                value={transport.beats_per_bar}
                unit="beats"
                smallStep={1}
                largeStep={4}
                resetValue={4}
                min={1}
                max={32}
                onchange={async (value) => {
                    await update({ beats_per_bar: value });
                    return transport.beats_per_bar;
                }} />
            <section>
                <header class="mb-1.5 flex min-h-8 items-center justify-between gap-2">
                    <span class="text-foreground-muted text-sm">Beat unit</span>
                    <span
                        class="border-border-strong bg-surface-control text-foreground flex h-9 w-37 items-center rounded-lg border px-2.5 font-mono text-sm font-semibold">
                        1/{transport.beat_unit}
                    </span>
                </header>
                <SegmentedControl
                    compact
                    name="transport-beat-unit"
                    ariaLabel="Beat unit"
                    value={String(transport.beat_unit)}
                    options={[1, 2, 4, 8, 16].map((unit) => ({
                        value: String(unit),
                        label: String(unit)
                    }))}
                    onchange={(value) => update({ beat_unit: Number(value) })} />
            </section>
        </div>
    {/if}
</section>
