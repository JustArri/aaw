<script lang="ts">
    import ProjectFilePanel from '$lib/components/projects/ProjectFilePanel.svelte';
    import ProjectLibrary from '$lib/components/projects/ProjectLibrary.svelte';
    import { ProjectsSection } from '$lib/sections/projects.svelte';

    const section = new ProjectsSection();
</script>

<ProjectFilePanel {section} />
<ProjectLibrary {section} />
