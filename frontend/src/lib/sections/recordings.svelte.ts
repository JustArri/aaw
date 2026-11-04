import { onMount } from 'svelte';
import { errorToUserMessage } from '$lib/error';
import { getDownloadRecordingUrl } from '$lib/generated/api';
import type { RecordingInfo } from '$lib/generated/models';
import { notify } from '$lib/push-notifications.svelte';
import { deleteRecording, fetchRecordings, recordings } from '$lib/state/recordings.svelte';

export { recordings };

export class RecordingsSection {
    loading = $state(true);
    refreshing = $state(false);
    deletingPath = $state<string | null>(null);
    errorMessage = $state('');

    constructor() {
        onMount(() => this.load());
    }

    private async load() {
        try {
            await fetchRecordings();
        } catch (error) {
            this.reportError(error, 'Load recordings');
        } finally {
            this.loading = false;
        }
    }

    private reportError(error: unknown, title: string) {
        this.errorMessage = errorToUserMessage(error);
        notify.error(this.errorMessage, { title });
    }

    filename(path: string) {
        return path.split('/').at(-1) || path;
    }

    formatSize(bytes: number) {
        if (bytes < 1_024) return `${bytes} B`;
        if (bytes < 1_048_576) return `${(bytes / 1_024).toFixed(1)} KB`;
        if (bytes < 1_073_741_824) return `${(bytes / 1_048_576).toFixed(1)} MB`;
        return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
    }

    formatModified(value: number | null) {
        return value === null ? 'Unknown date' : new Date(value).toLocaleString();
    }

    download(recording: RecordingInfo) {
        const anchor = document.createElement('a');
        anchor.href = getDownloadRecordingUrl({ path: recording.path });
        anchor.download = this.filename(recording.path);
        anchor.click();
    }

    async refresh() {
        if (this.refreshing) return;
        this.refreshing = true;
        this.errorMessage = '';
        try {
            await fetchRecordings();
        } catch (error) {
            this.reportError(error, 'Refresh recordings');
        } finally {
            this.refreshing = false;
        }
    }

    async remove(path: string) {
        if (this.deletingPath) return;
        this.deletingPath = path;
        this.errorMessage = '';
        try {
            await deleteRecording(path);
        } catch (error) {
            this.reportError(error, 'Delete recording');
        } finally {
            this.deletingPath = null;
        }
    }
}
