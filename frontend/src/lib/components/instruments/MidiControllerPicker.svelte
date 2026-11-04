<script lang="ts">
    import Icon from '@iconify/svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import { midiControllers } from '$lib/content/midi-controller';
    import type { InstrumentsSection } from '$lib/sections/instruments.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

<div class="grid min-w-0 gap-1">
    <div class="flex min-w-0 gap-1.5">
        <div class="relative min-w-0 flex-1">
            <SelectDropdown
                id="midi-controller-hints"
                bind:value={section.controllerSection.query}
                options={midiControllers.map((controller) => ({
                    value: String(controller.id),
                    label: `CC ${controller.id}`,
                    selectionLabel: `CC ${controller.id} · ${controller.name}`,
                    description: section.controllerSection.values.includes(controller.id)
                        ? `${controller.name} · Already added`
                        : controller.name,
                    disabled: section.controllerSection.values.includes(controller.id)
                }))}
                placeholder="Select controller"
                disabled={section.controllerSection.available.length === 0}
                compact
                clearable
                searchable
                searchPlaceholder="Search controllers"
                noMatchesText="No matching controllers"
                ariaLabel="MIDI controller to add"
                onchange={() => section.controllerSection.add()} />
        </div>
        <button
            class={`grid size-9 shrink-0 cursor-pointer place-items-center rounded-lg border-0 transition-colors ${section.controllerSection.learning ? 'bg-danger/15 text-danger animate-pulse' : 'bg-surface-button text-foreground enabled:hover:bg-surface-hover'}`}
            type="button"
            aria-label={section.controllerSection.learning
                ? 'Stop learning MIDI controller'
                : 'Learn MIDI controller'}
            title={section.controllerSection.learning
                ? 'Waiting for a Control Change event'
                : 'Learn from the next Control Change event'}
            aria-pressed={section.controllerSection.learning}
            onclick={() => section.controllerSection.toggleLearning()}>
            <Icon
                icon={section.controllerSection.learning
                    ? 'lucide:radio-tower'
                    : 'lucide:circle-dot'}
                width="1rem" />
        </button>
    </div>
    {#if section.controllerSection.selectedHint}
        <small class="text-foreground-muted truncate px-1 text-xs">
            CC {section.controllerSection.selectedHint.id} · {section.controllerSection.selectedHint
                .name}
        </small>
    {:else if section.controllerSection.addedHint}
        <small class="text-foreground-muted truncate px-1 text-xs">
            Already added: CC {section.controllerSection.addedHint.id} · {section.controllerSection
                .addedHint.name}
        </small>
    {:else}
        <small class="text-foreground-subtle truncate px-1 text-xs">No matching controller</small>
    {/if}
</div>
