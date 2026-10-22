<script lang="ts">
    import Icon from '@iconify/svelte';
    import { icons } from '$lib/icons';
    import { notify } from '$lib/push-notifications.svelte';
    import { activeProject } from '$lib/state/active-project.svelte';
    import { saveActiveProject } from '$lib/state/projects.svelte';

    let { activeLabel }: { activeLabel: string } = $props();

    const projectName = $derived(activeProject()?.name.trim() || 'Untitled project');
    const projectRevision = $derived(activeProject()?.revision ?? '?');
    let saving = $state(false);
    const saveState = $derived.by(() => {
        const project = activeProject();

        if (!project) {
            return null;
        }
        if (!project.storage) {
            return { label: 'Never saved', title: 'This project has never been saved' };
        }

        const changes = Math.abs(project.revision - project.storage.revision);
        if (changes === 0) {
            return { label: 'Saved', title: `Saved at revision ${project.storage.revision}` };
        }
        return {
            label: `${changes} ${changes === 1 ? 'change' : 'changes'}`,
            title: `${changes} unsaved ${changes === 1 ? 'change' : 'changes'}`
        };
    });

    async function onProjectSave() {
        if (saving) {
            return;
        }

        saving = true;
        const notification = notify.info('Saving project…', {
            duration: null,
            loading: true,
            title: 'Save project'
        });
        try {
            await saveActiveProject();
            notification.success('Project saved.');
        } catch (error) {
            notification.error(error instanceof Error ? error.message : 'Could not save project.');
        } finally {
            saving = false;
        }
    }
</script>

<header
    class="border-border bg-app-background/94 sticky top-0 z-30 grid h-15.5 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 border-b px-3 backdrop-blur-md min-[37.5rem]:px-5 min-[56.25rem]:h-17 min-[56.25rem]:px-7.5">
    <a
        class="flex min-w-0 cursor-pointer items-center gap-2 border-0 bg-transparent p-1 text-left"
        href="/"
        aria-label={`Open live controls for ${projectName}`}>
        <span
            class="bg-accent/10 text-accent grid size-7.75 shrink-0 place-items-center rounded-lg">
            <Icon icon="lucide:folder-open" width="0.9375rem" />
        </span>
        <span class="min-w-0">
            <small class="text-accent block truncate text-xs font-semibold">{activeLabel}</small>
            <strong class="block truncate text-sm font-semibold">{projectName}</strong>
        </span>
        <span class="text-foreground-icon shrink-0">
            <Icon icon={icons.chevron} width="0.9375rem" />
        </span>
    </a>
    <div
        class="[&>button]:border-border-strong [&>button]:bg-surface-card [&>button]:text-foreground-strong flex items-center gap-1.5 [&>button]:grid [&>button]:size-8.5 [&>button]:cursor-pointer [&>button]:place-items-center [&>button]:rounded-lg [&>button]:border">
        <span
            class="border-border text-foreground-subtle hidden rounded-md border px-1.75 py-1 font-mono text-xs min-[37.5rem]:inline min-[56.25rem]:text-xs">
            rev {projectRevision}
        </span>
        {#if saveState}
            <span
                class="text-foreground-muted mr-0.5 flex items-center gap-1.75 text-xs min-[56.25rem]:mr-1"
                title={saveState.title}>
                <span>{saveState.label}</span>
            </span>
        {/if}
        <button
            type="button"
            aria-label={saving ? 'Saving project' : 'Save project'}
            title={saving ? 'Saving project' : 'Save project'}
            disabled={saving || !activeProject()}
            onclick={onProjectSave}>
            <Icon
                icon={saving ? 'lucide:loader-circle' : icons.save}
                width="1.0625rem"
                class={saving ? 'animate-spin' : ''} />
        </button>
        <a
            class="border-border-strong bg-surface-card text-foreground-strong grid size-8.5 place-items-center rounded-lg border"
            href="/settings"
            aria-label="Settings">
            <Icon icon={icons.settings} width="1.0625rem" />
        </a>
    </div>
</header>
