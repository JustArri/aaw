<script lang="ts">
    import type { SystemSection } from '$lib/sections/system.svelte';
    import PowerActionCard from './PowerActionCard.svelte';

    interface Props {
        section: SystemSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="m-0 text-base min-[56.25rem]:text-lg">Power controls</h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm">
                Safely restart or shut down this AAW device.
            </p>
        </div>
    </header>

    <div
        class="grid gap-3 p-3.75 min-[43.75rem]:grid-cols-2 min-[56.25rem]:gap-4 min-[56.25rem]:p-5">
        <PowerActionCard {section} action="reboot" />
        <PowerActionCard {section} action="shutdown" />
    </div>
</section>
