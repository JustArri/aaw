<script lang="ts">
    import type { MidiInputsSection } from '$lib/sections/midi-inputs.svelte';
    import AutoConnectionForm from './AutoConnectionForm.svelte';
    import AutoConnectionRow from './AutoConnectionRow.svelte';

    interface Props {
        section: MidiInputsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card min-w-0 rounded-xl border">
    <header
        class="border-border min-h-16.75 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
            Auto-connections
        </h2>
        <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
            Reconnect known devices when they become available.
        </p>
    </header>

    <AutoConnectionForm {section} />

    {#if section.autoConnections.length === 0}
        <div class="p-5 text-center">
            <p class="text-foreground-muted m-0 text-sm">No automatic connections configured.</p>
        </div>
    {:else}
        <div class="divide-border divide-y">
            {#each section.autoConnections as connection (connection.port_id)}
                <AutoConnectionRow {section} {connection} />
            {/each}
        </div>
    {/if}
</section>
