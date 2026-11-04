<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { UserPresetsSection } from '$lib/sections/user-presets.svelte';
    import type { EditableUserPreset } from '$lib/state/user-presets.svelte';

    interface Props {
        section: UserPresetsSection;
        preset: EditableUserPreset;
        index: number;
    }

    let { section, preset, index }: Props = $props();
</script>

<div class="mt-3 flex justify-end gap-1">
    <Button
        variant="ghost"
        size="sm"
        square
        type="button"
        aria-label="Move preset up"
        title="Move up"
        disabled={index === 0}
        onclick={() => section.movePreset(index, -1)}>
        <Icon icon="lucide:arrow-up" width="0.9375rem" />
    </Button>
    <Button
        variant="ghost"
        size="sm"
        square
        type="button"
        aria-label="Move preset down"
        title="Move down"
        disabled={index === section.draft.entries.length - 1}
        onclick={() => section.movePreset(index, 1)}>
        <Icon icon="lucide:arrow-down" width="0.9375rem" />
    </Button>
    <Button
        variant="danger-ghost"
        size="sm"
        square
        type="button"
        aria-label={`Remove ${preset.name}`}
        title="Remove preset"
        onclick={() => section.removePreset(index)}>
        <Icon icon="lucide:trash-2" width="0.9375rem" />
    </Button>
</div>
