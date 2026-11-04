<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { UserPresetsSection } from '$lib/sections/user-presets.svelte';
    import type { EditableUserPreset } from '$lib/state/user-presets.svelte';

    interface Props {
        section: UserPresetsSection;
        preset: EditableUserPreset;
    }

    let { section, preset }: Props = $props();
</script>

<div
    class="border-accent/50 bg-surface-overlay pointer-events-none fixed z-80 flex w-[min(20rem,calc(100vw-2rem))] -translate-x-4 -translate-y-4 items-center gap-2 rounded-xl border p-3 shadow-[0_1rem_2.5rem_var(--overlay-shadow)]"
    style:left={`${section.dragX}px`}
    style:top={`${section.dragY}px`}>
    <Icon icon="lucide:grip-vertical" width="1rem" class="text-accent shrink-0" />
    <span class="text-foreground min-w-0 flex-1 truncate text-sm font-semibold">
        {preset.name}
    </span>
    {#if section.dropInsertionIndex !== null}
        <small class="text-foreground-muted shrink-0 font-mono text-xs">
            #{Math.min(section.dropInsertionIndex + 1, section.draft.entries.length)}
        </small>
    {/if}
</div>
