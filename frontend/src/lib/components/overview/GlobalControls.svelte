<script lang="ts">
    import NumericControl from '$lib/components/NumericControl.svelte';
    import type { OverviewSection } from '$lib/sections/overview.svelte';

    interface Props {
        section: OverviewSection;
    }

    let { section }: Props = $props();
</script>

<div
    class="contents min-[37.5rem]:order-0 min-[37.5rem]:col-start-2 min-[37.5rem]:row-start-3 min-[37.5rem]:grid min-[37.5rem]:content-start min-[37.5rem]:gap-3 min-[56.25rem]:gap-3.5">
    <section class="order-3 min-w-0 min-[37.5rem]:order-0" aria-label="Global transposition">
        <NumericControl
            label="Transposition"
            sectionLabel
            value={section.globalTransposition}
            unit="semitones"
            smallStep={1}
            largeStep={12}
            resetValue={0}
            min={-128}
            max={127}
            wideValue
            disabled={section.loading}
            onchange={section.setGlobalTransposition} />
    </section>
    <section class="order-5 min-w-0 min-[37.5rem]:order-0" aria-label="Global gain">
        <NumericControl
            label="Gain"
            sectionLabel
            value={section.globalGainDb}
            unit="dB"
            smallStep={0.5}
            largeStep={5}
            resetValue={0}
            min={-100}
            max={100}
            precision={1}
            disabled={section.loading || !section.project}
            onchange={section.setGlobalGainDb} />
    </section>
</div>
