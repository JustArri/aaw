<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import { midiControllerNames, midiControllers } from '$lib/content/midi-controller';
    import type { InstrumentsSection } from '$lib/sections/instruments.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

<div class="grid gap-1.5">
    <div
        class="border-border-strong bg-surface-inset flex max-h-32 min-h-10 flex-wrap content-start gap-1 overflow-y-auto rounded-lg border p-1.5">
        {#each section.controllerSection.values as controller (controller)}
            <span
                class="bg-surface-button text-foreground flex h-7 max-w-full items-center gap-1 rounded-md pl-2 text-xs">
                <span class="truncate">
                    CC {controller} · {midiControllerNames[controller] ?? 'Undefined'}
                </span>
                <Button
                    variant="danger-ghost"
                    size="xs"
                    square
                    type="button"
                    aria-label={`Remove CC ${controller}`}
                    title={`Remove CC ${controller}`}
                    onclick={() => section.controllerSection.remove(controller)}>
                    <Icon icon="lucide:x" width="0.875rem" />
                </Button>
            </span>
        {:else}
            <span class="text-foreground-muted self-center px-1 text-xs">
                No controllers selected
            </span>
        {/each}
    </div>
    <div class="flex justify-end gap-1.5">
        <Button
            variant="secondary"
            size="xs"
            class="text-foreground-muted"
            type="button"
            disabled={section.controllerSection.values.length === midiControllers.length}
            onclick={() => {
                section.controllerSection.values = midiControllers.map(({ id }) => id);
            }}>
            All
        </Button>
        <Button
            variant="secondary"
            size="xs"
            class="text-foreground-muted"
            type="button"
            disabled={section.controllerSection.values.length === 0}
            onclick={() => {
                section.controllerSection.values = [];
            }}>
            None
        </Button>
    </div>
</div>
