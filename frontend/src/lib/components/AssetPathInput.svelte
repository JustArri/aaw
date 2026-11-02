<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
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
            clearable
            searchable
            searchPlaceholder="Search assets"
            noMatchesText="No matching assets"
            ariaLabel="Asset path"
            bind:value />
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

    <Button
        variant="secondary"
        size={compact ? 'md' : 'lg'}
        square
        class="text-foreground-muted enabled:hover:border-accent/40 enabled:hover:bg-accent/10 enabled:hover:text-accent"
        type="button"
        aria-label="Reload assets"
        title="Reload assets"
        busy={reloading}
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
    </Button>
</div>
