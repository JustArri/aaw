<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { ControllersSection } from '$lib/sections/controllers.svelte';
    import DrumMachineCard from './DrumMachineCard.svelte';

    interface Props {
        section: ControllersSection;
    }

    let { section }: Props = $props();
</script>

<section class="grid min-w-0 gap-4">
    <div>
        <h3 class="text-foreground m-0 text-sm font-semibold">Drum machines</h3>
        <p class="text-foreground-muted m-0 text-xs">One-bar sequencers</p>
    </div>
    {#if section.loading}
        <p class="text-foreground-muted m-0 text-sm">Loading drum machines…</p>
    {:else if section.machines.length === 0}
        <div
            class="border-border text-foreground-muted rounded-xl border border-dashed p-5 text-center text-sm">
            Add a drum machine to sequence this instrument.
        </div>
    {/if}

    {#each section.machines as machine (machine.id)}
        <DrumMachineCard {section} {machine} />
    {/each}

    <Button
        class="justify-self-end"
        variant="accent-outline"
        busy={section.adding}
        onclick={() => section.addMachine()}>
        <Icon icon="lucide:plus" width="1rem" /> Add
    </Button>
</section>
