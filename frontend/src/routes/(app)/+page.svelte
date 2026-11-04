<script lang="ts">
    import GlobalControls from '$lib/components/overview/GlobalControls.svelte';
    import LiveInstruments from '$lib/components/overview/LiveInstruments.svelte';
    import LivePresets from '$lib/components/overview/LivePresets.svelte';
    import ProjectTransport from '$lib/components/ProjectTransport.svelte';
    import { OverviewSection } from '$lib/sections/overview.svelte';

    const section = new OverviewSection();
</script>

<div class="grid gap-3 min-[37.5rem]:grid-cols-2 min-[37.5rem]:items-start min-[56.25rem]:gap-3.5">
    <GlobalControls {section} />
    <div class="contents">
        <LivePresets {section} />
        <ProjectTransport
            showBeatVisualizer
            showConfiguration={false}
            title="Beat"
            subtitle=""
            class="order-2 min-[37.5rem]:order-0 min-[37.5rem]:col-span-2 min-[37.5rem]:row-start-2" />
        <LiveInstruments {section} />
    </div>
</div>
