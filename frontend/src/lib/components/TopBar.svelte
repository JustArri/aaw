<script lang="ts">
    import Icon from '@iconify/svelte';
    import * as api from '$lib/generated/api';
    import Button from '$lib/components/Button.svelte';
    import { errorToUserMessage } from '$lib/error';
    import { icons } from '$lib/icons';
    import { notify } from '$lib/push-notifications.svelte';
    import { activeProject, updateActiveProjectTransport } from '$lib/state/active-project.svelte';
    import {
        audioRecording,
        setAudioRecordingPaused,
        startAudioRecording,
        stopAudioRecording
    } from '$lib/state/audio.svelte';
    import { panicEngine } from '$lib/state/engine.svelte';
    import { gatewayOnline } from '$lib/state/gateway.svelte';
    import { saveActiveProject } from '$lib/state/projects.svelte';
    import { AudioRecordingState } from '$lib/generated/models';

    let { activeLabel }: { activeLabel: string } = $props();

    const projectName = $derived(activeProject()?.name.trim() || 'Untitled project');
    const recording = $derived(audioRecording());
    let saving = $state(false);
    let changingTransport = $state(false);
    let changingRecording = $state(false);
    let panicking = $state(false);
    const saveState = $derived.by(() => {
        const project = activeProject();

        if (!project) {
            return 'unavailable';
        }
        if (!project.storage) {
            return 'new';
        }
        return project.revision === project.storage.revision ? 'saved' : 'unsaved';
    });
    const saveTitle = $derived(
        saving
            ? 'Saving project'
            : saveState === 'new'
              ? 'Save new project'
              : saveState === 'unsaved'
                ? 'Save project changes'
                : saveState === 'saved'
                  ? 'Project is saved'
                  : 'Save project'
    );

    async function onProjectSave() {
        if (saving) {
            return;
        }

        saving = true;
        try {
            await saveActiveProject();
        } catch (error) {
            notify.error(error instanceof Error ? error.message : 'Could not save project.', {
                title: 'Save project'
            });
        } finally {
            saving = false;
        }
    }

    async function changeRecording(title: string, action: () => Promise<void>) {
        if (changingRecording) {
            return;
        }

        changingRecording = true;

        try {
            await action();
        } catch (error) {
            notify.error(
                error instanceof Error ? error.message : 'Could not change recording state.',
                { title }
            );
        } finally {
            changingRecording = false;
        }
    }

    function onRecordingStart() {
        return changeRecording('Start recording', startAudioRecording);
    }

    function onRecordingPauseChange() {
        const paused = recording?.state === AudioRecordingState.Running;
        return changeRecording(paused ? 'Pause recording' : 'Resume recording', () =>
            setAudioRecordingPaused(paused)
        );
    }

    function onRecordingStop() {
        return changeRecording('Stop recording', stopAudioRecording);
    }

    async function onPanic() {
        if (panicking) {
            return;
        }

        panicking = true;
        try {
            await panicEngine();
        } catch (error) {
            notify.error(error instanceof Error ? error.message : 'Could not silence the engine.', {
                title: 'Panic'
            });
        } finally {
            panicking = false;
        }
    }

    async function onTransportToggle() {
        const project = activeProject();
        if (!project || changingTransport) return;

        const previous = project.transport;
        const next = { ...previous, playing: !previous.playing };
        changingTransport = true;
        updateActiveProjectTransport(next);
        try {
            const response = await api.setTransport(next);
            if (response.status !== 204) {
                throw new Error(`Transport update failed (${response.status})`);
            }
        } catch (cause) {
            updateActiveProjectTransport(previous);
            notify.error(errorToUserMessage(cause), { title: 'Update transport' });
        } finally {
            changingTransport = false;
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
    <div class="flex items-center gap-1.5">
        <Button
            variant={activeProject()?.transport.playing ? 'danger-soft' : 'accent-outline'}
            size="sm"
            square
            class="size-8.5"
            type="button"
            aria-label={activeProject()?.transport.playing ? 'Stop transport' : 'Play transport'}
            title={activeProject()?.transport.playing ? 'Stop transport' : 'Play transport'}
            busy={changingTransport}
            disabled={changingTransport || !gatewayOnline() || !activeProject()}
            onclick={onTransportToggle}>
            <Icon
                icon={changingTransport
                    ? 'lucide:loader-circle'
                    : activeProject()?.transport.playing
                      ? 'lucide:square'
                      : 'lucide:play'}
                width="1rem"
                class={changingTransport ? 'animate-spin' : ''} />
        </Button>
        <span class="bg-border mx-0.5 h-6 w-px shrink-0" aria-hidden="true"></span>
        {#if recording?.state === AudioRecordingState.Running || recording?.state === AudioRecordingState.Paused}
            <div
                class="border-danger/35 bg-danger/8 flex items-center rounded-lg border"
                title={recording.output_file ?? 'Audio recording'}>
                <span
                    class={`bg-danger ml-2 size-2 rounded-full ${recording.state === AudioRecordingState.Running ? 'animate-pulse' : 'opacity-45'}`}
                    aria-hidden="true">
                </span>
                <Button
                    variant="danger-ghost"
                    size="sm"
                    square
                    class="size-8.5"
                    type="button"
                    aria-label={recording.state === AudioRecordingState.Running
                        ? 'Pause recording'
                        : 'Resume recording'}
                    title={recording.state === AudioRecordingState.Running
                        ? 'Pause recording'
                        : 'Resume recording'}
                    busy={changingRecording}
                    disabled={changingRecording || !gatewayOnline()}
                    onclick={onRecordingPauseChange}>
                    <Icon
                        icon={changingRecording
                            ? 'lucide:loader-circle'
                            : recording.state === AudioRecordingState.Running
                              ? 'lucide:pause'
                              : 'lucide:play'}
                        width="1rem"
                        class={changingRecording ? 'animate-spin' : ''} />
                </Button>
                <Button
                    variant="danger-ghost"
                    size="sm"
                    square
                    class="size-8.5"
                    type="button"
                    aria-label="Stop recording"
                    title="Stop recording"
                    busy={changingRecording}
                    disabled={changingRecording || !gatewayOnline()}
                    onclick={onRecordingStop}>
                    <Icon icon="lucide:square" width="0.9375rem" />
                </Button>
            </div>
        {:else}
            <Button
                variant="danger-outline"
                size="sm"
                square
                class="bg-surface-card size-8.5"
                type="button"
                aria-label="Start recording"
                title="Start recording"
                busy={changingRecording}
                disabled={changingRecording || !gatewayOnline() || recording === null}
                onclick={onRecordingStart}>
                {#if changingRecording}
                    <Icon icon="lucide:loader-circle" width="1rem" class="animate-spin" />
                {:else}
                    <span class="bg-danger size-3 rounded-full" aria-hidden="true"></span>
                {/if}
            </Button>
        {/if}
        <Button
            variant="danger-soft"
            size="sm"
            square
            class="size-8.5"
            type="button"
            aria-label="Panic: silence all synths"
            title="Panic: silence all synths"
            busy={panicking}
            disabled={panicking || !gatewayOnline()}
            onclick={onPanic}>
            <Icon
                icon={panicking ? 'lucide:loader-circle' : 'lucide:octagon-alert'}
                width="1.0625rem"
                class={panicking ? 'animate-spin' : ''} />
        </Button>
        <span class="bg-border mx-0.5 h-6 w-px shrink-0" aria-hidden="true"></span>
        <Button
            variant={saveState === 'new' || saveState === 'unsaved'
                ? 'accent-outline'
                : 'secondary'}
            size="sm"
            square
            class="size-8.5"
            type="button"
            aria-label={saveTitle}
            title={saveTitle}
            busy={saving}
            disabled={saving || !activeProject()}
            onclick={onProjectSave}>
            <Icon
                icon={saving
                    ? 'lucide:loader-circle'
                    : saveState === 'new'
                      ? 'lucide:save-all'
                      : icons.save}
                width="1.0625rem"
                class={saving ? 'animate-spin' : ''} />
        </Button>
        <span class="bg-border mx-0.5 h-6 w-px shrink-0" aria-hidden="true"></span>
        <a
            class="border-border-strong bg-surface-card text-foreground-strong grid size-8.5 place-items-center rounded-lg border"
            href="/settings"
            aria-label="Settings">
            <Icon icon={icons.settings} width="1.0625rem" />
        </a>
    </div>
</header>
