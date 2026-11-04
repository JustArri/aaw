<script lang="ts">
    import Icon from '@iconify/svelte';
    import { icons } from '$lib/icons';
    import type { OverviewSection } from '$lib/sections/overview.svelte';

    interface Props {
        section: OverviewSection;
    }

    let { section }: Props = $props();
</script>

<section
    class="border-border bg-surface-card order-4 overflow-hidden rounded-xl border min-[37.5rem]:order-0 min-[37.5rem]:col-start-1 min-[37.5rem]:row-start-3">
    <header
        class="border-border flex min-h-13.5 items-center justify-between border-b px-3.25 min-[56.25rem]:min-h-14.75 min-[56.25rem]:px-3.75">
        <span class="block text-sm font-bold">Instruments</span>
        <b
            class="bg-surface-badge text-foreground-strong grid h-5.75 min-w-0! place-items-center rounded-md px-1.5 text-center font-sans! text-xs leading-none font-semibold tabular-nums min-[56.25rem]:text-sm">
            <span>{section.enabledControllerCount} / {section.instruments.length}</span>
        </b>
    </header>
    {#if section.loading}
        <div class="text-foreground-muted grid min-h-24 place-items-center text-sm">
            Loading instruments…
        </div>
    {:else if section.instruments.length === 0}
        <div class="text-foreground-muted grid min-h-24 place-items-center p-4 text-sm">
            No instruments configured.
        </div>
    {:else}
        <div class="bg-border grid gap-px">
            {#each section.instruments as instrument (instrument.id)}
                {@const assignedSynth = section.synthEntry(instrument.instrument_id)}
                {@const programName = section.soundFontProgramName(assignedSynth)}
                <button
                    class={`grid min-h-14.5 cursor-pointer grid-cols-[0.1875rem_minmax(0,1fr)_2.125rem] items-center gap-2.5 border-0 px-2.5 py-1.75 text-left transition-[filter] enabled:hover:brightness-110 disabled:cursor-wait min-[56.25rem]:min-h-15.5 min-[56.25rem]:px-3.25 ${instrument.controller.data.enabled ? 'bg-surface-item-active' : 'bg-surface-disabled'}`}
                    type="button"
                    aria-pressed={instrument.controller.data.enabled}
                    disabled={section.togglingInstrument === instrument.id}
                    onclick={() =>
                        void section.toggleInstrument(
                            instrument.id,
                            !instrument.controller.data.enabled
                        )}>
                    <i
                        class={`h-7.25 rounded ${instrument.controller.data.enabled ? '' : 'opacity-35 grayscale'}`}
                        style:background={section.synthColor(instrument.instrument_id)}>
                    </i>
                    <span class="min-w-0">
                        <strong
                            class={`block overflow-hidden text-sm font-semibold text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm ${instrument.controller.data.enabled ? 'text-foreground' : 'text-foreground-disabled'}`}>
                            {section.controllerName(instrument)}
                        </strong>
                        <small
                            class="text-foreground-caption mt-1 block overflow-hidden text-xs text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm">
                            {section.synthName(assignedSynth)}
                            {#if programName}
                                <span class="text-foreground-muted">· {programName}</span>
                            {/if}
                        </small>
                    </span>
                    <b
                        class={`grid h-8 place-items-center rounded-lg border ${instrument.controller.data.enabled ? 'border-accent/30 bg-accent/10 text-accent' : 'border-border-strong text-foreground-off'}`}>
                        <Icon
                            icon={section.togglingInstrument === instrument.id
                                ? 'lucide:loader-circle'
                                : icons.power}
                            width="0.9375rem"
                            class={section.togglingInstrument === instrument.id
                                ? 'animate-spin'
                                : ''} />
                    </b>
                </button>
            {/each}
        </div>
    {/if}
</section>
