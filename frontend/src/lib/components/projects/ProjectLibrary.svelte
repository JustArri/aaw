<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { ProjectsSection } from '$lib/sections/projects.svelte';
    import ActiveProjectName from './ActiveProjectName.svelte';
    import ProjectLibraryHeader from './ProjectLibraryHeader.svelte';
    import ProjectList from './ProjectList.svelte';

    interface Props {
        section: ProjectsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card text-foreground overflow-hidden rounded-xl border">
    <ProjectLibraryHeader {section} />

    {#if section.libraryErrorMessage}
        <div
            class="border-danger/35 bg-danger/8 text-danger flex items-center gap-2 border-b px-3.75 py-3 text-sm min-[56.25rem]:px-5"
            role="alert">
            <Icon class="shrink-0" icon="lucide:circle-alert" width="1rem" />
            <span>{section.libraryErrorMessage}</span>
        </div>
    {/if}

    <ActiveProjectName {section} />
    <ProjectList {section} />
</section>
