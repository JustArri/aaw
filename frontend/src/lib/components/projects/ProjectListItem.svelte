<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { ProjectInfo } from '$lib/generated/models';
    import type { ProjectsSection } from '$lib/sections/projects.svelte';

    interface Props {
        section: ProjectsSection;
        project: ProjectInfo;
    }

    let { section, project }: Props = $props();
</script>

<article
    class="text-foreground hover:bg-surface-hover grid min-h-19 grid-cols-[2.5rem_minmax(0,1fr)] items-center gap-x-3 gap-y-2 px-3.75 py-3 transition-colors min-[37.5rem]:flex min-[37.5rem]:gap-3 min-[56.25rem]:gap-4 min-[56.25rem]:px-5">
    <span
        class={`grid size-10 shrink-0 place-items-center rounded-lg ${section.currentProjectId === project.id ? 'bg-accent/12 text-accent' : 'bg-surface-inset text-foreground-muted'}`}>
        <Icon
            icon={section.currentProjectId === project.id ? 'lucide:folder-open' : 'lucide:folder'}
            width="1.125rem" />
    </span>

    <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
            <h3
                class={`m-0 min-w-0 flex-1 overflow-hidden text-sm font-semibold text-ellipsis whitespace-nowrap min-[56.25rem]:text-base ${project.name.trim() ? 'text-foreground' : 'text-foreground-muted italic'}`}>
                {section.displayProjectName(project.name)}
            </h3>
            {#if section.currentProjectId === project.id}
                <span
                    class="bg-accent/10 text-accent shrink-0 rounded-full px-2 py-0.5 text-xs font-bold uppercase min-[37.5rem]:hidden">
                    Active
                </span>
            {/if}
        </div>
        <div class="text-foreground-muted mt-1 flex min-w-0 items-center gap-2 text-xs">
            <span class="shrink-0">Format v{project.version}</span>
            <span aria-hidden="true">·</span>
            <span
                class="overflow-hidden font-mono text-ellipsis whitespace-nowrap"
                title={project.id}>
                {project.id}
            </span>
        </div>
    </div>

    {#if section.currentProjectId === project.id}
        <span
            class="bg-accent/10 text-accent hidden shrink-0 items-center rounded-full px-2 py-0.5 text-xs font-bold uppercase min-[37.5rem]:inline-flex">
            Active
        </span>
    {/if}

    <div class="col-span-2 flex shrink-0 items-center justify-end gap-1.5 min-[37.5rem]:col-auto">
        <Button
            variant="secondary"
            size="sm"
            class="bg-surface-card enabled:hover:bg-surface-raised"
            type="button"
            disabled={section.currentProjectId === project.id || section.loadingId !== null}
            onclick={() => void section.load(project.id)}>
            {section.loadingId === project.id ? 'Loading…' : 'Load'}
        </Button>
        <Button
            variant="secondary"
            size="md"
            square
            class="bg-surface-card text-foreground-muted enabled:hover:bg-surface-raised"
            type="button"
            aria-label={`Export ${section.displayProjectName(project.name)}`}
            title={`Export ${section.displayProjectName(project.name)}`}
            busy={section.exportingId === project.id}
            disabled={section.exportingId !== null || section.loadingId !== null}
            onclick={() => void section.exportSaved(project.id, project.name)}>
            <Icon
                icon={section.exportingId === project.id
                    ? 'lucide:loader-circle'
                    : 'lucide:download'}
                class={section.exportingId === project.id ? 'animate-spin' : ''}
                width="1rem" />
        </Button>
        <Button
            variant="danger-outline"
            size="md"
            square
            class="bg-surface-card text-foreground-muted enabled:hover:bg-danger/8"
            type="button"
            aria-label={`Delete ${section.displayProjectName(project.name)}`}
            title={section.currentProjectId === project.id
                ? 'Delete saved file; keep the project open'
                : `Delete ${section.displayProjectName(project.name)}`}
            busy={section.deletingId === project.id}
            disabled={section.deletingId !== null}
            onclick={() =>
                void section.remove(project.id, section.displayProjectName(project.name))}>
            <Icon
                icon={section.deletingId === project.id ? 'lucide:loader-circle' : 'lucide:trash-2'}
                class={section.deletingId === project.id ? 'animate-spin' : ''}
                width="1rem" />
        </Button>
    </div>
</article>
