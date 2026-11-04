<script lang="ts">
    import NumberInput from '$lib/components/NumberInput.svelte';
    import type { SettingsSection } from '$lib/sections/settings.svelte';

    interface Props {
        section: SettingsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="m-0 text-base min-[56.25rem]:text-lg">MIDI</h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm">
                MIDI monitoring preferences are stored on this device.
            </p>
        </div>
    </header>
    <div class="flex items-center justify-between gap-5 p-3.75 min-[56.25rem]:p-5">
        <div>
            <label
                class="block text-sm font-semibold min-[56.25rem]:text-sm"
                for="midi-event-log-size">
                Event log size
            </label>
            <small class="text-foreground-muted mt-1 block text-xs min-[56.25rem]:text-sm">
                Maximum number of recent MIDI events retained
            </small>
        </div>
        <div class="flex shrink-0 items-center gap-2">
            <NumberInput
                class="h-9 w-20 px-2.5 text-right"
                id="midi-event-log-size"
                min={section.minMidiEventLogSize}
                max={section.maxMidiEventLogSize}
                step={1}
                value={section.midiEventLogSize}
                onchange={(value) => section.updateMidiEventLogSize(value)} />
            <span class="text-foreground-muted text-sm">events</span>
        </div>
    </div>
</section>
