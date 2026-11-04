import { onMount } from 'svelte';
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

export class SystemSection {
    confirming = $state<SystemAction | null>(null);
    running = $state<SystemAction | null>(null);
    synthDefaults = $state<Preferences>({
        dummy_max_voices: 32,
        fluid_max_voices: 256,
        sfizz_max_voices: 64,
        sfizz_preload_size: 65_536
    });
    voiceLimitsLoading = $state(true);
    voiceLimitsSaving = $state(false);
    voiceLimitsLoaded = $state(false);
    voiceLimitsChanged = $derived(
        preferences() !== null &&
            (this.synthDefaults.dummy_max_voices !== preferences()?.dummy_max_voices ||
                this.synthDefaults.fluid_max_voices !== preferences()?.fluid_max_voices ||
                this.synthDefaults.sfizz_max_voices !== preferences()?.sfizz_max_voices ||
                this.synthDefaults.sfizz_preload_size !== preferences()?.sfizz_preload_size)
    );

    constructor() {
        onMount(() => this.load());
    }

    private async load() {
        try {
            const value = await fetchPreferences();
            if (value) {
                this.synthDefaults = { ...value };
                this.voiceLimitsLoaded = true;
            }
        } catch (error) {
            notify.error(this.messageFor(error), { title: 'Load preferences' });
        } finally {
            this.voiceLimitsLoading = false;
        }
    }

    private messageFor(error: unknown) {
        return error instanceof Error ? error.message : 'The system action failed';
    }

    async run(action: SystemAction) {
        if (this.running) return;
        this.running = action;
        try {
            if (action === 'reboot') await rebootSystem();
            else await shutDownSystem();
        } catch (error) {
            this.running = null;
            this.confirming = null;
            notify.error(this.messageFor(error), {
                title: action === 'reboot' ? 'Reboot system' : 'Power off system'
            });
        }
    }

    async saveSynthDefaults() {
        if (this.voiceLimitsSaving || !this.voiceLimitsChanged) return;
        this.voiceLimitsSaving = true;
        try {
            await updatePreferences({ ...this.synthDefaults });
        } catch (error) {
            notify.error(this.messageFor(error), { title: 'Save preferences' });
        } finally {
            this.voiceLimitsSaving = false;
        }
    }
}
