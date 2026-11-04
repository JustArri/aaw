<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { UserPresetsSection } from '$lib/sections/user-presets.svelte';

    interface Props {
        section: UserPresetsSection;
    }

    let { section }: Props = $props();
</script>

<header class="border-border flex min-h-14.5 items-center justify-between border-b px-3.75">
    <div class="flex items-baseline gap-2">
        <h2 class="m-0 text-sm font-semibold">User presets</h2>
        <span class="text-foreground-muted text-xs">{section.draft.entries.length} total</span>
    </div>
    <div class="flex items-center gap-1.5">
        <Button
            variant="secondary"
            size="sm"
            square
            class="size-8.5"
            type="button"
            aria-label="Add user preset"
            title="Add user preset"
            disabled={section.busy}
            onclick={() => section.addPreset()}>
            <Icon icon="lucide:plus" width="1.125rem" />
        </Button>
        <Button
            variant="ghost"
            size="sm"
            square
            class="size-8.5"
            type="button"
            aria-label="Discard preset changes"
            title="Discard changes"
            disabled={!section.changed || section.busy}
            onclick={() => section.discardChanges()}>
            <Icon icon="lucide:undo-2" width="1rem" />
        </Button>
        <Button
            variant="accent"
            size="sm"
            square
            class="size-8.5"
            type="button"
            aria-label="Save user presets"
            title="Save user presets"
            busy={section.busy}
            disabled={!section.changed || section.busy}
            onclick={() => void section.save()}>
            <Icon
                icon={section.busy ? 'lucide:loader-circle' : 'lucide:check'}
                width="1.125rem"
                class={section.busy ? 'animate-spin' : ''} />
        </Button>
    </div>
</header>
