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

    let {
        class: className = '',
        onchange
    }: { class?: string; onchange?: (transport: MusicalTransport) => void | Promise<void> } =
        $props();

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
    aria-label="Project transport">
    <div class="flex items-center gap-2">
        <div class="mr-auto">
            <h3 class="text-foreground m-0 text-sm font-semibold">Project transport</h3>
            <p class="text-foreground-muted m-0 text-xs">Quarter-note tempo · shared clock</p>
        </div>
        <Button
            variant={transport.playing ? 'danger-soft' : 'primary'}
            onclick={() => update({ playing: !transport.playing })}>
            <Icon icon={transport.playing ? 'lucide:square' : 'lucide:play'} width="1rem" />
            {transport.playing ? 'Stop' : 'Play'}
        </Button>
    </div>
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
</section>
