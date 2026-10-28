<script lang="ts">
    import SaveIndicator from '$lib/components/SaveIndicator.svelte';

    let {
        id,
        value = $bindable(),
        pending = false,
        saving = false,
        disabled = false,
        required = false,
        oncommit
    }: {
        id: string;
        value?: string;
        pending?: boolean;
        saving?: boolean;
        disabled?: boolean;
        required?: boolean;
        oncommit: () => unknown | Promise<unknown>;
    } = $props();

    function commit() {
        if (pending && !saving && !disabled) {
            void oncommit();
        }
    }
</script>

<div class="flex gap-2">
    <input
        {id}
        class="border-border-strong bg-surface-inset text-foreground h-10 min-w-0 flex-1 rounded-lg border px-2.5 text-sm"
        bind:value
        {disabled}
        {required}
        onblur={commit}
        onkeydown={(event) => {
            if (event.key === 'Enter') {
                event.preventDefault();
                event.currentTarget.blur();
            }
        }} />
    <SaveIndicator {saving} {pending} />
</div>
