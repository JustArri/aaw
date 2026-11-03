<script lang="ts">
    import Icon from '@iconify/svelte';
    import { onMount } from 'svelte';
    import AutoSaveTextInput from '$lib/components/AutoSaveTextInput.svelte';
    import Button from '$lib/components/Button.svelte';
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

    let name = $state('');
    let loadingId = $state<string | null>(null);
    let deletingId = $state<string | null>(null);
    let exportingId = $state<string | null>(null);
    let saving = $state(false);
    let creating = $state(false);
    let renaming = $state(false);
    let refreshing = $state(false);
    let exporting = $state(false);
    let importing = $state(false);
    let selectedFile = $state<File | null>(null);
    let fileInput = $state<HTMLInputElement | null>(null);
    let errorMessage = $state('');

    let configurationList = $derived(
        [...engineMidiConfigurations()].sort((left, right) => left.name.localeCompare(right.name))
    );
    let activeId = $derived(activeEngineMidiConfiguration()?.storage?.id ?? null);
    let nameChanged = $derived(name.trim() !== (activeEngineMidiConfiguration()?.name ?? ''));

    $effect(() => {
        name = activeEngineMidiConfiguration()?.name ?? '';
    });

    onMount(() => {
        void fetchEngineMidiConfigurations().catch(reportError);
    });

    function reportError(error: unknown, title = 'MIDI configuration') {
        errorMessage = errorToUserMessage(error);
        notify.error(errorMessage, { title });
    }

    function displayName(value: string) {
        return value.trim() || 'Unnamed MIDI configuration';
    }

    function safeFilename(value: string) {
        return (
            value
                .trim()
                .replace(/[^\p{L}\p{N}._-]+/gu, '-')
                .replace(/^-+|-+$/g, '') || 'aaw-midi-configuration'
        );
    }

    function download(blob: Blob, filename: string) {
        const url = URL.createObjectURL(blob);
        const anchor = document.createElement('a');
        anchor.href = url;
        anchor.download = `${safeFilename(filename)}.json`;
        anchor.click();
        URL.revokeObjectURL(url);
    }

    async function rename() {
        const nextName = name.trim();
        if (!nextName || !nameChanged || renaming) {
            return;
        }
        renaming = true;
        errorMessage = '';
        try {
            await setActiveEngineMidiConfigurationName(nextName);
            name = nextName;
        } catch (error) {
            reportError(error, 'Rename MIDI configuration');
        } finally {
            renaming = false;
        }
    }

    async function saveCurrent() {
        if (saving) {
            return;
        }
        saving = true;
        errorMessage = '';
        try {
            await saveActiveEngineMidiConfiguration();
        } catch (error) {
            reportError(error, 'Save MIDI configuration');
        } finally {
            saving = false;
        }
    }

    async function createConfiguration() {
        if (creating) {
            return;
        }
        if (
            !confirm(
                'Create a new empty MIDI configuration? Unsaved app controls and auto-connections will be discarded.'
            )
        ) {
            return;
        }
        creating = true;
        errorMessage = '';
        try {
            await createNewActiveEngineMidiConfiguration();
        } catch (error) {
            reportError(error, 'Create MIDI configuration');
        } finally {
            creating = false;
        }
    }

    async function load(configurationId: string) {
        if (loadingId) {
            return;
        }
        loadingId = configurationId;
        errorMessage = '';
        try {
            await loadActiveEngineMidiConfiguration(configurationId);
        } catch (error) {
            reportError(error, 'Load MIDI configuration');
        } finally {
            loadingId = null;
        }
    }

    async function remove(configurationId: string, configurationName: string) {
        if (deletingId) {
            return;
        }
        const active = configurationId === activeId;
        const message = active
            ? `Delete the saved file for “${configurationName}”? The active configuration remains loaded.`
            : `Delete “${configurationName}”? This cannot be undone.`;
        if (!confirm(message)) {
            return;
        }
        deletingId = configurationId;
        errorMessage = '';
        try {
            await deleteEngineMidiConfiguration(configurationId);
        } catch (error) {
            reportError(error, 'Delete MIDI configuration');
        } finally {
            deletingId = null;
        }
    }

    async function refresh() {
        if (refreshing) {
            return;
        }
        refreshing = true;
        errorMessage = '';
        try {
            await refreshEngineMidiConfigurations();
        } catch (error) {
            reportError(error, 'Refresh MIDI configurations');
        } finally {
            refreshing = false;
        }
    }

    async function exportCurrent() {
        if (exporting) {
            return;
        }
        exporting = true;
        try {
            const configuration = await exportActiveEngineMidiConfiguration();
            download(
                new Blob([`${JSON.stringify(configuration, null, 4)}\n`], {
                    type: 'application/json'
                }),
                configuration.name
            );
        } catch (error) {
            reportError(error, 'Export MIDI configuration');
        } finally {
            exporting = false;
        }
    }

    async function exportSaved(configurationId: string, configurationName: string) {
        if (exportingId) {
            return;
        }
        exportingId = configurationId;
        try {
            download(await downloadEngineMidiConfiguration(configurationId), configurationName);
        } catch (error) {
            reportError(error, 'Export MIDI configuration');
        } finally {
            exportingId = null;
        }
    }

    async function readSelectedFile() {
        if (!selectedFile) {
            throw new Error('Select a MIDI configuration JSON file first');
        }
        try {
            return JSON.parse(await selectedFile.text()) as StoredEngineMidiConfiguration;
        } catch (error) {
            throw new Error('The selected file is not valid JSON', { cause: error });
        }
    }

    async function importConfiguration(setActive: boolean) {
        if (importing) {
            return;
        }
        importing = true;
        errorMessage = '';
        try {
            const configuration = await readSelectedFile();
            if (setActive) {
                await importActiveEngineMidiConfiguration(configuration);
            } else {
                await addEngineMidiConfiguration(configuration);
            }
            selectedFile = null;
            if (fileInput) {
                fileInput.value = '';
            }
        } catch (error) {
            reportError(error, 'Import MIDI configuration');
        } finally {
            importing = false;
        }
    }
</script>

<section class="border-border bg-surface-card mb-3 overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="m-0 text-base font-semibold min-[56.25rem]:text-lg">
                Engine MIDI configuration file
            </h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm">
                MIDI app controls and input auto-connections in one file.
            </p>
        </div>
        <Button
            variant="secondary"
            size="md"
            busy={exporting}
            disabled={exporting}
            onclick={() => void exportCurrent()}>
            <Icon icon={exporting ? 'lucide:loader-circle' : 'lucide:download'} width="1rem" />
            {exporting ? 'Exporting…' : 'Export active'}
        </Button>
    </header>

    <form
        class="grid gap-3 p-3.75 min-[37.5rem]:grid-cols-[minmax(0,1fr)_auto] min-[37.5rem]:items-end min-[56.25rem]:p-5"
        onsubmit={(event) => {
            event.preventDefault();
            void importConfiguration(false);
        }}>
        <div class="grid gap-1.5">
            <span class="text-sm font-semibold">Import configuration</span>
            <input
                id="engine-midi-configuration-file"
                bind:this={fileInput}
                class="sr-only"
                type="file"
                accept="application/json,.json"
                onchange={(event) => (selectedFile = event.currentTarget.files?.item(0) ?? null)} />
            <label
                class="border-border bg-surface-inset hover:bg-surface-hover focus-within:ring-accent flex h-11 min-w-0 cursor-pointer items-center gap-2 rounded-lg border px-3 text-sm transition-colors focus-within:ring-2"
                for="engine-midi-configuration-file">
                <Icon class="text-foreground-muted shrink-0" icon="lucide:file-up" width="1rem" />
                <span class={selectedFile ? 'truncate' : 'text-foreground-muted truncate'}>
                    {selectedFile?.name ?? 'Choose a JSON configuration file'}
                </span>
            </label>
        </div>
        <div class="flex gap-2">
            <Button
                variant="secondary"
                size="lg"
                type="button"
                busy={importing}
                disabled={!selectedFile || importing}
                onclick={() => void importConfiguration(true)}>
                Set active
            </Button>
            <Button
                variant="primary"
                size="lg"
                type="submit"
                busy={importing}
                disabled={!selectedFile || importing}>
                Add to library
            </Button>
        </div>
    </form>
</section>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-3 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="m-0 text-base font-semibold min-[56.25rem]:text-lg">
                Stored MIDI configurations
            </h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm">
                {configurationList.length}
                {configurationList.length === 1 ? 'configuration' : 'configurations'} available
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button
                variant="secondary"
                size="lg"
                square
                busy={refreshing}
                disabled={refreshing}
                aria-label="Refresh MIDI configurations"
                title="Refresh MIDI configurations"
                onclick={() => void refresh()}>
                <Icon
                    class={refreshing ? 'animate-spin' : ''}
                    icon={refreshing ? 'lucide:loader-circle' : 'lucide:refresh-cw'}
                    width="1rem" />
            </Button>
            <Button
                variant="secondary"
                size="lg"
                square
                class="min-[37.5rem]:w-auto min-[37.5rem]:px-3"
                busy={creating}
                disabled={creating || loadingId !== null}
                aria-label="Create new empty MIDI configuration"
                title="Create new empty MIDI configuration"
                onclick={() => void createConfiguration()}>
                <Icon
                    class={creating ? 'animate-spin' : ''}
                    icon={creating ? 'lucide:loader-circle' : 'lucide:file-plus-2'}
                    width="1rem" />
                <span class="hidden text-sm font-semibold min-[37.5rem]:inline">
                    {creating ? 'Creating…' : 'New'}
                </span>
            </Button>
            <Button
                variant="primary"
                size="md"
                busy={saving}
                disabled={saving || !name.trim()}
                onclick={() => void saveCurrent()}>
                {saving ? 'Saving…' : 'Save active'}
            </Button>
        </div>
    </header>

    {#if errorMessage}
        <div class="border-danger/35 bg-danger/8 text-danger border-b px-3.75 py-3 text-sm">
            {errorMessage}
        </div>
    {/if}

    <form
        class="border-border grid gap-3 border-b px-3.75 py-3.5 min-[37.5rem]:grid-cols-[minmax(0,1fr)_minmax(12rem,24.25rem)] min-[37.5rem]:items-end min-[56.25rem]:px-5"
        onsubmit={(event) => {
            event.preventDefault();
            void rename();
        }}>
        <div>
            <label class="block text-sm font-semibold" for="active-engine-midi-configuration-name">
                Active configuration name
            </label>
            <small class="text-foreground-muted mt-1 block text-sm">
                Rename before saving a new configuration.
            </small>
        </div>
        <AutoSaveTextInput
            id="active-engine-midi-configuration-name"
            bind:value={name}
            pending={nameChanged}
            saving={renaming}
            required
            oncommit={rename} />
    </form>

    {#if configurationList.length > 0}
        <div class="divide-border divide-y">
            {#each configurationList as configuration (configuration.id)}
                <article
                    class="hover:bg-surface-hover flex min-h-19 flex-wrap items-center gap-3 px-3.75 py-3 transition-colors min-[56.25rem]:px-5">
                    <span
                        class={`grid size-10 shrink-0 place-items-center rounded-lg ${activeId === configuration.id ? 'bg-accent/12 text-accent' : 'bg-surface-inset text-foreground-muted'}`}>
                        <Icon
                            icon={activeId === configuration.id
                                ? 'lucide:settings-2'
                                : 'lucide:file-cog'}
                            width="1.125rem" />
                    </span>
                    <div class="min-w-40 flex-1">
                        <h3 class="m-0 truncate text-sm font-semibold min-[56.25rem]:text-base">
                            {displayName(configuration.name)}
                        </h3>
                        <p class="text-foreground-muted mt-1 mb-0 truncate text-xs">
                            Format v{configuration.version} · {configuration.id}
                        </p>
                    </div>
                    {#if activeId === configuration.id}
                        <span
                            class="bg-accent/10 text-accent rounded-full px-2 py-0.5 text-xs font-bold uppercase">
                            Active
                        </span>
                    {/if}
                    <div class="ml-auto flex gap-1.5">
                        <Button
                            variant="secondary"
                            size="sm"
                            disabled={activeId === configuration.id || loadingId !== null}
                            onclick={() => void load(configuration.id)}>
                            {loadingId === configuration.id ? 'Loading…' : 'Load'}
                        </Button>
                        <Button
                            variant="secondary"
                            size="md"
                            square
                            busy={exportingId === configuration.id}
                            disabled={exportingId !== null}
                            aria-label={`Export ${displayName(configuration.name)}`}
                            title={`Export ${displayName(configuration.name)}`}
                            onclick={() => void exportSaved(configuration.id, configuration.name)}>
                            <Icon icon="lucide:download" width="1rem" />
                        </Button>
                        <Button
                            variant="danger-outline"
                            size="md"
                            square
                            busy={deletingId === configuration.id}
                            disabled={deletingId !== null}
                            aria-label={`Delete ${displayName(configuration.name)}`}
                            title={`Delete ${displayName(configuration.name)}`}
                            onclick={() =>
                                void remove(configuration.id, displayName(configuration.name))}>
                            <Icon icon="lucide:trash-2" width="1rem" />
                        </Button>
                    </div>
                </article>
            {/each}
        </div>
    {:else}
        <div
            class="text-foreground-muted grid min-h-48 place-items-center content-center px-6 text-center">
            <Icon class="text-accent" icon="lucide:file-cog" width="1.5rem" />
            <h3 class="text-foreground mt-3 mb-0 text-sm font-semibold">
                No stored MIDI configurations
            </h3>
            <p class="mt-1.5 mb-0 max-w-72 text-sm">
                Save the active MIDI setup to create the first combined configuration file.
            </p>
        </div>
    {/if}
</section>
