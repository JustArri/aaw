<script lang="ts">
    import Icon from '@iconify/svelte';
    import { onMount } from 'svelte';
    import {
        deleteProject,
        exportActiveProject,
        fetchProjects,
        loadProject,
        loadActiveProject,
        projects,
        refreshProjects,
        saveActiveProject,
        saveProject,
        importActiveProject
    } from '$lib/state/projects.svelte';
    import {
        activeProject,
        createNewActiveProject,
        setActiveProjectName
    } from '$lib/state/active-project.svelte';
    import type { StoredProject } from '$lib/generated/models';
    import { notify } from '$lib/push-notifications.svelte';

    let refreshing = $state(false);
    let refreshDone = $state(false);
    let saving = $state(false);
    let loadingId = $state(null as string | null);
    let deletingId = $state(null as string | null);
    let importing = $state(false);
    let exporting = $state(false);
    let exportingId = $state(null as string | null);
    let selectedFile = $state(null as File | null);
    let fileInput = $state(null as HTMLInputElement | null);
    let draggingFile = $state(false);
    let dragDepth = 0;
    let fileErrorMessage = $state('');
    let libraryErrorMessage = $state('');
    let projectName = $state('');
    let renaming = $state(false);
    let creating = $state(false);

    let projectList = $derived(
        [...projects()].sort((left, right) => left.name.localeCompare(right.name))
    );
    let currentProjectId = $derived(activeProject()?.storage?.id ?? null);

    $effect(() => {
        projectName = activeProject()?.name ?? '';
    });

    onMount(() => {
        void fetchProjects().catch((error) => {
            libraryErrorMessage = messageFor(error);
        });
    });

    function messageFor(error: unknown) {
        if (error instanceof Error) {
            return error.message;
        }
        return 'The project operation failed';
    }

    function safeFilename(name: string) {
        const filename = name
            .trim()
            .replace(/[^\p{L}\p{N}._-]+/gu, '-')
            .replace(/^-+|-+$/g, '');
        return filename || 'aaw-project';
    }

    function displayProjectName(name: string) {
        return name.trim() || 'Untitled project';
    }

    async function readSelectedProject() {
        if (!selectedFile) {
            throw new Error('Select a project JSON file first');
        }

        try {
            return JSON.parse(await selectedFile.text()) as StoredProject;
        } catch (error) {
            throw new Error('The selected file is not valid JSON', { cause: error });
        }
    }

    function clearSelectedFile() {
        selectedFile = null;
        if (fileInput) {
            fileInput.value = '';
        }
    }

    function selectProjectFile(file: File | null) {
        fileErrorMessage = '';

        if (!file) {
            clearSelectedFile();
            return;
        }

        if (file.type !== 'application/json' && !file.name.toLowerCase().endsWith('.json')) {
            clearSelectedFile();
            fileErrorMessage = 'Select a JSON project file';
            notify.error(fileErrorMessage, { title: 'Select project file' });
            return;
        }

        selectedFile = file;
    }

    function formatFileSize(size: number) {
        if (size < 1_024) {
            return `${size} B`;
        }
        if (size < 1_048_576) {
            return `${(size / 1_024).toFixed(1)} KB`;
        }
        return `${(size / 1_048_576).toFixed(1)} MB`;
    }

    function handleDragEnter(event: DragEvent) {
        event.preventDefault();
        dragDepth += 1;
        draggingFile = true;
    }

    function handleDragOver(event: DragEvent) {
        event.preventDefault();
        if (event.dataTransfer) {
            event.dataTransfer.dropEffect = 'copy';
        }
    }

    function handleDragLeave(event: DragEvent) {
        event.preventDefault();
        dragDepth = Math.max(0, dragDepth - 1);
        if (dragDepth === 0) {
            draggingFile = false;
        }
    }

    function handleDrop(event: DragEvent) {
        event.preventDefault();
        dragDepth = 0;
        draggingFile = false;

        const files = event.dataTransfer?.files;
        if (!files || files.length === 0) {
            return;
        }
        if (files.length !== 1) {
            fileErrorMessage = 'Drop one project file at a time';
            notify.error(fileErrorMessage, { title: 'Select project file' });
            return;
        }

        selectProjectFile(files.item(0));
    }

    function downloadProject(project: StoredProject) {
        const blob = new Blob([`${JSON.stringify(project, null, 4)}\n`], {
            type: 'application/json'
        });
        const url = URL.createObjectURL(blob);
        const anchor = document.createElement('a');
        anchor.href = url;
        anchor.download = `${safeFilename(project.name)}.json`;
        anchor.click();
        URL.revokeObjectURL(url);
    }

    async function importProject(setActive: boolean) {
        if (importing) {
            return;
        }
        importing = true;
        fileErrorMessage = '';
        const notification = notify.info('Reading and importing the project…', {
            duration: null,
            loading: true,
            title: setActive ? 'Import active project' : 'Add project to library'
        });

        try {
            const project = await readSelectedProject();
            if (setActive) {
                await importActiveProject(project);
            } else {
                await saveProject(project);
            }
            clearSelectedFile();
            notification.success(
                setActive ? 'Active project imported.' : 'Project added to the library.'
            );
        } catch (error) {
            fileErrorMessage = messageFor(error);
            notification.error(fileErrorMessage);
        } finally {
            importing = false;
        }
    }

    async function exportCurrent() {
        if (exporting) {
            return;
        }
        exporting = true;
        fileErrorMessage = '';
        const notification = notify.info('Preparing the project file…', {
            duration: null,
            loading: true,
            title: 'Export active project'
        });

        try {
            const project = await exportActiveProject();
            if (!project) {
                notification.warning('No active project is available.');
                return;
            }
            downloadProject(project);
            notification.success('Project exported.');
        } catch (error) {
            fileErrorMessage = messageFor(error);
            notification.error(fileErrorMessage);
        } finally {
            exporting = false;
        }
    }

    async function exportSaved(projectId: string) {
        if (exportingId) {
            return;
        }
        exportingId = projectId;
        libraryErrorMessage = '';
        const notification = notify.info('Preparing the project file…', {
            duration: null,
            loading: true,
            title: 'Export saved project'
        });

        try {
            const project = await loadProject(projectId);
            if (!project) {
                notification.warning('The project could not be loaded.');
                return;
            }
            downloadProject(project);
            notification.success('Project exported.');
        } catch (error) {
            libraryErrorMessage = messageFor(error);
            notification.error(libraryErrorMessage);
        } finally {
            exportingId = null;
        }
    }

    async function refresh() {
        if (refreshing) {
            return;
        }
        refreshing = true;
        refreshDone = false;
        libraryErrorMessage = '';

        try {
            await refreshProjects();
            refreshDone = true;
            setTimeout(() => {
                refreshDone = false;
            }, 1200);
        } catch (error) {
            libraryErrorMessage = messageFor(error);
            notify.error(libraryErrorMessage, { title: 'Refresh projects' });
        } finally {
            refreshing = false;
        }
    }

    async function saveCurrent() {
        if (saving) {
            return;
        }
        saving = true;
        libraryErrorMessage = '';
        const notification = notify.info('Saving the active project…', {
            duration: null,
            loading: true,
            title: 'Save project'
        });

        try {
            await saveActiveProject();
            notification.success('Project saved.');
        } catch (error) {
            libraryErrorMessage = messageFor(error);
            notification.error(libraryErrorMessage);
        } finally {
            saving = false;
        }
    }

    async function createProject() {
        if (creating) {
            return;
        }

        const project = activeProject();
        const hasUnsavedChanges =
            project !== null &&
            (project.storage === null || project.storage.revision !== project.revision);
        if (
            hasUnsavedChanges &&
            !confirm('Create a new project and discard the current unsaved changes?')
        ) {
            return;
        }

        creating = true;
        libraryErrorMessage = '';
        const notification = notify.info('Creating an empty project…', {
            duration: null,
            loading: true,
            title: 'New project'
        });

        try {
            await createNewActiveProject();
            notification.success('Empty project created.');
        } catch (error) {
            libraryErrorMessage = messageFor(error);
            notification.error(libraryErrorMessage);
        } finally {
            creating = false;
        }
    }

    async function renameActiveProject() {
        const name = projectName.trim();
        if (!name || renaming) {
            return;
        }
        renaming = true;
        libraryErrorMessage = '';
        const notification = notify.info('Updating the project name…', {
            duration: null,
            loading: true,
            title: 'Rename project'
        });

        try {
            await setActiveProjectName(name);
            projectName = name;
            notification.success('Project renamed.');
        } catch (error) {
            libraryErrorMessage = messageFor(error);
            notification.error(libraryErrorMessage);
        } finally {
            renaming = false;
        }
    }

    async function load(projectId: string) {
        if (loadingId) {
            return;
        }
        loadingId = projectId;
        libraryErrorMessage = '';
        const notification = notify.info('Loading the selected project…', {
            duration: null,
            loading: true,
            title: 'Load project'
        });

        try {
            await loadActiveProject(projectId);
            notification.success('Project loaded.');
        } catch (error) {
            libraryErrorMessage = messageFor(error);
            notification.error(libraryErrorMessage);
        } finally {
            loadingId = null;
        }
    }

    async function remove(projectId: string, projectName: string) {
        if (deletingId) {
            return;
        }

        const isActive = currentProjectId === projectId;
        const confirmation = isActive
            ? `Delete the saved file for “${projectName}”? The active project will remain open as an unsaved project.`
            : `Delete “${projectName}”? This cannot be undone.`;
        if (!confirm(confirmation)) {
            return;
        }
        deletingId = projectId;
        libraryErrorMessage = '';
        const notification = notify.info('Deleting the project…', {
            duration: null,
            loading: true,
            title: 'Delete project'
        });

        try {
            await deleteProject(projectId);
            notification.success('Project deleted.');
        } catch (error) {
            libraryErrorMessage = messageFor(error);
            notification.error(libraryErrorMessage);
        } finally {
            deletingId = null;
        }
    }
</script>

<section class="border-border bg-surface-card mb-3 overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                Project file
            </h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
                Import or export complete project JSON files.
            </p>
        </div>
        <button
            class="border-border-strong bg-surface-card text-foreground-strong hover:bg-surface-hover min-h-10 shrink-0 cursor-pointer rounded-lg border px-3 text-sm font-semibold disabled:cursor-not-allowed disabled:opacity-50"
            type="button"
            disabled={exporting}
            onclick={() => void exportCurrent()}>
            <span class="flex items-center gap-2">
                <Icon
                    class={exporting ? 'animate-spin' : ''}
                    icon={exporting ? 'lucide:loader-circle' : 'lucide:download'}
                    width="0.9375rem" />
                {exporting ? 'Exporting…' : 'Export active'}
            </span>
        </button>
    </header>

    {#if fileErrorMessage}
        <div
            class="border-danger/35 bg-danger/8 text-danger flex items-center gap-2 border-b px-3.75 py-3 text-sm min-[56.25rem]:px-5"
            role="alert">
            <Icon class="shrink-0" icon="lucide:circle-alert" width="1rem" />
            <span>{fileErrorMessage}</span>
        </div>
    {/if}

    <form
        class="grid gap-3 p-3.75 min-[56.25rem]:p-5"
        onsubmit={(event) => {
            event.preventDefault();
            void importProject(false);
        }}>
        <input
            class="sr-only"
            id="project-file"
            bind:this={fileInput}
            type="file"
            accept="application/json,.json"
            onchange={(event) => selectProjectFile(event.currentTarget.files?.item(0) ?? null)} />
        <label
            class={`grid min-h-36 cursor-pointer place-items-center rounded-xl border border-dashed px-4 py-5 text-center transition-colors ${draggingFile ? 'border-accent bg-accent/10' : selectedFile ? 'border-accent/50 bg-accent/5 hover:bg-accent/8' : 'border-border-strong bg-surface-inset hover:border-accent/50 hover:bg-surface-hover'}`}
            for="project-file"
            ondragenter={handleDragEnter}
            ondragover={handleDragOver}
            ondragleave={handleDragLeave}
            ondrop={handleDrop}>
            <span class="grid justify-items-center gap-2.5">
                <span
                    class={`grid size-11 place-items-center rounded-xl ${selectedFile || draggingFile ? 'bg-accent/12 text-accent' : 'bg-surface-icon text-foreground-muted'}`}>
                    <Icon
                        icon={draggingFile
                            ? 'lucide:download'
                            : selectedFile
                              ? 'lucide:file-check-2'
                              : 'lucide:file-up'}
                        width="1.25rem" />
                </span>
                {#if draggingFile}
                    <span class="text-sm font-semibold">Drop the project here</span>
                {:else if selectedFile}
                    <span class="grid min-w-0 gap-1">
                        <strong
                            class="max-w-full overflow-hidden text-sm text-ellipsis whitespace-nowrap">
                            {selectedFile.name}
                        </strong>
                        <small class="text-foreground-muted text-sm">
                            {formatFileSize(selectedFile.size)} · Drop another file or tap to replace
                        </small>
                    </span>
                {:else}
                    <span class="grid gap-1">
                        <strong class="text-sm">Drop a project JSON file here</strong>
                        <small class="text-foreground-muted text-sm">or tap to browse files</small>
                    </span>
                {/if}
            </span>
        </label>
        <div class="flex flex-wrap items-center justify-end gap-2">
            {#if selectedFile}
                <button
                    class="text-foreground-muted hover:text-foreground mr-auto min-h-11 cursor-pointer border-0 bg-transparent px-2 text-sm font-semibold"
                    type="button"
                    disabled={importing}
                    onclick={clearSelectedFile}>
                    Remove file
                </button>
            {/if}
            <button
                class="border-border-strong bg-surface-card text-foreground-strong hover:bg-surface-hover min-h-11 cursor-pointer rounded-lg border px-4 text-sm font-bold disabled:cursor-not-allowed disabled:opacity-50"
                type="button"
                disabled={!selectedFile || importing}
                onclick={() => void importProject(true)}>
                {importing ? 'Importing…' : 'Set active'}
            </button>
            <button
                class="bg-accent text-accent-foreground min-h-11 cursor-pointer rounded-lg border-0 px-4 text-sm font-bold disabled:cursor-not-allowed disabled:opacity-50"
                type="submit"
                disabled={!selectedFile || importing}>
                {importing ? 'Importing…' : 'Add to library'}
            </button>
        </div>
    </form>
</section>

<section class="border-border bg-surface-card text-foreground overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                Stored projects
            </h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
                {projectList.length}
                {projectList.length === 1 ? 'project' : 'projects'} available
            </p>
        </div>
        <div class="flex items-center gap-2">
            <button
                class="border-border-strong bg-surface-card text-foreground-muted hover:bg-surface-hover hover:text-foreground grid size-10 shrink-0 cursor-pointer place-items-center rounded-lg border disabled:cursor-not-allowed disabled:opacity-50"
                type="button"
                aria-label="Refresh projects from disk"
                title="Refresh projects from disk"
                disabled={refreshing}
                onclick={() => void refresh()}>
                <Icon
                    class={refreshing ? 'animate-spin' : refreshDone ? 'text-success' : ''}
                    icon={refreshing
                        ? 'lucide:loader-circle'
                        : refreshDone
                          ? 'lucide:check'
                          : 'lucide:refresh-cw'}
                    width="1rem" />
            </button>
            <button
                class="border-border-strong bg-surface-card text-foreground-strong hover:bg-surface-hover grid size-10 shrink-0 cursor-pointer place-items-center rounded-lg border disabled:cursor-not-allowed disabled:opacity-50 min-[37.5rem]:flex min-[37.5rem]:w-auto min-[37.5rem]:gap-2 min-[37.5rem]:px-3"
                type="button"
                aria-label="Create new project"
                title="Create new project"
                disabled={creating || importing || loadingId !== null}
                onclick={() => void createProject()}>
                <Icon
                    class={creating ? 'animate-spin' : ''}
                    icon={creating ? 'lucide:loader-circle' : 'lucide:file-plus-2'}
                    width="1rem" />
                <span class="hidden text-sm font-semibold min-[37.5rem]:inline">
                    {creating ? 'Creating…' : 'New'}
                </span>
            </button>
            <button
                class="bg-accent text-accent-foreground min-h-10 shrink-0 cursor-pointer rounded-lg border-0 px-3.5 text-sm font-bold disabled:cursor-not-allowed disabled:opacity-50 min-[56.25rem]:px-4 min-[56.25rem]:text-sm"
                type="button"
                disabled={saving}
                onclick={() => void saveCurrent()}>
                {saving ? 'Saving…' : 'Save active'}
            </button>
        </div>
    </header>

    {#if libraryErrorMessage}
        <div
            class="border-danger/35 bg-danger/8 text-danger flex items-center gap-2 border-b px-3.75 py-3 text-sm min-[56.25rem]:px-5"
            role="alert">
            <Icon class="shrink-0" icon="lucide:circle-alert" width="1rem" />
            <span>{libraryErrorMessage}</span>
        </div>
    {/if}

    <form
        class="border-border grid gap-3 border-b px-3.75 py-3.5 min-[37.5rem]:grid-cols-[minmax(0,1fr)_minmax(12rem,22rem)_auto] min-[37.5rem]:items-end min-[56.25rem]:px-5"
        onsubmit={(event) => {
            event.preventDefault();
            void renameActiveProject();
        }}>
        <div>
            <label class="text-foreground block text-sm font-semibold" for="active-project-name">
                Active project name
            </label>
            <small class="text-foreground-muted mt-1 block text-sm">
                This name is shown throughout the workspace.
            </small>
        </div>
        <input
            id="active-project-name"
            class="border-border-strong bg-surface-inset text-foreground h-10 min-w-0 rounded-lg border px-3 text-sm"
            bind:value={projectName}
            disabled={!activeProject() || renaming}
            required />
        <button
            class="bg-accent text-accent-foreground h-10 cursor-pointer rounded-lg border-0 px-4 text-sm font-bold disabled:cursor-not-allowed disabled:opacity-50"
            disabled={!activeProject() || !projectName.trim() || renaming}>
            {renaming ? 'Saving…' : 'Save name'}
        </button>
    </form>

    {#if projectList.length > 0}
        <div class="divide-border divide-y">
            {#each projectList as project (project.id)}
                <article
                    class="text-foreground hover:bg-surface-hover grid min-h-19 grid-cols-[2.5rem_minmax(0,1fr)] items-center gap-x-3 gap-y-2 px-3.75 py-3 transition-colors min-[37.5rem]:flex min-[37.5rem]:gap-3 min-[56.25rem]:gap-4 min-[56.25rem]:px-5">
                    <span
                        class={`grid size-10 shrink-0 place-items-center rounded-lg ${currentProjectId === project.id ? 'bg-accent/12 text-accent' : 'bg-surface-inset text-foreground-muted'}`}>
                        <Icon
                            icon={currentProjectId === project.id
                                ? 'lucide:folder-open'
                                : 'lucide:folder'}
                            width="1.125rem" />
                    </span>

                    <div class="min-w-0 flex-1">
                        <div class="flex items-center gap-2">
                            <h3
                                class={`m-0 min-w-0 flex-1 overflow-hidden text-sm font-semibold text-ellipsis whitespace-nowrap min-[56.25rem]:text-base ${project.name.trim() ? 'text-foreground' : 'text-foreground-muted italic'}`}>
                                {displayProjectName(project.name)}
                            </h3>
                            {#if currentProjectId === project.id}
                                <span
                                    class="bg-accent/10 text-accent shrink-0 rounded-full px-2 py-0.5 text-xs font-bold uppercase min-[37.5rem]:hidden">
                                    Active
                                </span>
                            {/if}
                        </div>
                        <div
                            class="text-foreground-muted mt-1 flex min-w-0 items-center gap-2 text-xs">
                            <span class="shrink-0">Format v{project.version}</span>
                            <span aria-hidden="true">·</span>
                            <span
                                class="overflow-hidden font-mono text-ellipsis whitespace-nowrap"
                                title={project.id}>
                                {project.id}
                            </span>
                        </div>
                    </div>

                    {#if currentProjectId === project.id}
                        <span
                            class="bg-accent/10 text-accent hidden shrink-0 items-center rounded-full px-2 py-0.5 text-xs font-bold uppercase min-[37.5rem]:inline-flex">
                            Active
                        </span>
                    {/if}

                    <div
                        class="col-span-2 flex shrink-0 items-center justify-end gap-1.5 min-[37.5rem]:col-auto">
                        <button
                            class="border-border-strong bg-surface-card text-foreground-strong hover:bg-surface-raised min-h-9 cursor-pointer rounded-lg border px-3 text-sm font-semibold disabled:cursor-not-allowed disabled:opacity-50"
                            type="button"
                            disabled={currentProjectId === project.id || loadingId !== null}
                            onclick={() => void load(project.id)}>
                            {loadingId === project.id ? 'Loading…' : 'Load'}
                        </button>
                        <button
                            class="border-border-strong bg-surface-card text-foreground-muted hover:bg-surface-raised hover:text-foreground grid size-9 cursor-pointer place-items-center rounded-lg border disabled:cursor-not-allowed disabled:opacity-40"
                            type="button"
                            aria-label={`Export ${displayProjectName(project.name)}`}
                            title={`Export ${displayProjectName(project.name)}`}
                            disabled={exportingId !== null || loadingId !== null}
                            onclick={() => void exportSaved(project.id)}>
                            <Icon
                                icon={exportingId === project.id
                                    ? 'lucide:loader-circle'
                                    : 'lucide:download'}
                                class={exportingId === project.id ? 'animate-spin' : ''}
                                width="1rem" />
                        </button>
                        <button
                            class="border-border-strong bg-surface-card text-foreground-muted hover:border-danger/40 hover:bg-danger/8 hover:text-danger grid size-9 cursor-pointer place-items-center rounded-lg border disabled:cursor-not-allowed disabled:opacity-40"
                            type="button"
                            aria-label={`Delete ${displayProjectName(project.name)}`}
                            title={currentProjectId === project.id
                                ? 'Delete saved file; keep the project open'
                                : `Delete ${displayProjectName(project.name)}`}
                            disabled={deletingId !== null}
                            onclick={() =>
                                void remove(project.id, displayProjectName(project.name))}>
                            <Icon
                                icon={deletingId === project.id
                                    ? 'lucide:loader-circle'
                                    : 'lucide:trash-2'}
                                class={deletingId === project.id ? 'animate-spin' : ''}
                                width="1rem" />
                        </button>
                    </div>
                </article>
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
</section>
