<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { ProjectsSection } from '$lib/sections/projects.svelte';

    interface Props {
        section: ProjectsSection;
    }

    let { section }: Props = $props();
</script>

<header
    class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
    <div>
        <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
            Stored projects
        </h2>
        <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
            {section.projectList.length}
            {section.projectList.length === 1 ? 'project' : 'projects'} available
        </p>
    </div>
    <div class="flex items-center gap-2">
        <Button
            variant="secondary"
            size="lg"
            square
            class="bg-surface-card text-foreground-muted"
            type="button"
            aria-label="Refresh projects from disk"
            title="Refresh projects from disk"
            busy={section.refreshing}
            disabled={section.refreshing}
            onclick={() => void section.refresh()}>
            <Icon
                class={section.refreshing
                    ? 'animate-spin'
                    : section.refreshDone
                      ? 'text-success'
                      : ''}
                icon={section.refreshing
                    ? 'lucide:loader-circle'
                    : section.refreshDone
                      ? 'lucide:check'
                      : 'lucide:refresh-cw'}
                width="1rem" />
        </Button>
        <Button
            variant="secondary"
            size="lg"
            square
            class="bg-surface-card min-[37.5rem]:w-auto min-[37.5rem]:px-3"
            type="button"
            aria-label="Create new project"
            title="Create new project"
            busy={section.creating}
            disabled={section.creating || section.importing || section.loadingId !== null}
            onclick={() => void section.createProject()}>
            <Icon
                class={section.creating ? 'animate-spin' : ''}
                icon={section.creating ? 'lucide:loader-circle' : 'lucide:file-plus-2'}
                width="1rem" />
            <span class="hidden text-sm font-semibold min-[37.5rem]:inline">
                {section.creating ? 'Creating…' : 'New'}
            </span>
        </Button>
        <Button
            variant="primary"
            size="md"
            type="button"
            busy={section.saving}
            disabled={section.saving}
            onclick={() => void section.saveCurrent()}>
            {section.saving ? 'Saving…' : 'Save active'}
        </Button>
    </div>
</header>
