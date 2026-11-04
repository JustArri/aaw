<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { KeyboardSection } from '$lib/sections/keyboard.svelte';

    interface Props {
        section: KeyboardSection;
    }

    let { section }: Props = $props();
</script>

<div class="group relative">
    <button
        class={`grid size-9 cursor-pointer place-items-center rounded-lg border transition-colors ${section.enabledChannelCount === section.channelColors.length ? 'border-border-strong bg-surface-card text-foreground-muted hover:bg-surface-hover hover:text-foreground' : 'border-accent/40 bg-accent/10 text-accent hover:bg-accent/15'}`}
        type="button"
        aria-label="Filter MIDI channels"
        aria-expanded={section.channelHelpOpen}
        aria-controls="midi-channel-color-help"
        onclick={() => {
            section.channelHelpOpen = !section.channelHelpOpen;
        }}
        onblur={() => {
            section.channelHelpOpen = false;
        }}>
        <Icon icon="lucide:info" width="1rem" />
    </button>
    <div
        id="midi-channel-color-help"
        class={`border-border bg-surface-overlay absolute top-full right-0 z-30 mt-2 w-56 rounded-xl border p-3 shadow-[0_0.75rem_2rem_var(--overlay-shadow)] transition-[visibility,opacity,transform] ${section.channelHelpOpen ? 'visible translate-y-0 opacity-100' : 'invisible -translate-y-1 opacity-0 group-focus-within:visible group-focus-within:translate-y-0 group-focus-within:opacity-100 group-hover:visible group-hover:translate-y-0 group-hover:opacity-100'}`}
        role="group"
        aria-label="MIDI channel filters">
        <div class="mb-2 flex items-center justify-between gap-2">
            <strong class="text-foreground block text-sm font-semibold">MIDI channels</strong>
            <div class="border-border-strong flex overflow-hidden rounded-md border">
                <button
                    class="text-foreground-muted hover:bg-surface-hover hover:text-foreground cursor-pointer border-0 bg-transparent px-2 py-1 text-xs font-semibold"
                    type="button"
                    onclick={() => section.setAllChannelsEnabled(true)}>
                    All
                </button>
                <button
                    class="border-border-strong text-foreground-muted hover:bg-surface-hover hover:text-foreground cursor-pointer border-0 border-l bg-transparent px-2 py-1 text-xs font-semibold"
                    type="button"
                    onclick={() => section.setAllChannelsEnabled(false)}>
                    None
                </button>
            </div>
        </div>
        <div class="grid grid-cols-2 gap-x-3 gap-y-2">
            {#each section.channelColors as color, channel}
                <button
                    class={`flex min-h-7 cursor-pointer items-center gap-2 rounded-md border px-1.5 text-left text-[0.6875rem] font-medium whitespace-nowrap transition-colors ${section.enabledChannels[channel] ? 'text-foreground' : 'border-border text-foreground-muted bg-surface-inset opacity-65 hover:opacity-100'}`}
                    style:border-color={section.enabledChannels[channel] ? color : undefined}
                    style:background-color={section.enabledChannels[channel]
                        ? `color-mix(in srgb, ${color} 12%, transparent)`
                        : undefined}
                    type="button"
                    aria-pressed={section.enabledChannels[channel]}
                    onclick={() =>
                        section.setChannelEnabled(channel, !section.enabledChannels[channel])}>
                    <span
                        class="size-2.5 shrink-0 rounded-full"
                        style:background-color={section.enabledChannels[channel]
                            ? color
                            : `color-mix(in srgb, ${color} 25%, transparent)`}>
                    </span>
                    Channel {channel + 1}
                </button>
            {/each}
        </div>
    </div>
</div>
