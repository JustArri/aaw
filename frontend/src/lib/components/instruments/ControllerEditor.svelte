<script lang="ts">
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import type { InstrumentsSection } from '$lib/sections/instruments.svelte';
    import BehaviorSection from './BehaviorSection.svelte';
    import ControllerEditorHeader from './ControllerEditorHeader.svelte';
    import IgnoredMessagesSection from './IgnoredMessagesSection.svelte';
    import KeySection from './KeySection.svelte';
    import MidiChannelsSection from './MidiChannelsSection.svelte';
    import MidiControllersSection from './MidiControllersSection.svelte';
    import MidiSourcesSection from './MidiSourcesSection.svelte';
    import NameSection from './NameSection.svelte';
    import VelocitySection from './VelocitySection.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

<aside class="border-border bg-surface-card min-w-0 overflow-hidden rounded-xl border">
    <ControllerEditorHeader {section} />
    <div>
        <NameSection {section} />
        <BehaviorSection {section} />
        <MidiSourcesSection {section} />
        <KeySection {section} />
        <VelocitySection {section} />
        <MidiChannelsSection {section} />
        <MidiControllersSection {section} />
        <IgnoredMessagesSection {section} />

        {#if !section.embedded}
            <section class="p-4">
                <TimedConfirmButton
                    class="w-full"
                    label="Remove instrument"
                    confirmLabel="Confirm remove instrument"
                    busyLabel="Removing…"
                    busy={section.removingInstrument}
                    disabled={section.busy}
                    onconfirm={() => section.deleteSelected()} />
            </section>
        {/if}
    </div>
</aside>
