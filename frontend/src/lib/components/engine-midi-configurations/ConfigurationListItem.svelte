<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { EngineMidiConfigurationInfo } from '$lib/generated/models';
    import type { EngineMidiConfigurationsSection } from '$lib/sections/engine-midi-configurations.svelte';

    interface Props {
        section: EngineMidiConfigurationsSection;
        configuration: EngineMidiConfigurationInfo;
    }

    let { section, configuration }: Props = $props();
</script>

<article
    class="hover:bg-surface-hover flex min-h-19 flex-wrap items-center gap-3 px-3.75 py-3 transition-colors min-[56.25rem]:px-5">
    <span
        class={`grid size-10 shrink-0 place-items-center rounded-lg ${section.activeId === configuration.id ? 'bg-accent/12 text-accent' : 'bg-surface-inset text-foreground-muted'}`}>
        <Icon
            icon={section.activeId === configuration.id ? 'lucide:settings-2' : 'lucide:file-cog'}
            width="1.125rem" />
    </span>
    <div class="min-w-40 flex-1">
        <h3 class="m-0 truncate text-sm font-semibold min-[56.25rem]:text-base">
            {section.displayName(configuration.name)}
        </h3>
        <p class="text-foreground-muted mt-1 mb-0 truncate text-xs">
            Format v{configuration.version} · {configuration.id}
        </p>
    </div>
    {#if section.activeId === configuration.id}
        <span class="bg-accent/10 text-accent rounded-full px-2 py-0.5 text-xs font-bold uppercase">
            Active
        </span>
    {/if}
    <div class="ml-auto flex gap-1.5">
        <Button
            variant="secondary"
            size="sm"
            disabled={section.activeId === configuration.id || section.loadingId !== null}
            onclick={() => void section.load(configuration.id)}>
            {section.loadingId === configuration.id ? 'Loading…' : 'Load'}
        </Button>
        <Button
            variant="secondary"
            size="md"
            square
            busy={section.exportingId === configuration.id}
            disabled={section.exportingId !== null}
            aria-label={`Export ${section.displayName(configuration.name)}`}
            title={`Export ${section.displayName(configuration.name)}`}
            onclick={() => void section.exportSaved(configuration.id, configuration.name)}>
            <Icon icon="lucide:download" width="1rem" />
        </Button>
        <Button
            variant="danger-outline"
            size="md"
            square
            busy={section.deletingId === configuration.id}
            disabled={section.deletingId !== null}
            aria-label={`Delete ${section.displayName(configuration.name)}`}
            title={`Delete ${section.displayName(configuration.name)}`}
            onclick={() =>
                void section.remove(configuration.id, section.displayName(configuration.name))}>
            <Icon icon="lucide:trash-2" width="1rem" />
        </Button>
    </div>
</article>
