<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { UserPresetsSection } from '$lib/sections/user-presets.svelte';
    import type { EditableUserPreset } from '$lib/state/user-presets.svelte';

    interface Props {
        section: UserPresetsSection;
        preset: EditableUserPreset;
        index: number;
    }

    let { section, preset, index }: Props = $props();
</script>

<div class="flex min-w-0 items-center gap-2">
    <button
        class="text-foreground-muted hover:bg-surface-hover grid size-8 shrink-0 cursor-grab touch-none place-items-center rounded-lg border-0 bg-transparent active:cursor-grabbing"
        type="button"
        aria-label={`Drag ${preset.name}`}
        title="Drag to reorder"
        onpointerdown={(event) => section.startPresetDrag(event, preset)}
        onpointermove={(event) => section.dragPreset(event)}
        onpointerup={(event) => section.stopPresetDrag(event)}
        onpointercancel={(event) => section.cancelPresetDrag(event)}>
        <Icon icon="lucide:grip-vertical" width="1rem" />
    </button>
    <span class="text-foreground-muted grid size-6 shrink-0 place-items-center font-mono text-sm">
        {index + 1}
    </span>
    <input
        class="bg-surface-card text-foreground placeholder:text-foreground-muted h-9 min-w-0 flex-1 rounded-lg border-0 px-2.5 text-sm placeholder:italic"
        aria-label={`Preset ${index + 1} name`}
        placeholder={`Unnamed preset #${index + 1}`}
        bind:value={preset.name} />
    <button
        class={`grid size-9 shrink-0 place-items-center rounded-lg border-0 bg-transparent ${section.changed ? 'text-foreground-icon cursor-default' : 'text-accent hover:bg-accent/10 cursor-pointer disabled:cursor-wait'}`}
        type="button"
        aria-label={`Activate ${preset.name}`}
        title={section.changed ? 'Save changes before activating' : 'Activate preset'}
        disabled={section.changed || section.activating !== null}
        onclick={() => void section.activate(index)}>
        <Icon
            icon={section.activating === index ? 'lucide:loader-circle' : 'lucide:play'}
            width="1rem"
            class={section.activating === index ? 'animate-spin' : ''} />
    </button>
</div>
