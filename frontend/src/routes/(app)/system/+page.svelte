<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import ApplyIconButton from '$lib/components/ApplyIconButton.svelte';
    import type { Preferences } from '$lib/generated/models';
    import { notify } from '$lib/push-notifications.svelte';
    import {
        fetchPreferences,
        preferences,
        rebootSystem,
        shutDownSystem,
        updatePreferences
    } from '$lib/state/system.svelte';

    type SystemAction = 'reboot' | 'shutdown';

    let confirming = $state(null as SystemAction | null);
    let running = $state(null as SystemAction | null);
    let synthDefaults = $state<Preferences>({
        dummy_max_voices: 32,
        fluid_max_voices: 256,
        sfizz_max_voices: 64
    });
    let voiceLimitsLoading = $state(true);
    let voiceLimitsSaving = $state(false);
    let voiceLimitsLoaded = $state(false);
    let voiceLimitsChanged = $derived(
        preferences() !== null &&
            (synthDefaults.dummy_max_voices !== preferences()?.dummy_max_voices ||
                synthDefaults.fluid_max_voices !== preferences()?.fluid_max_voices ||
                synthDefaults.sfizz_max_voices !== preferences()?.sfizz_max_voices)
    );

    onMount(async () => {
        try {
            const value = await fetchPreferences();
            if (value) {
                synthDefaults = { ...value };
                voiceLimitsLoaded = true;
            }
        } catch (error) {
            notify.error(messageFor(error), { title: 'Load preferences' });
        } finally {
            voiceLimitsLoading = false;
        }
    });

    function messageFor(error: unknown) {
        if (error instanceof Error) {
            return error.message;
        }
        return 'The system action failed';
    }

    async function run(action: SystemAction) {
        if (running) {
            return;
        }

        running = action;

        try {
            if (action === 'reboot') {
                await rebootSystem();
            } else {
                await shutDownSystem();
            }
        } catch (error) {
            running = null;
            confirming = null;
            notify.error(messageFor(error), {
                title: action === 'reboot' ? 'Reboot system' : 'Power off system'
            });
        }
    }

    async function saveSynthDefaults() {
        if (voiceLimitsSaving || !voiceLimitsChanged) {
            return;
        }

        voiceLimitsSaving = true;
        try {
            await updatePreferences({ ...synthDefaults });
        } catch (error) {
            notify.error(messageFor(error), { title: 'Save preferences' });
        } finally {
            voiceLimitsSaving = false;
        }
    }
</script>

<div class="grid gap-3">
    <form
        class="border-border bg-surface-card overflow-hidden rounded-xl border"
        onsubmit={(event) => {
            event.preventDefault();
            void saveSynthDefaults();
        }}>
        <header
            class="border-border flex min-h-16.75 items-center justify-between gap-3 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
            <div>
                <h2 class="m-0 text-base min-[56.25rem]:text-lg">Synth defaults</h2>
                <p class="text-foreground-muted mt-1 mb-0 text-sm">
                    Suggested voice counts for new synths. They can be changed during creation.
                </p>
            </div>
            {#if voiceLimitsLoading}
                <Icon
                    class="text-accent shrink-0 animate-spin"
                    icon="lucide:loader-circle"
                    width="1.125rem"
                    aria-label="Loading synth defaults" />
            {:else}
                <ApplyIconButton
                    saving={voiceLimitsSaving}
                    pending={voiceLimitsChanged}
                    disabled={!voiceLimitsLoaded}
                    label="Apply synth defaults" />
            {/if}
        </header>

        <div
            class="grid gap-3 p-3.75 min-[43.75rem]:grid-cols-3 min-[56.25rem]:gap-4 min-[56.25rem]:p-5">
            <label
                class="border-border bg-surface-inset flex items-center justify-between gap-4 rounded-xl border p-3.5">
                <strong class="min-w-0 text-sm font-semibold">Dummy</strong>
                <input
                    class="border-border-strong bg-surface-card text-foreground h-10 w-20 shrink-0 rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                    type="number"
                    min="1"
                    max="65535"
                    step="1"
                    required
                    disabled={!voiceLimitsLoaded}
                    bind:value={synthDefaults.dummy_max_voices} />
            </label>

            <label
                class="border-border bg-surface-inset flex items-center justify-between gap-4 rounded-xl border p-3.5">
                <strong class="min-w-0 text-sm font-semibold">Fluid</strong>
                <input
                    class="border-border-strong bg-surface-card text-foreground h-10 w-20 shrink-0 rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                    type="number"
                    min="1"
                    max="65535"
                    step="1"
                    required
                    disabled={!voiceLimitsLoaded}
                    bind:value={synthDefaults.fluid_max_voices} />
            </label>

            <label
                class="border-border bg-surface-inset flex items-center justify-between gap-4 rounded-xl border p-3.5">
                <strong class="min-w-0 text-sm font-semibold">Sfizz</strong>
                <input
                    class="border-border-strong bg-surface-card text-foreground h-10 w-20 shrink-0 rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                    type="number"
                    min="1"
                    max="256"
                    step="1"
                    required
                    disabled={!voiceLimitsLoaded}
                    bind:value={synthDefaults.sfizz_max_voices} />
            </label>
        </div>
    </form>

    <section class="border-border bg-surface-card overflow-hidden rounded-xl border">
        <header
            class="border-border flex min-h-16.75 items-center border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
            <div>
                <h2 class="m-0 text-base min-[56.25rem]:text-lg">Power controls</h2>
                <p class="text-foreground-muted mt-1 mb-0 text-sm">
                    Safely restart or shut down this AAW device.
                </p>
            </div>
        </header>

        <div
            class="grid gap-3 p-3.75 min-[43.75rem]:grid-cols-2 min-[56.25rem]:gap-4 min-[56.25rem]:p-5">
            <article class="border-border bg-surface-inset flex flex-col rounded-xl border p-4">
                <div class="flex items-start gap-3">
                    <span
                        class="border-border-strong bg-surface-button text-foreground grid size-10 shrink-0 place-items-center rounded-lg border">
                        <Icon icon="lucide:refresh-cw" width="1.25rem" />
                    </span>
                    <div>
                        <h3 class="m-0 text-base font-semibold">Reboot</h3>
                        <p class="text-foreground-muted mt-1 mb-0 text-sm">
                            Restart the device and briefly interrupt audio and network connections.
                        </p>
                    </div>
                </div>

                <div class="mt-5 flex min-h-10 items-center justify-end gap-2">
                    {#if confirming === 'reboot'}
                        <button
                            class="border-border-strong bg-surface-button text-foreground hover:bg-surface-hover min-h-10 cursor-pointer rounded-lg border px-3 text-sm font-semibold disabled:cursor-not-allowed disabled:opacity-50"
                            type="button"
                            disabled={running !== null}
                            onclick={() => (confirming = null)}>
                            Cancel
                        </button>
                        <button
                            class="bg-danger hover:bg-danger/85 inline-flex min-h-10 cursor-pointer items-center gap-2 rounded-lg border-0 px-3.5 text-sm font-bold text-white transition-colors disabled:cursor-wait disabled:opacity-60"
                            type="button"
                            disabled={running !== null}
                            onclick={() => void run('reboot')}>
                            <Icon
                                icon={running === 'reboot'
                                    ? 'lucide:loader-circle'
                                    : 'lucide:refresh-cw'}
                                width="1rem"
                                class={running === 'reboot' ? 'animate-spin' : ''} />
                            {running === 'reboot' ? 'Rebooting…' : 'Confirm reboot'}
                        </button>
                    {:else}
                        <button
                            class="border-border-strong bg-surface-button text-foreground hover:border-accent/50 hover:bg-accent/10 hover:text-accent inline-flex min-h-10 cursor-pointer items-center gap-2 rounded-lg border px-3.5 text-sm font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-50"
                            type="button"
                            disabled={running !== null}
                            onclick={() => (confirming = 'reboot')}>
                            <Icon icon="lucide:refresh-cw" width="1rem" />
                            Reboot
                        </button>
                    {/if}
                </div>
            </article>

            <article class="border-border bg-surface-inset flex flex-col rounded-xl border p-4">
                <div class="flex items-start gap-3">
                    <span
                        class="border-danger/30 bg-danger/8 text-danger grid size-10 shrink-0 place-items-center rounded-lg border">
                        <Icon icon="lucide:power" width="1.25rem" />
                    </span>
                    <div>
                        <h3 class="m-0 text-base font-semibold">Power off</h3>
                        <p class="text-foreground-muted mt-1 mb-0 text-sm">
                            Shut down the device. Physical access is required to start it again.
                        </p>
                    </div>
                </div>

                <div class="mt-5 flex min-h-10 items-center justify-end gap-2">
                    {#if confirming === 'shutdown'}
                        <button
                            class="border-border-strong bg-surface-button text-foreground hover:bg-surface-hover min-h-10 cursor-pointer rounded-lg border px-3 text-sm font-semibold disabled:cursor-not-allowed disabled:opacity-50"
                            type="button"
                            disabled={running !== null}
                            onclick={() => (confirming = null)}>
                            Cancel
                        </button>
                        <button
                            class="bg-danger hover:bg-danger/85 inline-flex min-h-10 cursor-pointer items-center gap-2 rounded-lg border-0 px-3.5 text-sm font-bold text-white transition-colors disabled:cursor-wait disabled:opacity-60"
                            type="button"
                            disabled={running !== null}
                            onclick={() => void run('shutdown')}>
                            <Icon
                                icon={running === 'shutdown'
                                    ? 'lucide:loader-circle'
                                    : 'lucide:power'}
                                width="1rem"
                                class={running === 'shutdown' ? 'animate-spin' : ''} />
                            {running === 'shutdown' ? 'Powering off…' : 'Confirm power off'}
                        </button>
                    {:else}
                        <button
                            class="border-border-strong bg-surface-button text-foreground hover:border-danger/50 hover:bg-danger/10 hover:text-danger inline-flex min-h-10 cursor-pointer items-center gap-2 rounded-lg border px-3.5 text-sm font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-50"
                            type="button"
                            disabled={running !== null}
                            onclick={() => (confirming = 'shutdown')}>
                            <Icon icon="lucide:power" width="1rem" />
                            Power off
                        </button>
                    {/if}
                </div>
            </article>
        </div>
    </section>
</div>
