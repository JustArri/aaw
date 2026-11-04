<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { KeyboardSection } from '$lib/sections/keyboard.svelte';

    interface Props {
        section: KeyboardSection;
    }

    let { section }: Props = $props();
</script>

<div
    class="border-border bg-surface-inset flex items-center gap-2 border-b px-3.75 py-2.5 min-[56.25rem]:px-5">
    <div
        class="border-border-strong bg-surface-card flex h-9 shrink-0 items-stretch overflow-hidden rounded-lg border">
        <span
            class="text-foreground-muted grid min-w-17 place-items-center px-2 font-mono text-xs font-semibold tabular-nums"
            title={section.playedRange
                ? `Recent range ${section.noteName(section.playedRange.min)} – ${section.noteName(section.playedRange.max)}`
                : 'No recent range'}>
            {section.playedRange
                ? `${section.noteName(section.playedRange.min)}–${section.noteName(section.playedRange.max)}`
                : '—'}
        </span>
        <button
            class={`border-border-strong grid w-9 cursor-pointer place-items-center border-0 border-l transition-colors ${section.autoView ? 'bg-accent/12 text-accent hover:bg-accent/18' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'}`}
            type="button"
            aria-label={section.autoView ? 'Disable Auto view' : 'Enable Auto view'}
            aria-pressed={section.autoView}
            title={`Auto view ${section.autoView ? 'on' : 'off'}`}
            onclick={() => section.setAutoView(!section.autoView)}>
            <Icon icon="lucide:scan" width="1rem" />
        </button>
        <button
            class="border-border-strong text-foreground-muted hover:bg-surface-hover hover:text-foreground grid w-9 cursor-pointer place-items-center border-0 border-l transition-colors disabled:cursor-not-allowed disabled:opacity-40"
            type="button"
            aria-label="Clear held notes and learned range"
            title="Clear held notes and learned range"
            disabled={section.allNotes.length === 0 && !section.hasRecentActivity}
            onclick={() => section.clearKeyboard()}>
            <Icon icon="lucide:eraser" width="1rem" />
        </button>
    </div>
    <div
        class="h-10 min-w-0 flex-1 overflow-x-auto overflow-y-hidden overscroll-x-contain"
        aria-live="polite">
        {#if section.activeKeys.length === 0}
            <p class="text-foreground-muted m-0 flex h-full items-center text-sm">
                Play a note to see it here.
            </p>
        {:else}
            <div class="flex h-full w-max items-center gap-2" aria-label="Active notes">
                {#each section.activeKeys as activeKey (activeKey.key)}
                    <span
                        class="border-border-strong bg-surface-card text-foreground inline-flex shrink-0 items-center gap-2 rounded-lg border px-2 py-1 text-sm font-semibold"
                        title={activeKey.notes
                            .map(
                                (note) =>
                                    `Input ${note.slot + 1}, channel ${note.channel + 1}, velocity ${note.velocity}`
                            )
                            .join('\n')}>
                        <span>{section.noteName(activeKey.key)}</span>
                        <span class="font-mono text-xs tabular-nums">{activeKey.velocity}</span>
                        <span class="flex items-center gap-1" aria-label="MIDI channels">
                            {#each [...new Set(activeKey.notes.map((note) => note.channel))] as channel}
                                <span
                                    class="size-2 rounded-full"
                                    style:background-color={section.channelColor(channel)}
                                    title={`Channel ${channel + 1}`}>
                                </span>
                            {/each}
                        </span>
                    </span>
                {/each}
            </div>
        {/if}
    </div>
</div>
