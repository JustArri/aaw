<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { UserPresetsSection } from '$lib/sections/user-presets.svelte';
    import type { EditableUserPreset } from '$lib/state/user-presets.svelte';
    import PresetActionRow from './PresetActionRow.svelte';

    interface Props {
        section: UserPresetsSection;
        preset: EditableUserPreset;
        index: number;
    }

    let { section, preset, index }: Props = $props();
</script>

<div class="mt-4 grid gap-2">
    <div class="flex items-baseline justify-between gap-2">
        <span class="text-foreground-strong text-xs font-semibold uppercase">Actions</span>
        <span class="text-foreground-muted text-xs">Applied from top to bottom</span>
    </div>
    {#each preset.actions as action, actionIndex}
        <PresetActionRow {section} {action} presetIndex={index} {actionIndex} />
    {:else}
        <p
            class="text-foreground-muted border-border m-0 rounded-lg border border-dashed p-3 text-center text-xs">
            This preset has no actions.
        </p>
    {/each}
    <Button
        variant="secondary"
        size="sm"
        class="justify-self-start"
        type="button"
        onclick={() => section.addAction(index)}>
        <Icon icon="lucide:plus" width="0.875rem" />
        Add action
    </Button>
</div>
