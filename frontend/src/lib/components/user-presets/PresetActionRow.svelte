<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import type { UserPresetAction } from '$lib/generated/models';
    import type { UserPresetsSection } from '$lib/sections/user-presets.svelte';
    import PresetActionFields from './PresetActionFields.svelte';

    interface Props {
        section: UserPresetsSection;
        action: UserPresetAction;
        presetIndex: number;
        actionIndex: number;
    }

    let { section, action, presetIndex, actionIndex }: Props = $props();
</script>

<div class="border-border bg-surface-card grid gap-2 rounded-lg border p-2.5">
    <div class="flex min-w-0 items-center gap-1.5">
        <span class="text-foreground-muted w-5 shrink-0 text-center font-mono text-xs">
            {actionIndex + 1}
        </span>
        <SelectDropdown
            class="min-w-0 flex-1"
            value={section.actionType(action)}
            options={section.actionTypeOptions}
            compact
            ariaLabel={`Preset ${presetIndex + 1} action ${actionIndex + 1} type`}
            onchange={(value) => section.setActionType(presetIndex, actionIndex, value)} />
        <Button
            variant="ghost"
            size="sm"
            square
            type="button"
            aria-label="Move action up"
            title="Move action up"
            disabled={actionIndex === 0}
            onclick={() => section.moveAction(presetIndex, actionIndex, -1)}>
            <Icon icon="lucide:arrow-up" width="0.875rem" />
        </Button>
        <Button
            variant="ghost"
            size="sm"
            square
            type="button"
            aria-label="Move action down"
            title="Move action down"
            disabled={actionIndex === section.draft.entries[presetIndex].actions.length - 1}
            onclick={() => section.moveAction(presetIndex, actionIndex, 1)}>
            <Icon icon="lucide:arrow-down" width="0.875rem" />
        </Button>
        <Button
            variant="danger-ghost"
            size="sm"
            square
            type="button"
            aria-label="Remove action"
            title="Remove action"
            onclick={() => section.removeAction(presetIndex, actionIndex)}>
            <Icon icon="lucide:x" width="0.875rem" />
        </Button>
    </div>

    <PresetActionFields {section} {action} {presetIndex} {actionIndex} />
</div>
