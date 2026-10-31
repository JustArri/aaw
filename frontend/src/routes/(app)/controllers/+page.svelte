<script lang="ts">
    import { page } from '$app/state';
    import Icon from '@iconify/svelte';
    import ControllerEditor from '../instruments/+page.svelte';

    let requestedInstrument = $derived(page.url.searchParams.get('synth'));
    let instrumentId = $derived(
        requestedInstrument === null ? Number.NaN : Number(requestedInstrument)
    );
</script>

<div class="grid gap-3">
    <header class="flex min-w-0 items-center gap-3">
        <a
            class="border-border bg-surface-button text-foreground-muted hover:border-accent/35 hover:text-accent grid size-9 shrink-0 place-items-center rounded-lg border transition-colors"
            href={`/synths?synth=${instrumentId}`}
            aria-label="Back to instrument">
            <Icon icon="lucide:arrow-left" width="1rem" />
        </a>
        <div class="min-w-0">
            <h2 class="text-foreground m-0 truncate text-base font-semibold">Controllers</h2>
            <p class="text-foreground-muted m-0 text-sm">MIDI routing and transforms</p>
        </div>
    </header>

    {#if Number.isInteger(instrumentId) && instrumentId >= 0}
        <ControllerEditor embedded {instrumentId} />
    {:else}
        <p class="text-danger m-0 text-sm">Select an instrument first.</p>
    {/if}
</div>
