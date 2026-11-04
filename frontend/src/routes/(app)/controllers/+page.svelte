<script lang="ts">
    import ControllersHeader from '$lib/components/controllers/ControllersHeader.svelte';
    import DrumMachinesSection from '$lib/components/controllers/DrumMachinesSection.svelte';
    import MidiInputControllersSection from '$lib/components/controllers/MidiInputControllersSection.svelte';
    import ProjectTransport from '$lib/components/ProjectTransport.svelte';
    import { ControllersSection } from '$lib/sections/controllers.svelte';

    const section = new ControllersSection();
</script>

<div class="grid min-w-0 gap-5 overflow-x-clip">
    <ControllersHeader {section} />

    {#if Number.isInteger(section.instrumentId) && section.instrumentId >= 0}
        <ProjectTransport onchange={() => section.load()} />
        <DrumMachinesSection {section} />
        <MidiInputControllersSection {section} />
    {:else}
        <p class="text-danger m-0 text-sm">Select an instrument first.</p>
    {/if}
</div>
