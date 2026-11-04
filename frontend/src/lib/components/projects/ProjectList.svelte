<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { ProjectsSection } from '$lib/sections/projects.svelte';
    import ProjectListItem from './ProjectListItem.svelte';

    interface Props {
        section: ProjectsSection;
    }

    let { section }: Props = $props();
</script>

{#if section.projectList.length > 0}
    <div class="divide-border divide-y">
        {#each section.projectList as project (project.id)}
            <ProjectListItem {section} {project} />
        {/each}
    </div>
{:else}
    <div
        class="text-foreground-muted grid min-h-56 place-items-center content-center px-6 text-center">
        <span class="bg-accent/10 text-accent grid size-12 place-items-center rounded-xl">
            <Icon icon="lucide:folders" width="1.5rem" />
        </span>
        <h3 class="text-foreground mt-3 mb-0 text-sm font-semibold">No stored projects</h3>
        <p class="mt-1.5 mb-0 max-w-65 text-sm">
            Save the active workspace to create the first project file.
        </p>
    </div>
{/if}
