<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { OverviewSection } from '$lib/sections/overview.svelte';

    interface Props {
        section: OverviewSection;
    }

    let { section }: Props = $props();
</script>

<section
    class="border-border bg-surface-card order-1 overflow-hidden rounded-xl border min-[37.5rem]:order-0 min-[37.5rem]:col-span-2 min-[37.5rem]:row-start-1">
    <header
        class="border-border flex min-h-13.5 items-center justify-between border-b px-3.25 min-[56.25rem]:min-h-14.75 min-[56.25rem]:px-3.75">
        <span class="block text-sm font-bold">User presets</span>
        <div class="flex items-center gap-2">
            <b
                class="bg-surface-badge text-foreground-strong grid h-5.75 min-w-0! place-items-center rounded-md px-1.5 text-center font-sans! text-xs leading-none font-semibold tabular-nums min-[56.25rem]:text-sm">
                <span>{section.presets.entries.length}</span>
            </b>
            <button
                class="border-border-strong bg-surface-button text-foreground hover:bg-surface-hover grid size-8.5 cursor-pointer place-items-center rounded-lg border transition-colors disabled:cursor-not-allowed disabled:opacity-50"
                type="button"
                aria-label="Create preset from current state"
                title="Create preset from current state"
                disabled={section.loading ||
                    section.creatingPreset ||
                    section.activatingPreset !== null ||
                    section.togglingInstrument !== null}
                onclick={() => void section.createPreset()}>
                <Icon
                    icon={section.creatingPreset ? 'lucide:loader-circle' : 'lucide:plus'}
                    width="1rem"
                    class={section.creatingPreset ? 'animate-spin' : ''} />
            </button>
        </div>
    </header>
    {#if section.loading}
        <div class="text-foreground-muted grid min-h-24 place-items-center text-sm">
            Loading presets…
        </div>
    {:else if section.presets.entries.length === 0}
        <div class="text-foreground-muted grid min-h-24 place-items-center p-4 text-sm">
            No user presets configured.
        </div>
    {:else}
        <div class="grid grid-cols-2 gap-1.75 p-2.25 min-[37.5rem]:grid-cols-4">
            {#each section.presets.entries as preset, index}
                <button
                    class="bg-surface-item border-border-strong hover:border-accent/30 relative grid min-h-18.75 cursor-pointer grid-cols-[1.375rem_minmax(0,1fr)] content-center items-start gap-1.75 rounded-lg border px-2.25 py-2.75 text-left transition-colors disabled:cursor-wait"
                    type="button"
                    disabled={section.activatingPreset !== null ||
                        section.togglingInstrument !== null ||
                        section.creatingPreset}
                    onclick={() => void section.applyPreset(index)}>
                    {#if section.activationAnimationPreset === index}
                        <span
                            use:section.activationGlow
                            class="border-accent bg-accent/12 pointer-events-none absolute -inset-px z-10 rounded-lg border-2"
                            aria-hidden="true">
                        </span>
                    {/if}
                    <small class="text-foreground-index font-mono text-xs min-[56.25rem]:text-sm">
                        {String(index + 1).padStart(2, '0')}
                    </small>
                    <span class="min-w-0">
                        <strong
                            class={`block overflow-hidden text-sm text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm ${preset.name.trim() ? 'text-foreground' : 'text-foreground-muted italic'}`}>
                            {section.presetName(index, preset.name)}
                        </strong>
                        <em
                            class="text-foreground-muted mt-1 block overflow-hidden text-xs text-ellipsis whitespace-nowrap not-italic min-[56.25rem]:text-sm">
                            {preset.actions.length}
                            {preset.actions.length === 1 ? 'action' : 'actions'}
                        </em>
                    </span>
                    {#if section.activatingPreset === index}
                        <Icon
                            icon="lucide:rotate-cw"
                            width="0.9375rem"
                            class="text-accent absolute right-2 bottom-2 animate-spin" />
                    {/if}
                </button>
            {/each}
        </div>
    {/if}
</section>
