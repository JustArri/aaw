<script lang="ts">
    import Icon from '@iconify/svelte';
    import { flip } from 'svelte/animate';
    import { onMount } from 'svelte';
    import Button from '$lib/components/Button.svelte';
    import EffectNumberField from '$lib/components/EffectNumberField.svelte';
    import EqualizerEditor from '$lib/components/EqualizerEditor.svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import type { Effect, Effects } from '$lib/generated/models';
    import { notify } from '$lib/push-notifications.svelte';
    import { effectsTuning, fetchEffectsTuning } from '$lib/state/effects.svelte';
    import { audioOutputStream } from '$lib/state/audio.svelte';

    type EffectKind =
        | 'Gain'
        | 'LowPassFilter'
        | 'HighPassFilter'
        | 'Equalizer'
        | 'Compressor'
        | 'Limiter'
        | 'Delay'
        | 'Reverb'
        | 'Chorus'
        | 'Distortion';
    type ExistingEffectKind = EffectKind | 'LoadFailed';

    const addableEffects: { kind: EffectKind; label: string }[] = [
        { kind: 'Gain', label: 'Gain' },
        { kind: 'LowPassFilter', label: 'Low-pass' },
        { kind: 'HighPassFilter', label: 'High-pass' },
        { kind: 'Equalizer', label: 'Equalizer' },
        { kind: 'Compressor', label: 'Compressor' },
        { kind: 'Limiter', label: 'Limiter' },
        { kind: 'Delay', label: 'Delay' },
        { kind: 'Reverb', label: 'Reverb' },
        { kind: 'Chorus', label: 'Chorus' },
        { kind: 'Distortion', label: 'Distortion' }
    ];

    let {
        title = 'Effects',
        effects,
        onsave
    }: {
        title?: string;
        effects: Effects;
        onsave: (effects: Effects) => Promise<void>;
    } = $props();

    function clone<T>(value: T): T {
        return JSON.parse(JSON.stringify(value)) as T;
    }

    let draft = $state<Effect[]>([]);
    let baseline = $state('');
    let lastIncoming = $state('');
    let open = $state(false);
    let dialog = $state<HTMLDialogElement>();
    let draggedEffect = $state<Effect | null>(null);
    let dragX = $state(0);
    let dragY = $state(0);
    let dropInsertionIndex = $state<number | null>(null);
    let initialized = false;
    let saving = $state(false);
    let error = $state('');
    let dirty = $derived(JSON.stringify(draft) !== baseline);
    let atEffectCapacity = $derived(
        effectsTuning() !== null && draft.length >= effectsTuning()!.max_effects
    );

    $effect(() => {
        const incoming = JSON.stringify(effects.effects);
        if (!initialized) {
            draft = clone(effects.effects);
            baseline = incoming;
            lastIncoming = incoming;
            initialized = true;
            return;
        }
        if (incoming === lastIncoming) {
            return;
        }
        const wasClean = JSON.stringify(draft) === baseline;
        lastIncoming = incoming;
        if (wasClean) {
            draft = clone(effects.effects);
            baseline = incoming;
        }
    });

    $effect(() => {
        if (!dialog) {
            return;
        }
        if (open && !dialog.open) {
            dialog.showModal();
        } else if (!open && dialog.open) {
            dialog.close();
        }
    });

    onMount(() => {
        void fetchEffectsTuning().catch((cause) => {
            notify.error(cause instanceof Error ? cause.message : String(cause), {
                title: 'Load effects configuration'
            });
        });
    });

    function effectKind(effect: Effect): ExistingEffectKind {
        return effect.type;
    }

    function effectLabel(effect: Effect) {
        switch (effectKind(effect)) {
            case 'Gain':
                return 'Gain';
            case 'LowPassFilter':
                return 'Low-pass filter';
            case 'HighPassFilter':
                return 'High-pass filter';
            case 'Equalizer':
                return 'Equalizer';
            case 'Compressor':
                return 'Compressor';
            case 'Limiter':
                return 'Limiter';
            case 'Delay':
                return 'Delay';
            case 'Reverb':
                return 'Reverb';
            case 'Chorus':
                return 'Chorus';
            case 'Distortion':
                return 'Distortion';
            case 'LoadFailed':
                return 'Failed effect';
        }
    }

    function newEffect(kind: EffectKind): Effect {
        switch (kind) {
            case 'Gain':
                return { enabled: true, type: 'Gain', data: { value: 1 } };
            case 'LowPassFilter':
                return {
                    enabled: true,
                    type: 'LowPassFilter',
                    data: { frequency: 1000, resonance: 0.707 }
                };
            case 'HighPassFilter':
                return {
                    enabled: true,
                    type: 'HighPassFilter',
                    data: { frequency: 80, resonance: 0.707 }
                };
            case 'Equalizer':
                return { enabled: true, type: 'Equalizer', data: { bands: [] } };
            case 'Compressor':
                return {
                    enabled: true,
                    type: 'Compressor',
                    data: {
                        threshold_db: -18,
                        ratio: 4,
                        attack_ms: 10,
                        release_ms: 100,
                        makeup_gain_db: 0
                    }
                };
            case 'Limiter':
                return {
                    enabled: true,
                    type: 'Limiter',
                    data: { threshold_db: -1, release_ms: 50 }
                };
            case 'Delay':
                return {
                    enabled: true,
                    type: 'Delay',
                    data: { time_ms: 350, feedback: 0.35, mix: 0.25 }
                };
            case 'Reverb':
                return {
                    enabled: true,
                    type: 'Reverb',
                    data: { room_size: 0.5, damping: 0.4, mix: 0.25 }
                };
            case 'Chorus':
                return {
                    enabled: true,
                    type: 'Chorus',
                    data: { rate_hz: 0.8, depth_ms: 3, delay_ms: 12, mix: 0.3 }
                };
            case 'Distortion':
                return {
                    enabled: true,
                    type: 'Distortion',
                    data: { drive_db: 12, mix: 0.5 }
                };
        }
    }

    function percent(value: number) {
        return Math.round(value * 100);
    }

    function linearGainToDecibels(value: number) {
        if (value <= 0) {
            return -100;
        }
        return Math.round(20 * Math.log10(value) * 10) / 10;
    }

    function decibelsToLinearGain(value: number) {
        return 10 ** (value / 20);
    }

    function addEffect(kind: EffectKind) {
        if (atEffectCapacity) {
            return;
        }
        draft.push(newEffect(kind));
    }

    function removeEffect(index: number) {
        draft.splice(index, 1);
    }

    function moveEffect(index: number, offset: -1 | 1) {
        const destination = index + offset;
        if (destination < 0 || destination >= draft.length) {
            return;
        }
        const [effect] = draft.splice(index, 1);
        draft.splice(destination, 0, effect);
    }

    function startEffectDrag(event: PointerEvent, effect: Effect) {
        if (event.button !== 0) {
            return;
        }
        const handle = event.currentTarget as HTMLElement;
        handle.setPointerCapture(event.pointerId);
        event.preventDefault();
        draggedEffect = effect;
        dragX = event.clientX;
        dragY = event.clientY;
        dropInsertionIndex = draft.indexOf(effect);
    }

    function dragEffect(event: PointerEvent) {
        if (!draggedEffect) {
            return;
        }
        dragX = event.clientX;
        dragY = event.clientY;
        const target = document
            .elementFromPoint(event.clientX, event.clientY)
            ?.closest<HTMLElement>('[data-effect-index]');
        const targetIndex = Number(target?.dataset.effectIndex);
        if (!target || !Number.isInteger(targetIndex)) {
            return;
        }
        const bounds = target.getBoundingClientRect();
        dropInsertionIndex = targetIndex + (event.clientY > bounds.top + bounds.height / 2 ? 1 : 0);
    }

    function stopEffectDrag(event: PointerEvent) {
        const handle = event.currentTarget as HTMLElement;
        if (handle.hasPointerCapture(event.pointerId)) {
            handle.releasePointerCapture(event.pointerId);
        }
        if (draggedEffect && dropInsertionIndex !== null) {
            const source = draft.indexOf(draggedEffect);
            let destination = dropInsertionIndex;
            if (destination > source) {
                destination -= 1;
            }
            if (source >= 0 && destination !== source) {
                const [effect] = draft.splice(source, 1);
                draft.splice(destination, 0, effect);
            }
        }
        draggedEffect = null;
        dropInsertionIndex = null;
    }

    function cancelEffectDrag(event: PointerEvent) {
        const handle = event.currentTarget as HTMLElement;
        if (handle.hasPointerCapture(event.pointerId)) {
            handle.releasePointerCapture(event.pointerId);
        }
        draggedEffect = null;
        dropInsertionIndex = null;
    }

    function closeEditor() {
        open = false;
    }

    function discard() {
        draft = clone(effects.effects);
        baseline = JSON.stringify(effects.effects);
        lastIncoming = baseline;
        error = '';
    }

    async function save() {
        if (!dirty || saving) {
            return;
        }
        saving = true;
        error = '';
        try {
            const next = { ...effects, effects: clone(draft) };
            await onsave(next);
            baseline = JSON.stringify(next.effects);
            lastIncoming = baseline;
        } catch (cause) {
            error = cause instanceof Error ? cause.message : String(cause);
            notify.error(error, { title });
        } finally {
            saving = false;
        }
    }
</script>

<button
    class="border-border bg-surface-card hover:bg-surface-hover flex min-h-12 w-full cursor-pointer items-center gap-3 rounded-xl border px-3 text-left transition-colors"
    type="button"
    onclick={() => (open = true)}>
    <span class="bg-surface-button grid size-8 shrink-0 place-items-center rounded-lg">
        <Icon icon="lucide:audio-lines" width="1rem" />
    </span>
    <span class="min-w-0 flex-1">
        <strong class="block truncate text-sm">{title}</strong>
        <small class="text-foreground-muted block text-xs">
            {draft.length}
            {draft.length === 1 ? 'effect' : 'effects'}
        </small>
    </span>
    {#if dirty}<span class="bg-accent size-2 rounded-full" title="Unsaved changes"></span>{/if}
    <Icon class="text-foreground-muted" icon="lucide:expand" width="1rem" />
</button>

<dialog
    bind:this={dialog}
    class="border-border-strong bg-surface-overlay backdrop:bg-scrim/70 m-0 h-dvh max-h-none w-full max-w-none flex-col overflow-hidden border-0 p-0 backdrop:backdrop-blur-[0.125rem] open:flex min-[56.25rem]:m-auto min-[56.25rem]:h-[calc(100dvh-3rem)] min-[56.25rem]:w-[min(calc(100vw-3rem),75rem)] min-[56.25rem]:rounded-2xl min-[56.25rem]:border"
    aria-label={title}
    onclick={(event) => {
        if (event.target === event.currentTarget) {
            closeEditor();
        }
    }}
    onclose={() => (open = false)}>
    <header
        class="border-border flex min-h-16 shrink-0 items-center justify-between gap-3 border-b px-3 min-[37.5rem]:px-5">
        <div class="flex min-w-0 items-center gap-3">
            <span
                class="bg-accent/10 text-accent grid size-9 shrink-0 place-items-center rounded-lg">
                <Icon icon="lucide:audio-lines" width="1.125rem" />
            </span>
            <div class="min-w-0">
                <h2 class="m-0 truncate text-base font-semibold">{title}</h2>
                <p class="text-foreground-muted m-0 text-xs">
                    {draft.length}
                    {draft.length === 1 ? 'effect' : 'effects'} in signal order
                </p>
            </div>
        </div>
        <div class="flex shrink-0 items-center gap-1">
            <Button
                variant="ghost"
                size="sm"
                type="button"
                aria-label="Discard effect changes"
                title="Discard changes"
                disabled={!dirty || saving}
                onclick={discard}>
                <Icon icon="lucide:undo-2" width="1rem" />
                <span class="hidden min-[37.5rem]:inline">Discard</span>
            </Button>
            <Button
                variant="primary"
                size="sm"
                type="button"
                aria-label="Apply effects"
                title="Apply effects"
                busy={saving}
                disabled={!dirty || saving}
                onclick={() => void save()}>
                <Icon
                    icon={saving ? 'lucide:loader-circle' : 'lucide:check'}
                    width="1.125rem"
                    class={saving ? 'animate-spin' : ''} />
                <span class="hidden min-[37.5rem]:inline">Apply</span>
            </Button>
            <Button
                variant="ghost"
                size="md"
                square
                type="button"
                aria-label="Close effects editor"
                onclick={closeEditor}>
                <Icon icon="lucide:x" width="1.125rem" />
            </Button>
        </div>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto">
        <div class="mx-auto grid max-w-5xl gap-4 p-3 min-[37.5rem]:p-5">
            {#if error}<p class="text-danger m-0 rounded-lg px-1 text-xs">{error}</p>{/if}

            <div class="grid gap-4">
                {#each draft as effect, index (effect)}
                    <div
                        class="relative"
                        data-effect-index={index}
                        animate:flip={{ duration: 160 }}>
                        {#if draggedEffect && dropInsertionIndex === index}<span
                                class="bg-accent pointer-events-none absolute -top-2 right-2 left-2 z-10 h-1 rounded-full shadow-[0_0_0.5rem_var(--accent-glow)]">
                            </span>{/if}
                        {#if draggedEffect && dropInsertionIndex === draft.length && index === draft.length - 1}<span
                                class="bg-accent pointer-events-none absolute right-2 -bottom-2 left-2 z-10 h-1 rounded-full shadow-[0_0_0.5rem_var(--accent-glow)]">
                            </span>{/if}
                        <article
                            class={`border-border bg-surface-card overflow-hidden rounded-xl border transition-opacity ${draggedEffect === effect ? 'opacity-40' : ''}`}>
                            <header class="flex min-h-12 items-center gap-1 px-3">
                                <button
                                    class="text-foreground-muted hover:bg-surface-hover grid size-8 shrink-0 cursor-grab touch-none place-items-center rounded-md border-0 bg-transparent active:cursor-grabbing"
                                    type="button"
                                    aria-label={`Drag ${effectLabel(effect)}`}
                                    title="Drag to reorder"
                                    onpointerdown={(event) => startEffectDrag(event, effect)}
                                    onpointermove={dragEffect}
                                    onpointerup={stopEffectDrag}
                                    onpointercancel={cancelEffectDrag}>
                                    <Icon icon="lucide:grip-vertical" width="1rem" />
                                </button>
                                <span
                                    class="text-foreground-muted w-6 text-center font-mono text-xs">
                                    {index + 1}
                                </span>
                                <strong class="min-w-0 flex-1 truncate text-sm">
                                    {effectLabel(effect)}
                                </strong>
                                <button
                                    class={`grid size-8 cursor-pointer place-items-center rounded-md border transition-colors ${effect.enabled ? 'border-accent/40 bg-accent/12 text-accent' : 'border-border text-foreground-muted hover:bg-surface-hover bg-transparent'}`}
                                    type="button"
                                    aria-label={`${effect.enabled ? 'Disable' : 'Enable'} ${effectLabel(effect)}`}
                                    aria-pressed={effect.enabled}
                                    title={effect.enabled ? 'Disable effect' : 'Enable effect'}
                                    onclick={() => (effect.enabled = !effect.enabled)}>
                                    <Icon icon="lucide:power" width="0.875rem" />
                                </button>
                                <button
                                    class="text-foreground-muted hover:bg-surface-hover grid size-8 cursor-pointer place-items-center rounded-md border-0 bg-transparent disabled:opacity-25"
                                    type="button"
                                    aria-label="Move effect up"
                                    disabled={index === 0}
                                    onclick={() => moveEffect(index, -1)}>
                                    <Icon icon="lucide:arrow-up" width="0.875rem" />
                                </button>
                                <button
                                    class="text-foreground-muted hover:bg-surface-hover grid size-8 cursor-pointer place-items-center rounded-md border-0 bg-transparent disabled:opacity-25"
                                    type="button"
                                    aria-label="Move effect down"
                                    disabled={index === draft.length - 1}
                                    onclick={() => moveEffect(index, 1)}>
                                    <Icon icon="lucide:arrow-down" width="0.875rem" />
                                </button>
                                <button
                                    class="text-foreground-muted hover:bg-danger/10 hover:text-danger grid size-8 cursor-pointer place-items-center rounded-md border-0 bg-transparent"
                                    type="button"
                                    aria-label="Remove effect"
                                    onclick={() => removeEffect(index)}>
                                    <Icon icon="lucide:trash-2" width="0.875rem" />
                                </button>
                            </header>

                            {#if effect.type === 'Gain'}
                                <div class="border-border border-t p-4">
                                    <NumericControl
                                        label="Gain"
                                        value={linearGainToDecibels(effect.data.value)}
                                        unit="dB"
                                        smallStep={0.5}
                                        largeStep={5}
                                        resetValue={0}
                                        min={-100}
                                        max={100}
                                        precision={1}
                                        wideValue
                                        compact
                                        onchange={(value) => {
                                            effect.data.value = decibelsToLinearGain(value);
                                        }} />
                                </div>
                            {:else if effect.type === 'LowPassFilter'}
                                <div
                                    class="border-border grid gap-3 border-t p-4 min-[37.5rem]:grid-cols-2">
                                    <EffectNumberField
                                        label="Frequency · Hz"
                                        min={0.01}
                                        step={1}
                                        bind:value={effect.data.frequency} />
                                    <EffectNumberField
                                        label="Resonance · Q"
                                        min={0.001}
                                        step={0.01}
                                        bind:value={effect.data.resonance} />
                                </div>
                            {:else if effect.type === 'HighPassFilter'}
                                <div
                                    class="border-border grid gap-3 border-t p-4 min-[37.5rem]:grid-cols-2">
                                    <EffectNumberField
                                        label="Frequency · Hz"
                                        min={0.01}
                                        step={1}
                                        bind:value={effect.data.frequency} />
                                    <EffectNumberField
                                        label="Resonance · Q"
                                        min={0.001}
                                        step={0.01}
                                        bind:value={effect.data.resonance} />
                                </div>
                            {:else if effect.type === 'Equalizer'}
                                <EqualizerEditor
                                    bind:bands={effect.data.bands}
                                    maxBands={effectsTuning()?.max_equalizer_bands ?? 16}
                                    sampleRate={audioOutputStream()?.sample_rate ?? 48_000} />
                            {:else if effect.type === 'Compressor'}
                                <div
                                    class="border-border grid gap-3 border-t p-4 min-[37.5rem]:grid-cols-2 min-[56.25rem]:grid-cols-5">
                                    <EffectNumberField
                                        label="Threshold · dB"
                                        max={0}
                                        step={0.5}
                                        bind:value={effect.data.threshold_db} />
                                    <EffectNumberField
                                        label="Ratio · :1"
                                        min={1}
                                        step={0.1}
                                        bind:value={effect.data.ratio} />
                                    <EffectNumberField
                                        label="Attack · ms"
                                        min={0.01}
                                        step={0.1}
                                        bind:value={effect.data.attack_ms} />
                                    <EffectNumberField
                                        label="Release · ms"
                                        min={0.01}
                                        step={1}
                                        bind:value={effect.data.release_ms} />
                                    <EffectNumberField
                                        label="Makeup · dB"
                                        step={0.5}
                                        bind:value={effect.data.makeup_gain_db} />
                                </div>
                            {:else if effect.type === 'Limiter'}
                                <div
                                    class="border-border grid gap-3 border-t p-4 min-[37.5rem]:grid-cols-2">
                                    <EffectNumberField
                                        label="Ceiling · dB"
                                        max={0}
                                        step={0.1}
                                        bind:value={effect.data.threshold_db} />
                                    <EffectNumberField
                                        label="Release · ms"
                                        min={0.01}
                                        step={1}
                                        bind:value={effect.data.release_ms} />
                                </div>
                            {:else if effect.type === 'Delay'}
                                <div
                                    class="border-border grid gap-3 border-t p-4 min-[37.5rem]:grid-cols-3">
                                    <EffectNumberField
                                        label="Time · ms"
                                        min={1}
                                        max={effectsTuning()?.max_delay_ms ?? 5000}
                                        step={1}
                                        bind:value={effect.data.time_ms} />
                                    <EffectNumberField
                                        label="Feedback · %"
                                        min={0}
                                        max={99}
                                        step={1}
                                        value={percent(effect.data.feedback)}
                                        onchange={(value) =>
                                            (effect.data.feedback = value / 100)} />
                                    <EffectNumberField
                                        label="Wet mix · %"
                                        min={0}
                                        max={100}
                                        step={1}
                                        value={percent(effect.data.mix)}
                                        onchange={(value) => (effect.data.mix = value / 100)} />
                                </div>
                            {:else if effect.type === 'Reverb'}
                                <div
                                    class="border-border grid gap-3 border-t p-4 min-[37.5rem]:grid-cols-3">
                                    <EffectNumberField
                                        label="Room size · %"
                                        min={0}
                                        max={100}
                                        step={1}
                                        value={percent(effect.data.room_size)}
                                        onchange={(value) =>
                                            (effect.data.room_size = value / 100)} />
                                    <EffectNumberField
                                        label="Damping · %"
                                        min={0}
                                        max={100}
                                        step={1}
                                        value={percent(effect.data.damping)}
                                        onchange={(value) => (effect.data.damping = value / 100)} />
                                    <EffectNumberField
                                        label="Wet mix · %"
                                        min={0}
                                        max={100}
                                        step={1}
                                        value={percent(effect.data.mix)}
                                        onchange={(value) => (effect.data.mix = value / 100)} />
                                </div>
                            {:else if effect.type === 'Chorus'}
                                <div
                                    class="border-border grid gap-3 border-t p-4 min-[37.5rem]:grid-cols-2 min-[56.25rem]:grid-cols-4">
                                    <EffectNumberField
                                        label="Rate · Hz"
                                        min={0.01}
                                        step={0.1}
                                        bind:value={effect.data.rate_hz} />
                                    <EffectNumberField
                                        label="Depth · ms"
                                        min={0}
                                        max={effectsTuning()?.max_chorus_delay_ms ?? 50}
                                        step={0.1}
                                        bind:value={effect.data.depth_ms} />
                                    <EffectNumberField
                                        label="Delay · ms"
                                        min={0.1}
                                        max={effectsTuning()?.max_chorus_delay_ms ?? 50}
                                        step={0.1}
                                        bind:value={effect.data.delay_ms} />
                                    <EffectNumberField
                                        label="Wet mix · %"
                                        min={0}
                                        max={100}
                                        step={1}
                                        value={percent(effect.data.mix)}
                                        onchange={(value) => (effect.data.mix = value / 100)} />
                                </div>
                            {:else if effect.type === 'Distortion'}
                                <div
                                    class="border-border grid gap-3 border-t p-4 min-[37.5rem]:grid-cols-2">
                                    <EffectNumberField
                                        label="Drive · dB"
                                        min={0}
                                        step={0.5}
                                        bind:value={effect.data.drive_db} />
                                    <EffectNumberField
                                        label="Wet mix · %"
                                        min={0}
                                        max={100}
                                        step={1}
                                        value={percent(effect.data.mix)}
                                        onchange={(value) => (effect.data.mix = value / 100)} />
                                </div>
                            {:else if effect.type === 'LoadFailed'}
                                <p
                                    class="border-danger/25 bg-danger/8 text-danger m-0 border-t p-4 text-sm">
                                    {effect.data.description}
                                </p>
                            {/if}
                        </article>
                    </div>
                {:else}
                    <p class="text-foreground-muted m-0 py-4 text-center text-sm">
                        No effects in this queue.
                    </p>
                {/each}

                <section class="border-border grid gap-2 rounded-xl border border-dashed p-3">
                    <span class="text-foreground-muted text-xs font-semibold uppercase">
                        Add effect
                    </span>
                    <div
                        class="grid grid-cols-2 gap-2 min-[37.5rem]:grid-cols-3 min-[56.25rem]:grid-cols-5">
                        {#each addableEffects as addable}
                            <Button
                                variant="secondary"
                                size="lg"
                                class="min-w-0 px-2"
                                type="button"
                                title={atEffectCapacity
                                    ? 'Effect queue is full'
                                    : `Add ${addable.label.toLowerCase()}`}
                                disabled={atEffectCapacity}
                                onclick={() => addEffect(addable.kind)}>
                                <Icon icon="lucide:plus" width="1rem" />
                                <span class="truncate">{addable.label}</span>
                            </Button>
                        {/each}
                    </div>
                </section>
            </div>
        </div>
    </div>

    {#if draggedEffect}
        <div
            class="border-accent/50 bg-surface-overlay pointer-events-none fixed z-90 flex w-[min(20rem,calc(100vw-2rem))] -translate-x-4 -translate-y-4 items-center gap-2 rounded-xl border p-3 shadow-[0_1rem_2.5rem_var(--overlay-shadow)]"
            style:left={`${dragX}px`}
            style:top={`${dragY}px`}>
            <Icon class="text-foreground-muted" icon="lucide:grip-vertical" width="1rem" />
            <strong class="min-w-0 flex-1 truncate text-sm">{effectLabel(draggedEffect)}</strong>
            {#if dropInsertionIndex !== null}<small
                    class="bg-accent/12 text-accent rounded-md px-2 py-1 font-mono text-xs">
                    #{Math.min(dropInsertionIndex + 1, draft.length)}
                </small>{/if}
        </div>
    {/if}
</dialog>
