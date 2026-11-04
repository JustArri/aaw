<script lang="ts">
    import AutoSaveTextInput from '$lib/components/AutoSaveTextInput.svelte';
    import type { ProjectsSection } from '$lib/sections/projects.svelte';

    interface Props {
        section: ProjectsSection;
    }

    let { section }: Props = $props();
</script>

<form
    class="border-border grid gap-3 border-b px-3.75 py-3.5 min-[37.5rem]:grid-cols-[minmax(0,1fr)_minmax(12rem,24.25rem)] min-[37.5rem]:items-end min-[56.25rem]:px-5"
    onsubmit={(event) => {
        event.preventDefault();
        void section.renameActiveProject();
    }}>
    <div>
        <label class="text-foreground block text-sm font-semibold" for="active-project-name">
            Active project name
        </label>
        <small class="text-foreground-muted mt-1 block text-sm">
            This name is shown throughout the workspace.
        </small>
    </div>
    <AutoSaveTextInput
        id="active-project-name"
        bind:value={section.projectName}
        pending={section.projectNameChanged}
        saving={section.renaming}
        disabled={!section.active}
        required
        oncommit={() => section.renameActiveProject()} />
</form>
