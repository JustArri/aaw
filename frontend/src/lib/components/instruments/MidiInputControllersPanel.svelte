<script lang="ts">
    import ControllerEditor from '$lib/components/instruments/ControllerEditor.svelte';
    import ControllerList from '$lib/components/instruments/ControllerList.svelte';
    import EmbeddedEmptyState from '$lib/components/instruments/EmbeddedEmptyState.svelte';
    import { InstrumentsSection } from '$lib/sections/instruments.svelte';

    interface Props {
        instrumentId?: number | null;
        embedded?: boolean;
        showAddButton?: boolean;
    }

    let {
        instrumentId = null,
        embedded: routeEmbedded = false,
        showAddButton: routeShowAddButton = true
    }: Props = $props();

    const section = new InstrumentsSection({
        instrumentId: () => instrumentId,
        embedded: () => routeEmbedded,
        showAddButton: () => routeShowAddButton
    });
</script>

<div
    class={section.embedded
        ? 'min-w-0'
        : 'grid min-w-0 gap-3 min-[56.25rem]:grid-cols-[minmax(18rem,0.8fr)_minmax(24rem,1.2fr)] min-[56.25rem]:items-start'}>
    {#if !section.embedded}
        <ControllerList {section} />
    {/if}

    {#if section.embedded && !section.loading && section.entries.length === 0}
        <EmbeddedEmptyState {section} />
    {/if}

    {#if section.selectedSlot && section.selectedId !== null}
        <ControllerEditor {section} />
    {/if}
</div>
