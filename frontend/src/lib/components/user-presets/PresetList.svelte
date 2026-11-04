<script lang="ts">
    import { flip } from 'svelte/animate';
    import type { UserPresetsSection } from '$lib/sections/user-presets.svelte';
    import PresetCard from './PresetCard.svelte';

    interface Props {
        section: UserPresetsSection;
    }

    let { section }: Props = $props();
</script>

<div class="grid p-2 min-[56.25rem]:p-3">
    {#each section.draft.entries as preset, index (preset)}
        <div animate:flip={{ duration: 160 }}>
            {#if index > 0}<div class="bg-border mx-4 my-3 h-px"></div>{/if}
            <PresetCard {section} {preset} {index} />
        </div>
    {/each}
</div>
