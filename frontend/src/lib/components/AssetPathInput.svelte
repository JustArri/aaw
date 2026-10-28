<script lang="ts">
    import Icon from '@iconify/svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import { notify } from '$lib/push-notifications.svelte';

    type Props = {
        id: string;
        assets: string[];
        value?: string;
        placeholder?: string;
        disabled?: boolean;
        required?: boolean;
        compact?: boolean;
        onreload?: () => unknown | Promise<unknown>;
    };

    let {
        id,
        assets,
        value = $bindable(''),
        placeholder,
        disabled = false,
        required = false,
        compact = false,
        onreload
    }: Props = $props();

    let reloading = $state(false);
    let reloadDone = $state(false);

    function fileName(path: string) {
        return path.split('/').at(-1) ?? path;
    }

    function directory(path: string) {
        const separator = path.lastIndexOf('/');
        return separator < 0 ? '' : path.slice(0, separator);
    }

    let options = $derived(
        [value && !assets.includes(value) ? value : '', ...assets].filter(Boolean).map((asset) => ({
            value: asset,
            label: fileName(asset),
            selectionLabel: asset,
            description: directory(asset) || undefined
        }))
    );

    async function reload() {
        if (reloading) {
            return;
        }
        reloading = true;
        reloadDone = false;
        try {
            await onreload?.();
            reloadDone = true;
            setTimeout(() => {
                reloadDone = false;
            }, 1200);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : 'Could not reload assets.', {
                title: 'Reload assets'
            });
        } finally {
            reloading = false;
        }
    }
</script>

<div class="flex min-w-0 gap-2">
    <div class="relative min-w-0 flex-1">
        <SelectDropdown
            {id}
            {options}
            {placeholder}
            {disabled}
            {compact}
            searchable
            searchPlaceholder="Search assets"
            noMatchesText="No matching assets"
            ariaLabel="Asset path"
            bind:value />
        {#if value && !disabled}
            <button
                class={`text-foreground-muted hover:bg-surface-hover hover:text-foreground absolute top-1/2 z-10 grid -translate-y-1/2 cursor-pointer place-items-center rounded-md border-0 bg-transparent transition-colors ${compact ? 'right-7 size-7' : 'right-8 size-8'}`}
                type="button"
                aria-label="Clear asset path"
                title="Clear asset path"
                onclick={() => (value = '')}>
                <Icon icon="lucide:x" width="1rem" />
            </button>
        {/if}
        {#if required}
            <input
                class="pointer-events-none absolute size-px opacity-0"
                type="text"
                tabindex="-1"
                aria-hidden="true"
                {value}
                required />
        {/if}
    </div>

    <button
        class={`border-border-strong bg-surface-button text-foreground-muted hover:border-accent/40 hover:bg-accent/10 hover:text-accent grid shrink-0 cursor-pointer place-items-center rounded-lg border transition-colors disabled:cursor-wait disabled:opacity-60 ${compact ? 'size-9' : 'size-10'}`}
        type="button"
        aria-label="Reload assets"
        title="Reload assets"
        disabled={reloading}
        onclick={reload}>
        <Icon
            icon={reloading
                ? 'lucide:loader-circle'
                : reloadDone
                  ? 'lucide:check'
                  : 'lucide:refresh-cw'}
            width="1.125rem"
            class={reloading ? 'animate-spin' : reloadDone ? 'text-success' : ''} />
    </button>
</div>
