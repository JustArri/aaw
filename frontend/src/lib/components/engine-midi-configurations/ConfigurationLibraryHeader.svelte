<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { EngineMidiConfigurationsSection } from '$lib/sections/engine-midi-configurations.svelte';

    interface Props {
        section: EngineMidiConfigurationsSection;
    }

    let { section }: Props = $props();
</script>

<header
    class="border-border flex min-h-16.75 items-center justify-between gap-3 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
    <div>
        <h2 class="m-0 text-base font-semibold min-[56.25rem]:text-lg">
            Stored MIDI configurations
        </h2>
        <p class="text-foreground-muted mt-1 mb-0 text-sm">
            {section.configurationList.length}
            {section.configurationList.length === 1 ? 'configuration' : 'configurations'} available
        </p>
    </div>
    <div class="flex items-center gap-2">
        <Button
            variant="secondary"
            size="lg"
            square
            busy={section.refreshing}
            disabled={section.refreshing}
            aria-label="Refresh MIDI configurations"
            title="Refresh MIDI configurations"
            onclick={() => void section.refresh()}>
            <Icon
                class={section.refreshing ? 'animate-spin' : ''}
                icon={section.refreshing ? 'lucide:loader-circle' : 'lucide:refresh-cw'}
                width="1rem" />
        </Button>
        <Button
            variant="secondary"
            size="lg"
            square
            class="min-[37.5rem]:w-auto min-[37.5rem]:px-3"
            busy={section.creating}
            disabled={section.creating || section.loadingId !== null}
            aria-label="Create new empty MIDI configuration"
            title="Create new empty MIDI configuration"
            onclick={() => void section.createConfiguration()}>
            <Icon
                class={section.creating ? 'animate-spin' : ''}
                icon={section.creating ? 'lucide:loader-circle' : 'lucide:file-plus-2'}
                width="1rem" />
            <span class="hidden text-sm font-semibold min-[37.5rem]:inline">
                {section.creating ? 'Creating…' : 'New'}
            </span>
        </Button>
        <Button
            variant="primary"
            size="md"
            busy={section.saving}
            disabled={section.saving || !section.name.trim()}
            onclick={() => void section.saveCurrent()}>
            {section.saving ? 'Saving…' : 'Save active'}
        </Button>
    </div>
</header>
