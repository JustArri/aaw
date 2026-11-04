<script lang="ts">
    import MidiEventsHeader from '$lib/components/midi-events/MidiEventsHeader.svelte';
    import MidiEventsTable from '$lib/components/midi-events/MidiEventsTable.svelte';
    import { MidiEventsSection } from '$lib/sections/midi-events.svelte';

    const section = new MidiEventsSection();
</script>

<section
    class="border-border bg-surface-card flex max-h-[calc(100dvh-9rem)] w-full max-w-full min-w-0 flex-col overflow-hidden rounded-xl border contain-[inline-size] min-[56.25rem]:max-h-[calc(100dvh-8rem)]">
    <MidiEventsHeader {section} />
    <MidiEventsTable {section} />
</section>
