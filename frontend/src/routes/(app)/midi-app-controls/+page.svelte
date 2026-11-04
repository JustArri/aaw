<script lang="ts">
    import EngineMidiAppControls from '$lib/components/midi-app-controls/EngineMidiAppControls.svelte';
    import { MidiAppControlsSection } from '$lib/sections/midi-app-controls.svelte';

    const section = new MidiAppControlsSection();
</script>

<EngineMidiAppControls {section} />
