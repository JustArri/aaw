import { onMount } from 'svelte';
import {
    deleteProject,
    downloadProjectFile,
    exportActiveProject,
    fetchProjects,
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

export class ProjectsSection {
    refreshing = $state(false);
    refreshDone = $state(false);
    saving = $state(false);
    loadingId = $state<string | null>(null);
    deletingId = $state<string | null>(null);
    importing = $state(false);
    exporting = $state(false);
    exportingId = $state<string | null>(null);
    selectedFile = $state<File | null>(null);
    fileInput = $state<HTMLInputElement | null>(null);
    draggingFile = $state(false);
    fileErrorMessage = $state('');
    libraryErrorMessage = $state('');
    projectName = $state('');
    renaming = $state(false);
    creating = $state(false);
    private dragDepth = 0;

    projectList = $derived(
        [...projects()].sort((left, right) => left.name.localeCompare(right.name))
    );
    currentProjectId = $derived(activeProject()?.storage?.id ?? null);
    active = $derived(activeProject());
    projectNameChanged = $derived(this.projectName.trim() !== (activeProject()?.name ?? ''));

    constructor() {
        $effect(() => {
            this.projectName = activeProject()?.name ?? '';
        });
        onMount(() => {
            void fetchProjects().catch((error) => {
                this.libraryErrorMessage = this.messageFor(error);
            });
        });
    }

    private messageFor(error: unknown) {
        return error instanceof Error ? error.message : 'The project operation failed';
    }

    private safeFilename(name: string) {
        const filename = name
            .trim()
            .replace(/[^\p{L}\p{N}._-]+/gu, '-')
            .replace(/^-+|-+$/g, '');
        return filename || 'aaw-project';
    }

    displayProjectName(name: string) {
        return name.trim() || 'Untitled project';
    }

    private async readSelectedProject() {
        if (!this.selectedFile) throw new Error('Select a project JSON file first');
        try {
            return JSON.parse(await this.selectedFile.text()) as StoredProject;
        } catch (error) {
            throw new Error('The selected file is not valid JSON', { cause: error });
        }
    }

    clearSelectedFile() {
        this.selectedFile = null;
        if (this.fileInput) this.fileInput.value = '';
    }

    selectProjectFile(file: File | null) {
        this.fileErrorMessage = '';
        if (!file) {
            this.clearSelectedFile();
            return;
        }
        if (file.type !== 'application/json' && !file.name.toLowerCase().endsWith('.json')) {
            this.clearSelectedFile();
            this.fileErrorMessage = 'Select a JSON project file';
            notify.error(this.fileErrorMessage, { title: 'Select project file' });
            return;
        }
        this.selectedFile = file;
    }

    formatFileSize(size: number) {
        if (size < 1_024) return `${size} B`;
        if (size < 1_048_576) return `${(size / 1_024).toFixed(1)} KB`;
        return `${(size / 1_048_576).toFixed(1)} MB`;
    }

    handleDragEnter(event: DragEvent) {
        event.preventDefault();
        this.dragDepth += 1;
        this.draggingFile = true;
    }

    handleDragOver(event: DragEvent) {
        event.preventDefault();
        if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
    }

    handleDragLeave(event: DragEvent) {
        event.preventDefault();
        this.dragDepth = Math.max(0, this.dragDepth - 1);
        if (this.dragDepth === 0) this.draggingFile = false;
    }

    handleDrop(event: DragEvent) {
        event.preventDefault();
        this.dragDepth = 0;
        this.draggingFile = false;
        const files = event.dataTransfer?.files;
        if (!files || files.length === 0) return;
        if (files.length !== 1) {
            this.fileErrorMessage = 'Drop one project file at a time';
            notify.error(this.fileErrorMessage, { title: 'Select project file' });
            return;
        }
        this.selectProjectFile(files.item(0));
    }

    private downloadBlob(blob: Blob, name: string) {
        const url = URL.createObjectURL(blob);
        const anchor = document.createElement('a');
        anchor.href = url;
        anchor.download = `${this.safeFilename(name)}.json`;
        anchor.click();
        URL.revokeObjectURL(url);
    }

    private downloadProject(project: StoredProject) {
        this.downloadBlob(
            new Blob([`${JSON.stringify(project, null, 4)}\n`], { type: 'application/json' }),
            project.name
        );
    }

    async importProject(setActive: boolean) {
        if (this.importing) return;
        this.importing = true;
        this.fileErrorMessage = '';
        try {
            const project = await this.readSelectedProject();
            if (setActive) await importActiveProject(project);
            else await saveProject(project);
            this.clearSelectedFile();
        } catch (error) {
            this.fileErrorMessage = this.messageFor(error);
            notify.error(this.fileErrorMessage, {
                title: setActive ? 'Import active project' : 'Add project to library'
            });
        } finally {
            this.importing = false;
        }
    }

    async exportCurrent() {
        if (this.exporting) return;
        this.exporting = true;
        this.fileErrorMessage = '';
        try {
            const project = await exportActiveProject();
            if (!project) {
                notify.warning('No active project is available.', {
                    title: 'Export active project'
                });
                return;
            }
            this.downloadProject(project);
        } catch (error) {
            this.fileErrorMessage = this.messageFor(error);
            notify.error(this.fileErrorMessage, { title: 'Export active project' });
        } finally {
            this.exporting = false;
        }
    }

    async exportSaved(projectId: string, projectName: string) {
        if (this.exportingId) return;
        this.exportingId = projectId;
        this.libraryErrorMessage = '';
        try {
            const projectFile = await downloadProjectFile(projectId);
            if (!projectFile) {
                notify.warning('The project could not be loaded.', {
                    title: 'Export saved project'
                });
                return;
            }
            this.downloadBlob(projectFile, projectName);
        } catch (error) {
            this.libraryErrorMessage = this.messageFor(error);
            notify.error(this.libraryErrorMessage, { title: 'Export saved project' });
        } finally {
            this.exportingId = null;
        }
    }

    async refresh() {
        if (this.refreshing) return;
        this.refreshing = true;
        this.refreshDone = false;
        this.libraryErrorMessage = '';
        try {
            await refreshProjects();
            this.refreshDone = true;
            setTimeout(() => (this.refreshDone = false), 1200);
        } catch (error) {
            this.libraryErrorMessage = this.messageFor(error);
            notify.error(this.libraryErrorMessage, { title: 'Refresh projects' });
        } finally {
            this.refreshing = false;
        }
    }

    async saveCurrent() {
        if (this.saving) return;
        this.saving = true;
        this.libraryErrorMessage = '';
        try {
            await saveActiveProject();
        } catch (error) {
            this.libraryErrorMessage = this.messageFor(error);
            notify.error(this.libraryErrorMessage, { title: 'Save project' });
        } finally {
            this.saving = false;
        }
    }

    async createProject() {
        if (this.creating) return;
        const project = activeProject();
        const hasUnsavedChanges =
            project !== null &&
            (project.storage === null || project.storage.revision !== project.revision);
        if (
            hasUnsavedChanges &&
            !confirm('Create a new project and discard the current unsaved changes?')
        )
            return;
        this.creating = true;
        this.libraryErrorMessage = '';
        try {
            await createNewActiveProject();
        } catch (error) {
            this.libraryErrorMessage = this.messageFor(error);
            notify.error(this.libraryErrorMessage, { title: 'New project' });
        } finally {
            this.creating = false;
        }
    }

    async renameActiveProject() {
        const name = this.projectName.trim();
        if (!name || !this.projectNameChanged || this.renaming) return;
        this.renaming = true;
        this.libraryErrorMessage = '';
        try {
            await setActiveProjectName(name);
            this.projectName = name;
        } catch (error) {
            this.libraryErrorMessage = this.messageFor(error);
            notify.error(this.libraryErrorMessage, { title: 'Rename project' });
        } finally {
            this.renaming = false;
        }
    }

    async load(projectId: string) {
        if (this.loadingId) return;
        this.loadingId = projectId;
        this.libraryErrorMessage = '';
        try {
            await loadActiveProject(projectId);
        } catch (error) {
            this.libraryErrorMessage = this.messageFor(error);
            notify.error(this.libraryErrorMessage, { title: 'Load project' });
        } finally {
            this.loadingId = null;
        }
    }

    async remove(projectId: string, projectName: string) {
        if (this.deletingId) return;
        const isActive = this.currentProjectId === projectId;
        const confirmation = isActive
            ? `Delete the saved file for “${projectName}”? The active project will remain open as an unsaved project.`
            : `Delete “${projectName}”? This cannot be undone.`;
        if (!confirm(confirmation)) return;
        this.deletingId = projectId;
        this.libraryErrorMessage = '';
        try {
            await deleteProject(projectId);
        } catch (error) {
            this.libraryErrorMessage = this.messageFor(error);
            notify.error(this.libraryErrorMessage, { title: 'Delete project' });
        } finally {
            this.deletingId = null;
        }
    }
}
