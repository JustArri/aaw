<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';

    interface Props {
        name: string;
        index: number;
        learning: boolean;
        saving: boolean;
        removeDisabled: boolean;
        learningIcon?: string;
        onnamechange: (value: string) => void;
        ontogglelearning: () => void;
        onremove: () => void | Promise<void>;
    }

    let {
        name,
        index,
        learning,
        saving,
        removeDisabled,
        learningIcon = 'lucide:radio-tower',
        onnamechange,
        ontogglelearning,
        onremove
    }: Props = $props();
</script>

<div class="flex flex-wrap items-end gap-2">
    <label class="grid min-w-48 flex-1 gap-1">
        <span class="text-foreground-muted text-xs">Name</span>
        <input
            class="border-border-strong bg-surface-card text-foreground h-10 rounded-lg border px-2.5 text-sm"
            value={name}
            placeholder={`Mapping ${index + 1}`}
            oninput={(event) => onnamechange(event.currentTarget.value)} />
    </label>
    <div class="flex items-center gap-1.5">
        <Button
            variant={learning ? 'danger' : 'secondary'}
            size="sm"
            aria-pressed={learning}
            onclick={ontogglelearning}>
            <Icon
                icon={learning ? learningIcon : 'lucide:circle-dot'}
                width="0.875rem"
                class={learning ? 'animate-pulse' : ''} />
            {learning ? 'Listening…' : 'MIDI Learn'}
        </Button>
        <TimedConfirmButton
            label="Remove"
            confirmLabel="Confirm remove"
            busyLabel="Removing"
            busy={saving}
            disabled={removeDisabled}
            onconfirm={onremove} />
    </div>
</div>
