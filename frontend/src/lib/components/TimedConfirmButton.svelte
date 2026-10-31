<script lang="ts">
    import { onDestroy } from 'svelte';
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';

    let {
        label,
        confirmLabel,
        busyLabel,
        timeout = 4000,
        busy = false,
        disabled = false,
        class: className = '',
        onconfirm
    }: {
        label: string;
        confirmLabel: string;
        busyLabel: string;
        timeout?: number;
        busy?: boolean;
        disabled?: boolean;
        class?: string;
        onconfirm: () => unknown | Promise<unknown>;
    } = $props();

    let armed = $state(false);
    let resetTimer: ReturnType<typeof setTimeout> | undefined;

    function reset() {
        clearTimeout(resetTimer);
        resetTimer = undefined;
        armed = false;
    }

    async function activate() {
        if (armed) {
            clearTimeout(resetTimer);
            resetTimer = undefined;
            await onconfirm();
            armed = false;
            return;
        }

        armed = true;
        resetTimer = setTimeout(reset, timeout);
    }

    onDestroy(() => clearTimeout(resetTimer));
</script>

<Button
    variant={armed ? 'danger' : 'danger-ghost'}
    size="sm"
    class={className}
    {busy}
    disabled={disabled || busy}
    onclick={activate}>
    {#if busy}
        <Icon icon="lucide:loader-circle" width="0.875rem" class="animate-spin" />
    {/if}
    {busy ? busyLabel : armed ? confirmLabel : label}
</Button>
