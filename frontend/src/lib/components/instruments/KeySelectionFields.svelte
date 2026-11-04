<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { InstrumentsSection } from '$lib/sections/instruments.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

{#if section.keySection.selectionType === 'Range'}
    <div class="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)_2.5rem] items-end gap-2">
        <label class="grid gap-1 text-sm">
            <span class="text-foreground-muted">First key</span>
            <input
                class="border-border-strong bg-surface-inset h-10 rounded-lg border px-2.5 font-mono"
                type="number"
                min="0"
                max="127"
                bind:value={section.keySection.rangeMin} />
        </label>
        <label class="grid gap-1 text-sm">
            <span class="text-foreground-muted">Last key</span>
            <input
                class="border-border-strong bg-surface-inset h-10 rounded-lg border px-2.5 font-mono"
                type="number"
                min="0"
                max="127"
                bind:value={section.keySection.rangeMax} />
        </label>
        <button
            class={`grid size-10 cursor-pointer place-items-center rounded-lg border transition-colors ${section.keySection.recordingSelection === 'Range' ? 'border-danger/40 bg-danger/15 text-danger animate-pulse' : 'border-border-strong bg-surface-button text-foreground hover:bg-surface-hover'}`}
            type="button"
            aria-label={section.keySection.recordingSelection === 'Range'
                ? 'Stop recording key range'
                : 'Record key range'}
            title={section.keySection.recordingSelection === 'Range'
                ? section.keySection.firstRangeKey === null
                    ? 'Waiting for the first key'
                    : 'Waiting for the second key'
                : 'Record two keys to define the range'}
            aria-pressed={section.keySection.recordingSelection === 'Range'}
            onclick={() => section.keySection.toggleRecording('Range')}>
            <Icon
                icon={section.keySection.recordingSelection === 'Range'
                    ? 'lucide:radio-tower'
                    : 'lucide:circle-dot'}
                width="1rem" />
        </button>
    </div>
{:else if section.keySection.selectionType === 'Keys'}
    <div class="grid grid-cols-[minmax(0,1fr)_2.5rem] items-end gap-2">
        <label class="grid min-w-0 gap-1 text-sm">
            <span class="text-foreground-muted">MIDI key numbers</span>
            <input
                class="border-border-strong bg-surface-inset h-10 min-w-0 rounded-lg border px-2.5 font-mono"
                placeholder="36, 38, 42"
                bind:value={section.keySection.selectedKeys} />
        </label>
        <button
            class={`grid size-10 cursor-pointer place-items-center rounded-lg border transition-colors ${section.keySection.recordingSelection === 'Keys' ? 'border-danger/40 bg-danger/15 text-danger animate-pulse' : 'border-border-strong bg-surface-button text-foreground hover:bg-surface-hover'}`}
            type="button"
            aria-label={section.keySection.recordingSelection === 'Keys'
                ? 'Stop recording specific keys'
                : 'Record specific keys'}
            title={section.keySection.recordingSelection === 'Keys'
                ? 'Recording keys; press again to stop'
                : 'Record specific keys'}
            aria-pressed={section.keySection.recordingSelection === 'Keys'}
            onclick={() => section.keySection.toggleRecording('Keys')}>
            <Icon
                icon={section.keySection.recordingSelection === 'Keys'
                    ? 'lucide:radio-tower'
                    : 'lucide:circle-dot'}
                width="1rem" />
        </button>
    </div>
{/if}
