<script lang="ts">
    import AppearanceSettings from '$lib/components/settings/AppearanceSettings.svelte';
    import MidiSettings from '$lib/components/settings/MidiSettings.svelte';
    import { SettingsSection } from '$lib/sections/settings.svelte';

    const section = new SettingsSection();
</script>

<div class="grid w-full gap-3">
    <AppearanceSettings {section} />
    <MidiSettings {section} />
</div>
