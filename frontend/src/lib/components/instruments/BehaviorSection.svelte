<script lang="ts">
    import DotToggle from '$lib/components/DotToggle.svelte';
    import type { InstrumentsSection } from '$lib/sections/instruments.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border grid gap-3 border-b p-4">
    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">Behavior</h3>
    <DotToggle
        class="min-w-0 justify-between px-3"
        label="Use global transposition"
        indicatorEnd
        bind:checked={section.behaviorSection.useGlobalTransposition}
        onchange={() => void section.behaviorSection.save()} />
</section>
