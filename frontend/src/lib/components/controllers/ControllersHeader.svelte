<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { ControllersSection } from '$lib/sections/controllers.svelte';

    interface Props {
        section: ControllersSection;
    }

    let { section }: Props = $props();
</script>

<header class="flex min-w-0 items-center gap-3">
    <a
        class="border-border bg-surface-button text-foreground-muted hover:text-accent grid size-9 shrink-0 place-items-center rounded-lg border transition-colors"
        href={`/instruments?instrument=${section.instrumentId}`}
        aria-label="Back to instrument">
        <Icon icon="lucide:arrow-left" width="1rem" />
    </a>
    <div class="min-w-0 flex-1">
        <h2 class="text-foreground m-0 truncate text-base font-semibold">Controllers</h2>
        <p class="text-foreground-muted m-0 text-sm">MIDI input and sample-clock sequencers</p>
    </div>
</header>
