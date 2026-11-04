<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { UserPresetsSection } from '$lib/sections/user-presets.svelte';
    import PresetDragPreview from './PresetDragPreview.svelte';
    import PresetList from './PresetList.svelte';
    import UserPresetsHeader from './UserPresetsHeader.svelte';

    interface Props {
        section: UserPresetsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card min-w-0 overflow-hidden rounded-xl border">
    <UserPresetsHeader {section} />

    {#if section.loading}
        <div class="text-foreground-muted grid min-h-52 place-items-center text-sm">
            Loading presets…
        </div>
    {:else if section.draft.entries.length === 0}
        <div class="text-foreground-muted grid min-h-52 place-items-center p-6 text-center">
            <div>
                <Icon icon="lucide:list-music" width="1.5rem" class="mx-auto mb-2" />
                <p class="m-0 text-sm">No user presets yet.</p>
                <Button
                    variant="accent"
                    size="sm"
                    class="mt-3"
                    type="button"
                    onclick={() => section.addPreset()}>
                    Add first preset
                </Button>
            </div>
        </div>
    {:else}
        <PresetList {section} />
    {/if}
</section>

{#if section.draggedPreset}
    <PresetDragPreview {section} preset={section.draggedPreset} />
{/if}
