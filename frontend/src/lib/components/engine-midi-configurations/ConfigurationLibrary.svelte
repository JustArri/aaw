<script lang="ts">
    import ActiveConfigurationName from './ActiveConfigurationName.svelte';
    import ConfigurationLibraryHeader from './ConfigurationLibraryHeader.svelte';
    import ConfigurationList from './ConfigurationList.svelte';
    import type { EngineMidiConfigurationsSection } from '$lib/sections/engine-midi-configurations.svelte';

    interface Props {
        section: EngineMidiConfigurationsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <ConfigurationLibraryHeader {section} />

    {#if section.errorMessage}
        <div class="border-danger/35 bg-danger/8 text-danger border-b px-3.75 py-3 text-sm">
            {section.errorMessage}
        </div>
    {/if}

    <ActiveConfigurationName {section} />
    <ConfigurationList {section} />
</section>
