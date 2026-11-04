<script lang="ts">
    import ApplyButton from '$lib/components/ApplyButton.svelte';

    interface Props {
        message: string;
        saving: boolean;
        pending: boolean;
        applyLabel: string;
    }

    let { message, saving, pending, applyLabel }: Props = $props();
</script>

<div class="flex items-center justify-between gap-3">
    <small class="text-foreground-muted">{message}</small>
    <ApplyButton {saving} {pending} label={applyLabel} />
</div>
