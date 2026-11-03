<script lang="ts">
    import Icon from '@iconify/svelte';
    import { onMount } from 'svelte';
    import Button from '$lib/components/Button.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import { errorToUserMessage } from '$lib/error';
    import { getDownloadRecordingUrl } from '$lib/generated/api';
    import type { RecordingInfo } from '$lib/generated/models';
    import { notify } from '$lib/push-notifications.svelte';
    import { deleteRecording, fetchRecordings, recordings } from '$lib/state/recordings.svelte';

    let loading = $state(true);
    let refreshing = $state(false);
    let deletingPath = $state<string | null>(null);
    let errorMessage = $state('');

    onMount(async () => {
        try {
            await fetchRecordings();
        } catch (error) {
            reportError(error, 'Load recordings');
        } finally {
            loading = false;
        }
    });

    function reportError(error: unknown, title: string) {
        errorMessage = errorToUserMessage(error);
        notify.error(errorMessage, { title });
    }

    function filename(path: string) {
        return path.split('/').at(-1) || path;
    }

    function formatSize(bytes: number) {
        if (bytes < 1_024) {
            return `${bytes} B`;
        }
        if (bytes < 1_048_576) {
            return `${(bytes / 1_024).toFixed(1)} KB`;
        }
        if (bytes < 1_073_741_824) {
            return `${(bytes / 1_048_576).toFixed(1)} MB`;
        }
        return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
    }

    function formatModified(value: number | null) {
        return value === null ? 'Unknown date' : new Date(value).toLocaleString();
    }

    function download(recording: RecordingInfo) {
        const anchor = document.createElement('a');
        anchor.href = getDownloadRecordingUrl({ path: recording.path });
        anchor.download = filename(recording.path);
        anchor.click();
    }

    async function refresh() {
        if (refreshing) {
            return;
        }
        refreshing = true;
        errorMessage = '';
        try {
            await fetchRecordings();
        } catch (error) {
            reportError(error, 'Refresh recordings');
        } finally {
            refreshing = false;
        }
    }

    async function remove(path: string) {
        if (deletingPath) {
            return;
        }
        deletingPath = path;
        errorMessage = '';
        try {
            await deleteRecording(path);
        } catch (error) {
            reportError(error, 'Delete recording');
        } finally {
            deletingPath = null;
        }
    }
</script>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="m-0 text-base font-semibold min-[56.25rem]:text-lg">Recordings</h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm">
                {recordings().length}
                {recordings().length === 1 ? 'file' : 'files'} available
            </p>
        </div>
        <Button
            variant="secondary"
            size="lg"
            square
            busy={refreshing}
            disabled={refreshing}
            aria-label="Refresh recordings"
            title="Refresh recordings"
            onclick={() => void refresh()}>
            <Icon
                class={refreshing ? 'animate-spin' : ''}
                icon={refreshing ? 'lucide:loader-circle' : 'lucide:refresh-cw'}
                width="1rem" />
        </Button>
    </header>

    {#if errorMessage}
        <div class="border-danger/35 bg-danger/8 text-danger border-b px-3.75 py-3 text-sm">
            {errorMessage}
        </div>
    {/if}

    {#if loading}
        <div class="text-foreground-muted grid min-h-48 place-items-center">
            <Icon class="animate-spin" icon="lucide:loader-circle" width="1.5rem" />
        </div>
    {:else if recordings().length > 0}
        <div class="divide-border divide-y">
            {#each recordings() as recording (recording.path)}
                <article
                    class="hover:bg-surface-hover grid min-h-19 grid-cols-[2.5rem_minmax(0,1fr)] items-center gap-x-3 gap-y-2 px-3.75 py-3 transition-colors min-[37.5rem]:flex min-[37.5rem]:gap-3 min-[56.25rem]:px-5">
                    <span
                        class={`grid size-10 shrink-0 place-items-center rounded-lg ${recording.active ? 'bg-danger/10 text-danger' : 'bg-accent/10 text-accent'}`}>
                        <Icon
                            icon={recording.active ? 'lucide:radio' : 'lucide:file-music'}
                            width="1.125rem" />
                    </span>
                    <div class="min-w-0 flex-1">
                        <div class="flex items-center gap-2">
                            <h3 class="m-0 truncate text-sm font-semibold min-[56.25rem]:text-base">
                                {filename(recording.path)}
                            </h3>
                            {#if recording.active}
                                <span
                                    class="bg-danger/10 text-danger shrink-0 rounded-full px-2 py-0.5 text-xs font-bold uppercase">
                                    Recording
                                </span>
                            {/if}
                        </div>
                        <p class="text-foreground-muted mt-1 mb-0 truncate text-xs">
                            {formatSize(recording.size)} · {formatModified(
                                recording.modified_unix_ms
                            )}
                            {#if recording.path !== filename(recording.path)}
                                · {recording.path}
                            {/if}
                        </p>
                    </div>
                    <div
                        class="col-span-2 flex shrink-0 items-center justify-end gap-1.5 min-[37.5rem]:col-auto">
                        <Button
                            variant="secondary"
                            size="sm"
                            disabled={deletingPath !== null}
                            onclick={() => download(recording)}>
                            <Icon icon="lucide:download" width="0.9375rem" />
                            Download
                        </Button>
                        <TimedConfirmButton
                            label="Delete"
                            confirmLabel="Confirm delete"
                            busyLabel="Deleting…"
                            busy={deletingPath === recording.path}
                            disabled={recording.active || deletingPath !== null}
                            onconfirm={() => remove(recording.path)} />
                    </div>
                </article>
            {/each}
        </div>
    {:else}
        <div
            class="text-foreground-muted grid min-h-56 place-items-center content-center px-6 text-center">
            <span class="bg-accent/10 text-accent grid size-12 place-items-center rounded-xl">
                <Icon icon="lucide:file-music" width="1.5rem" />
            </span>
            <h3 class="text-foreground mt-3 mb-0 text-sm font-semibold">No recordings</h3>
            <p class="mt-1.5 mb-0 max-w-72 text-sm">
                Finished audio recordings will appear here for download or deletion.
            </p>
        </div>
    {/if}
</section>
