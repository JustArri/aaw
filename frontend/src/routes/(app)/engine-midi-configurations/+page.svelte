<script lang="ts">
    import ConfigurationLibrary from '$lib/components/engine-midi-configurations/ConfigurationLibrary.svelte';
    import ImportExportConfiguration from '$lib/components/engine-midi-configurations/ImportExportConfiguration.svelte';
    import { EngineMidiConfigurationsSection } from '$lib/sections/engine-midi-configurations.svelte';

    const section = new EngineMidiConfigurationsSection();
</script>

<ImportExportConfiguration {section} />
<ConfigurationLibrary {section} />
