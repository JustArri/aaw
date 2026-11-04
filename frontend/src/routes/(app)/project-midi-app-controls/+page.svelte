<script lang="ts">
    import ProjectMidiAppControls from '$lib/components/midi-app-controls/ProjectMidiAppControls.svelte';
    import { ProjectMidiAppControlsSection } from '$lib/sections/project-midi-app-controls.svelte';

    const section = new ProjectMidiAppControlsSection();
</script>

<ProjectMidiAppControls {section} />
