<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { EngineMidiConfigurationsSection } from '$lib/sections/engine-midi-configurations.svelte';
    import ConfigurationListItem from './ConfigurationListItem.svelte';

    interface Props {
        section: EngineMidiConfigurationsSection;
    }

    let { section }: Props = $props();
</script>

{#if section.configurationList.length > 0}
    <div class="divide-border divide-y">
        {#each section.configurationList as configuration (configuration.id)}
            <ConfigurationListItem {section} {configuration} />
        {/each}
    </div>
{:else}
    <div
        class="text-foreground-muted grid min-h-48 place-items-center content-center px-6 text-center">
        <Icon class="text-accent" icon="lucide:file-cog" width="1.5rem" />
        <h3 class="text-foreground mt-3 mb-0 text-sm font-semibold">
            No stored MIDI configurations
        </h3>
        <p class="mt-1.5 mb-0 max-w-72 text-sm">
            Save the active MIDI setup to create the first combined configuration file.
        </p>
    </div>
{/if}
