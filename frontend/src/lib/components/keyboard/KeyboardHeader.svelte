<script lang="ts">
    import EngineStatusIndicator from '$lib/components/EngineStatusIndicator.svelte';
    import type { KeyboardSection } from '$lib/sections/keyboard.svelte';
    import ChannelFilterPopover from './ChannelFilterPopover.svelte';

    interface Props {
        section: KeyboardSection;
    }

    let { section }: Props = $props();
</script>

<header
    class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
    <div class="min-w-0">
        <div class="flex items-center gap-2">
            <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                Keyboard
            </h2>
            <EngineStatusIndicator />
        </div>
        <p class="text-foreground-muted mt-1 mb-0 flex items-center gap-1.5 truncate text-sm">
            {#if section.latestNote}
                <span
                    class="size-2 shrink-0 rounded-full"
                    style:background-color={section.channelColor(section.latestNote.channel)}>
                </span>
                Input {section.latestNote.slot + 1} · Ch {section.latestNote.channel + 1}
            {:else}
                Waiting for MIDI input
            {/if}
        </p>
    </div>
    <div class="flex shrink-0 items-center gap-2">
        <ChannelFilterPopover {section} />
        <span
            class="bg-surface-badge text-foreground-strong min-w-7 rounded-md px-2 py-1 text-center font-mono text-sm tabular-nums"
            title={`${section.notes.length} active MIDI ${section.notes.length === 1 ? 'note' : 'notes'}`}>
            {section.activeKeys.length}
        </span>
    </div>
</header>
