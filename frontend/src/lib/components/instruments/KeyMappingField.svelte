<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { InstrumentsSection } from '$lib/sections/instruments.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

{#if section.keySection.mappingType === 'Custom'}
    <div class="grid grid-cols-[minmax(0,1fr)_2.5rem] items-end gap-2">
        <label class="grid min-w-0 gap-1 text-sm">
            <span class="text-foreground-muted">Input:output pairs</span>
            <input
                class="border-border-strong bg-surface-inset h-10 min-w-0 rounded-lg border px-2.5 font-mono"
                placeholder="36:48, 38:50"
                bind:value={section.keySection.mapping} />
        </label>
        <button
            class={`grid size-10 cursor-pointer place-items-center rounded-lg border transition-colors ${section.keySection.recordingMapping ? 'border-danger/40 bg-danger/15 text-danger animate-pulse' : 'border-border-strong bg-surface-button text-foreground hover:bg-surface-hover'}`}
            type="button"
            aria-label={section.keySection.recordingMapping
                ? 'Stop recording key mappings'
                : 'Record key mappings'}
            title={section.keySection.recordingMapping
                ? section.keySection.firstMappingKey === null
                    ? 'Waiting for an input key'
                    : 'Waiting for its output key'
                : 'Record input and output key pairs'}
            aria-pressed={section.keySection.recordingMapping}
            onclick={() => section.keySection.toggleMappingRecording()}>
            <Icon
                icon={section.keySection.recordingMapping
                    ? 'lucide:radio-tower'
                    : 'lucide:circle-dot'}
                width="1rem" />
        </button>
        {#if section.keySection.recordingMapping}
            <small
                class="text-foreground-muted col-span-2 flex items-center gap-2 text-xs"
                aria-live="polite">
                <span class="text-accent font-mono font-semibold tabular-nums">
                    {section.keySection.firstMappingKey === null
                        ? '… → …'
                        : `${section.keySection.firstMappingKey} → …`}
                </span>
                <span>
                    {section.keySection.firstMappingKey === null
                        ? 'Play the input key'
                        : 'Now play the output key'}
                </span>
            </small>
        {/if}
    </div>
{/if}
