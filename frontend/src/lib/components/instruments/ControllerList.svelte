<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { InstrumentsSection } from '$lib/sections/instruments.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card min-w-0 overflow-hidden rounded-xl border">
    <header class="border-border flex min-h-14.5 items-center justify-between border-b px-3.75">
        <div class="flex items-baseline gap-2">
            <h2 class="m-0 text-sm font-semibold">Controllers</h2>
            <span class="text-foreground-muted text-xs">
                {section.entries.filter(({ slot }) => slot.enabled).length} enabled · {section
                    .entries.length} total
            </span>
        </div>
        <Button
            variant="ghost"
            size="sm"
            square
            class="size-8.5"
            type="button"
            aria-label="Reload controllers"
            busy={section.reloading}
            disabled={section.reloading}
            onclick={() => void section.reloadInstruments()}>
            <Icon
                icon={section.loading || section.reloading
                    ? 'lucide:loader-circle'
                    : section.reloadDone
                      ? 'lucide:check'
                      : 'lucide:refresh-cw'}
                width="1.0625rem"
                class={section.loading || section.reloading
                    ? 'animate-spin'
                    : section.reloadDone
                      ? 'text-success'
                      : ''} />
        </Button>
    </header>

    {#if section.loading}
        <div class="text-foreground-muted grid min-h-52 place-items-center text-sm">
            Loading controllers…
        </div>
    {:else if section.entries.length === 0}
        <div class="text-foreground-muted grid min-h-52 place-items-center p-6 text-center">
            <div>
                <Icon icon="lucide:sliders-horizontal" width="1.5rem" class="mx-auto mb-2" />
                <p class="m-0 text-sm">No controllers are configured.</p>
            </div>
        </div>
    {:else}
        <div class="grid gap-1.5 p-1.5">
            {#each section.entries as entry (entry.id)}
                {@const assignedSynth = section.synthEntry(entry.slot.synth_id)}
                <article
                    class={`bg-surface-item grid overflow-hidden rounded-lg border transition-colors ${entry.slot.synth_id === null ? 'grid-cols-[minmax(0,1fr)_auto]' : 'grid-cols-[minmax(0,1fr)_auto_auto] min-[37.5rem]:grid-cols-[minmax(0,1fr)_minmax(7.5rem,11rem)_auto]'} ${entry.id === section.selectedId ? 'border-accent/45' : 'border-transparent'}`}>
                    <button
                        class="hover:bg-surface-hover grid min-w-0 cursor-pointer grid-cols-[auto_minmax(0,1fr)] items-center gap-3 border-0 bg-transparent px-3 py-3 text-left transition-colors"
                        type="button"
                        onclick={() => section.selectInstrument(entry.id)}>
                        <span
                            class="text-foreground-muted grid size-8 place-items-center rounded-lg font-mono text-sm"
                            style:background={`${section.synthColor(entry.slot.synth_id)}18`}>
                            {entry.slot.instrument_id + 1}.{entry.slot.controller_id + 1}
                        </span>
                        <span class="min-w-0">
                            <strong class="text-foreground block truncate text-sm font-semibold">
                                {section.displayName(entry.slot)}
                            </strong>
                            <small
                                class={`text-foreground-muted mt-0.5 block truncate text-sm ${entry.slot.synth_id === null ? '' : 'min-[37.5rem]:hidden'}`}>
                                {section.synthName(assignedSynth)}
                            </small>
                        </span>
                    </button>
                    {#if entry.slot.synth_id !== null}
                        <a
                            class="text-foreground-muted hover:border-accent/35 hover:bg-surface-hover hover:text-accent min-[37.5rem]:border-border my-auto grid size-9 place-items-center rounded-lg border border-transparent transition-colors min-[37.5rem]:my-0 min-[37.5rem]:flex min-[37.5rem]:size-auto min-[37.5rem]:min-w-0 min-[37.5rem]:items-center min-[37.5rem]:gap-2 min-[37.5rem]:rounded-none min-[37.5rem]:border-0 min-[37.5rem]:border-l min-[37.5rem]:px-3"
                            href={`/instruments?instrument=${entry.slot.synth_id}`}
                            aria-label={`Open ${section.synthName(assignedSynth)}`}
                            title="Open assigned synth">
                            <span
                                class="hidden min-w-0 flex-1 truncate text-xs min-[37.5rem]:block">
                                {section.synthName(assignedSynth)}
                            </span>
                            <Icon class="shrink-0" icon="lucide:arrow-up-right" width="1rem" />
                        </a>
                    {/if}
                    <button
                        class={`mr-2 grid size-9 cursor-pointer place-items-center self-center rounded-lg border transition-colors ${entry.slot.enabled ? 'border-accent/60 bg-accent/20 text-accent' : 'border-border-strong bg-surface-button text-foreground-muted hover:border-accent/35 hover:text-foreground'}`}
                        type="button"
                        aria-label={`${entry.slot.enabled ? 'Disable' : 'Enable'} ${section.displayName(entry.slot)}`}
                        disabled={section.togglingId !== null}
                        onclick={() => void section.toggleEnabled(entry.id, !entry.slot.enabled)}>
                        <Icon
                            icon={section.togglingId === entry.id
                                ? 'lucide:loader-circle'
                                : 'lucide:power'}
                            width="1rem"
                            class={section.togglingId === entry.id ? 'animate-spin' : ''} />
                    </button>
                </article>
            {/each}
        </div>
    {/if}
</section>
