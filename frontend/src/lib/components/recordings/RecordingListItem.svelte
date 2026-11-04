<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import type { RecordingInfo } from '$lib/generated/models';
    import type { RecordingsSection } from '$lib/sections/recordings.svelte';

    interface Props {
        section: RecordingsSection;
        recording: RecordingInfo;
    }

    let { section, recording }: Props = $props();
</script>

<article
    class="hover:bg-surface-hover grid min-h-19 grid-cols-[2.5rem_minmax(0,1fr)] items-center gap-x-3 gap-y-2 px-3.75 py-3 transition-colors min-[37.5rem]:flex min-[37.5rem]:gap-3 min-[56.25rem]:px-5">
    <span
        class={`grid size-10 shrink-0 place-items-center rounded-lg ${recording.active ? 'bg-danger/10 text-danger' : 'bg-accent/10 text-accent'}`}>
        <Icon icon={recording.active ? 'lucide:radio' : 'lucide:file-music'} width="1.125rem" />
    </span>
    <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
            <h3 class="m-0 truncate text-sm font-semibold min-[56.25rem]:text-base">
                {section.filename(recording.path)}
            </h3>
            {#if recording.active}
                <span
                    class="bg-danger/10 text-danger shrink-0 rounded-full px-2 py-0.5 text-xs font-bold uppercase">
                    Recording
                </span>
            {/if}
        </div>
        <p class="text-foreground-muted mt-1 mb-0 truncate text-xs">
            {section.formatSize(recording.size)} · {section.formatModified(
                recording.modified_unix_ms
            )}
            {#if recording.path !== section.filename(recording.path)}
                · {recording.path}
            {/if}
        </p>
    </div>
    <div class="col-span-2 flex shrink-0 items-center justify-end gap-1.5 min-[37.5rem]:col-auto">
        <Button
            variant="secondary"
            size="sm"
            disabled={section.deletingPath !== null}
            onclick={() => section.download(recording)}>
            <Icon icon="lucide:download" width="0.9375rem" />
            Download
        </Button>
        <TimedConfirmButton
            label="Delete"
            confirmLabel="Confirm delete"
            busyLabel="Deleting…"
            busy={section.deletingPath === recording.path}
            disabled={recording.active || section.deletingPath !== null}
            onconfirm={() => section.remove(recording.path)} />
    </div>
</article>
