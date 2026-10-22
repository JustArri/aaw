<script lang="ts">
    import Icon from '@iconify/svelte';
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

    let open = $state(false);
    let activeIndex = $state(-1);
    let reloading = $state(false);
    let reloadDone = $state(false);
    let inputElement: HTMLInputElement;
    let suggestions = $derived(
        assets
            .filter((asset) => asset.toLocaleLowerCase().includes(value.toLocaleLowerCase()))
            .slice(0, 12)
    );

    function fileName(path: string) {
        return path.split('/').at(-1) ?? path;
    }

    function directory(path: string) {
        const separator = path.lastIndexOf('/');
        return separator < 0 ? '' : path.slice(0, separator);
    }

    function select(asset: string) {
        value = asset;
        open = false;
        activeIndex = -1;
    }

    function clear() {
        value = '';
        open = true;
        activeIndex = -1;
        inputElement.focus();
    }

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'ArrowDown') {
            event.preventDefault();
            open = true;
            activeIndex = Math.min(activeIndex + 1, suggestions.length - 1);
        } else if (event.key === 'ArrowUp') {
            event.preventDefault();
            activeIndex = Math.max(activeIndex - 1, 0);
        } else if (event.key === 'Enter' && open && activeIndex >= 0) {
            event.preventDefault();
            select(suggestions[activeIndex]);
        } else if (event.key === 'Escape') {
            open = false;
            activeIndex = -1;
        }
    }

    async function reload() {
        if (reloading) {
            return;
        }
        reloading = true;
        reloadDone = false;
        open = false;
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
        <input
            bind:this={inputElement}
            {id}
            class={`border-border-strong bg-surface-inset text-foreground w-full min-w-0 rounded-lg border pr-9 pl-2.5 font-mono ${compact ? 'h-9 text-xs' : 'h-10'}`}
            type="text"
            role="combobox"
            aria-autocomplete="list"
            aria-controls={`${id}-suggestions`}
            aria-expanded={open && suggestions.length > 0}
            aria-activedescendant={activeIndex >= 0 ? `${id}-suggestion-${activeIndex}` : undefined}
            {placeholder}
            {disabled}
            {required}
            bind:value
            onfocus={() => (open = true)}
            onblur={() => (open = false)}
            oninput={() => {
                open = true;
                activeIndex = -1;
            }}
            onkeydown={handleKeydown} />

        {#if value && !disabled}
            <button
                class="text-foreground-muted hover:bg-surface-hover hover:text-foreground absolute top-1/2 right-1 grid size-7 -translate-y-1/2 cursor-pointer place-items-center rounded-md border-0 bg-transparent transition-colors"
                type="button"
                aria-label="Clear asset path"
                title="Clear asset path"
                onpointerdown={(event) => event.preventDefault()}
                onclick={clear}>
                <Icon icon="lucide:x" width="1rem" />
            </button>
        {/if}

        {#if open && !disabled && suggestions.length > 0}
            <div
                id={`${id}-suggestions`}
                class="border-border-strong bg-surface-card absolute top-[calc(100%+0.25rem)] right-0 left-0 z-20 max-h-56 overflow-y-auto rounded-lg border p-1 shadow-lg"
                role="listbox">
                {#each suggestions as asset, index}
                    <button
                        id={`${id}-suggestion-${index}`}
                        class={`text-foreground hover:bg-surface-hover block w-full cursor-pointer rounded-md border-0 px-2.5 py-2 text-left ${index === activeIndex ? 'bg-surface-hover' : 'bg-transparent'}`}
                        type="button"
                        role="option"
                        aria-selected={index === activeIndex}
                        onpointerdown={(event) => {
                            event.preventDefault();
                            select(asset);
                        }}>
                        <span class="block truncate text-sm font-semibold">{fileName(asset)}</span>
                        {#if directory(asset)}
                            <span class="text-foreground-muted mt-0.5 block truncate text-xs">
                                {directory(asset)}
                            </span>
                        {/if}
                    </button>
                {/each}
            </div>
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
