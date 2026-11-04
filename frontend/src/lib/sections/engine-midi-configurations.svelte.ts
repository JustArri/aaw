import { onMount } from 'svelte';
import type { StoredEngineMidiConfiguration } from '$lib/generated/models';
import { errorToUserMessage } from '$lib/error';
import { notify } from '$lib/push-notifications.svelte';
import {
    activeEngineMidiConfiguration,
    addEngineMidiConfiguration,
    createNewActiveEngineMidiConfiguration,
    deleteEngineMidiConfiguration,
    downloadEngineMidiConfiguration,
    engineMidiConfigurations,
    exportActiveEngineMidiConfiguration,
    fetchEngineMidiConfigurations,
    importActiveEngineMidiConfiguration,
    loadActiveEngineMidiConfiguration,
    refreshEngineMidiConfigurations,
    saveActiveEngineMidiConfiguration,
    setActiveEngineMidiConfigurationName
} from '$lib/state/engine-midi-configurations.svelte';

export class EngineMidiConfigurationsSection {
    name = $state('');
    loadingId = $state<string | null>(null);
    deletingId = $state<string | null>(null);
    exportingId = $state<string | null>(null);
    saving = $state(false);
    creating = $state(false);
    renaming = $state(false);
    refreshing = $state(false);
    exporting = $state(false);
    importing = $state(false);
    selectedFile = $state<File | null>(null);
    fileInput = $state<HTMLInputElement | null>(null);
    errorMessage = $state('');

    configurationList = $derived(
        [...engineMidiConfigurations()].sort((left, right) => left.name.localeCompare(right.name))
    );
    activeId = $derived(activeEngineMidiConfiguration()?.storage?.id ?? null);
    nameChanged = $derived(this.name.trim() !== (activeEngineMidiConfiguration()?.name ?? ''));

    constructor() {
        $effect(() => {
            this.name = activeEngineMidiConfiguration()?.name ?? '';
        });
        onMount(() => {
            void fetchEngineMidiConfigurations().catch((error) => this.reportError(error));
        });
    }

    reportError(error: unknown, title = 'MIDI configuration') {
        this.errorMessage = errorToUserMessage(error);
        notify.error(this.errorMessage, { title });
    }

    displayName(value: string) {
        return value.trim() || 'Unnamed MIDI configuration';
    }

    private safeFilename(value: string) {
        return (
            value
                .trim()
                .replace(/[^\p{L}\p{N}._-]+/gu, '-')
                .replace(/^-+|-+$/g, '') || 'aaw-midi-configuration'
        );
    }

    private download(blob: Blob, filename: string) {
        const url = URL.createObjectURL(blob);
        const anchor = document.createElement('a');
        anchor.href = url;
        anchor.download = `${this.safeFilename(filename)}.json`;
        anchor.click();
        URL.revokeObjectURL(url);
    }

    async rename() {
        const nextName = this.name.trim();
        if (!nextName || !this.nameChanged || this.renaming) return;
        this.renaming = true;
        this.errorMessage = '';
        try {
            await setActiveEngineMidiConfigurationName(nextName);
            this.name = nextName;
        } catch (error) {
            this.reportError(error, 'Rename MIDI configuration');
        } finally {
            this.renaming = false;
        }
    }

    async saveCurrent() {
        if (this.saving) return;
        this.saving = true;
        this.errorMessage = '';
        try {
            await saveActiveEngineMidiConfiguration();
        } catch (error) {
            this.reportError(error, 'Save MIDI configuration');
        } finally {
            this.saving = false;
        }
    }

    async createConfiguration() {
        if (this.creating) return;
        if (
            !confirm(
                'Create a new empty MIDI configuration? Unsaved app controls and auto-connections will be discarded.'
            )
        )
            return;
        this.creating = true;
        this.errorMessage = '';
        try {
            await createNewActiveEngineMidiConfiguration();
        } catch (error) {
            this.reportError(error, 'Create MIDI configuration');
        } finally {
            this.creating = false;
        }
    }

    async load(configurationId: string) {
        if (this.loadingId) return;
        this.loadingId = configurationId;
        this.errorMessage = '';
        try {
            await loadActiveEngineMidiConfiguration(configurationId);
        } catch (error) {
            this.reportError(error, 'Load MIDI configuration');
        } finally {
            this.loadingId = null;
        }
    }

    async remove(configurationId: string, configurationName: string) {
        if (this.deletingId) return;
        const active = configurationId === this.activeId;
        const message = active
            ? `Delete the saved file for “${configurationName}”? The active configuration remains loaded.`
            : `Delete “${configurationName}”? This cannot be undone.`;
        if (!confirm(message)) return;
        this.deletingId = configurationId;
        this.errorMessage = '';
        try {
            await deleteEngineMidiConfiguration(configurationId);
        } catch (error) {
            this.reportError(error, 'Delete MIDI configuration');
        } finally {
            this.deletingId = null;
        }
    }

    async refresh() {
        if (this.refreshing) return;
        this.refreshing = true;
        this.errorMessage = '';
        try {
            await refreshEngineMidiConfigurations();
        } catch (error) {
            this.reportError(error, 'Refresh MIDI configurations');
        } finally {
            this.refreshing = false;
        }
    }

    async exportCurrent() {
        if (this.exporting) return;
        this.exporting = true;
        try {
            const configuration = await exportActiveEngineMidiConfiguration();
            this.download(
                new Blob([`${JSON.stringify(configuration, null, 4)}\n`], {
                    type: 'application/json'
                }),
                configuration.name
            );
        } catch (error) {
            this.reportError(error, 'Export MIDI configuration');
        } finally {
            this.exporting = false;
        }
    }

    async exportSaved(configurationId: string, configurationName: string) {
        if (this.exportingId) return;
        this.exportingId = configurationId;
        try {
            this.download(
                await downloadEngineMidiConfiguration(configurationId),
                configurationName
            );
        } catch (error) {
            this.reportError(error, 'Export MIDI configuration');
        } finally {
            this.exportingId = null;
        }
    }

    private async readSelectedFile() {
        if (!this.selectedFile) throw new Error('Select a MIDI configuration JSON file first');
        try {
            return JSON.parse(await this.selectedFile.text()) as StoredEngineMidiConfiguration;
        } catch (error) {
            throw new Error('The selected file is not valid JSON', { cause: error });
        }
    }

    async importConfiguration(setActive: boolean) {
        if (this.importing) return;
        this.importing = true;
        this.errorMessage = '';
        try {
            const configuration = await this.readSelectedFile();
            if (setActive) await importActiveEngineMidiConfiguration(configuration);
            else await addEngineMidiConfiguration(configuration);
            this.selectedFile = null;
            if (this.fileInput) this.fileInput.value = '';
        } catch (error) {
            this.reportError(error, 'Import MIDI configuration');
        } finally {
            this.importing = false;
        }
    }
}
