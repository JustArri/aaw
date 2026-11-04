<script lang="ts">
    import type { UserPresetsSection } from '$lib/sections/user-presets.svelte';
    import type { EditableUserPreset } from '$lib/state/user-presets.svelte';
    import PresetActions from './PresetActions.svelte';
    import PresetCardFooter from './PresetCardFooter.svelte';
    import PresetCardHeader from './PresetCardHeader.svelte';

    interface Props {
        section: UserPresetsSection;
        preset: EditableUserPreset;
        index: number;
    }

    let { section, preset, index }: Props = $props();
</script>

<article
    class={`bg-surface-inset-deep relative min-w-0 rounded-xl p-4 transition-opacity ${section.draggedPreset === preset ? 'opacity-40' : ''}`}
    data-preset-index={index}>
    {#if section.draggedPreset && section.dropInsertionIndex === index}
        <span
            class="bg-accent pointer-events-none absolute -top-1.5 right-2 left-2 z-10 h-1 rounded-full shadow-[0_0_0.5rem_var(--accent-glow)]">
        </span>
    {/if}
    {#if section.draggedPreset && section.dropInsertionIndex === section.draft.entries.length && index === section.draft.entries.length - 1}
        <span
            class="bg-accent pointer-events-none absolute right-2 -bottom-1.5 left-2 z-10 h-1 rounded-full shadow-[0_0_0.5rem_var(--accent-glow)]">
        </span>
    {/if}

    <PresetCardHeader {section} {preset} {index} />
    <PresetActions {section} {preset} {index} />
    <PresetCardFooter {section} {preset} {index} />
</article>
