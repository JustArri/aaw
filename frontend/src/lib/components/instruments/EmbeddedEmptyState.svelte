<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { InstrumentsSection } from '$lib/sections/instruments.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

<section
    class="border-border bg-surface-card grid justify-items-center gap-3 rounded-xl border p-6 text-center">
    <div>
        <h3 class="text-foreground m-0 text-sm font-semibold">No controllers</h3>
        <p class="text-foreground-muted mt-1 mb-0 text-sm">
            Add a MIDI input controller to route MIDI events into this instrument.
        </p>
    </div>
    {#if section.showAddButton}
        <Button
            variant="primary"
            size="sm"
            type="button"
            busy={section.addingInstrument}
            disabled={section.busy}
            onclick={() => void section.appendInstrument()}>
            <Icon icon="lucide:plus" width="1rem" />
            Add MIDI input
        </Button>
    {/if}
</section>
