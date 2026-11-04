<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import EngineStatusIndicator from '$lib/components/EngineStatusIndicator.svelte';
    import type { MidiEventsSection } from '$lib/sections/midi-events.svelte';

    interface Props {
        section: MidiEventsSection;
    }

    let { section }: Props = $props();
</script>

<header
    class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
    <div>
        <div class="flex items-center gap-2">
            <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                MIDI events
            </h2>
            <EngineStatusIndicator />
        </div>
        <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
            Live input monitor · newest events first
        </p>
    </div>
    <div class="flex items-center gap-2">
        <span
            class="bg-surface-badge text-foreground-strong min-w-7 rounded-md px-2 py-1 text-center font-mono text-sm tabular-nums">
            {section.events.length} / {section.logSize}
        </span>
        <Button
            variant="secondary"
            size="md"
            square
            class="bg-surface-card text-foreground-muted"
            type="button"
            aria-label="Clear MIDI events"
            disabled={section.events.length === 0}
            onclick={section.clearEvents}>
            <Icon icon="lucide:trash-2" width="1rem" />
        </Button>
    </div>
</header>
