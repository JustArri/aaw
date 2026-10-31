<script lang="ts">
    import Icon from '@iconify/svelte';
    import { tick } from 'svelte';
    import Button from '$lib/components/Button.svelte';

    export type SelectOption = {
        value: string;
        label: string;
        selectionLabel?: string;
        description?: string;
        group?: string;
        disabled?: boolean;
    };

    type Props = {
        id?: string;
        options: SelectOption[];
        value?: string;
        placeholder?: string;
        disabled?: boolean;
        busy?: boolean;
        compact?: boolean;
        clearable?: boolean;
        searchable?: boolean;
        searchPlaceholder?: string;
        noMatchesText?: string;
        ariaLabel?: string;
        class?: string;
        onchange?: (value: string) => void;
    };

    let {
        id,
        options,
        value = $bindable(),
        placeholder = 'Select an option',
        disabled = false,
        busy = false,
        compact = false,
        clearable = false,
        searchable = false,
        searchPlaceholder = 'Search options',
        noMatchesText = 'No matching options',
        ariaLabel,
        class: className = '',
        onchange
    }: Props = $props();

    let open = $state(false);
    let activeIndex = $state(-1);
    let root: HTMLDivElement;
    let panel = $state<HTMLDivElement>();
    let menu = $state<HTMLDivElement>();
    let searchInput = $state<HTMLInputElement>();
    let searchQuery = $state('');
    let panelStyle = $state('');
    let pointerStartX = 0;
    let pointerStartY = 0;
    let suppressSelection = false;
    let suppressClick = false;
    let selected = $derived(options.find((option) => option.value === value));
    let selectedText = $derived(selected?.selectionLabel ?? selected?.label ?? placeholder);
    let visibleOptions = $derived(
        options
            .map((option, index) => ({ option, index }))
            .filter(({ option }) => {
                const query = searchQuery.trim().toLocaleLowerCase();
                return (
                    !query ||
                    option.label.toLocaleLowerCase().includes(query) ||
                    option.selectionLabel?.toLocaleLowerCase().includes(query) ||
                    option.description?.toLocaleLowerCase().includes(query) ||
                    option.group?.toLocaleLowerCase().includes(query)
                );
            })
    );

    const TOUCH_SCROLL_THRESHOLD = 8;

    function firstEnabledOption() {
        return options.findIndex((option) => !option.disabled);
    }

    function selectedOrFirstEnabledOption() {
        const selectedIndex = options.findIndex(
            (option) => option.value === value && !option.disabled
        );
        return selectedIndex >= 0 ? selectedIndex : firstEnabledOption();
    }

    function openMenu() {
        if (disabled || busy) {
            return;
        }
        open = true;
        positionPanel();
        searchQuery = '';
        activeIndex = selectedOrFirstEnabledOption();
        if (searchable) {
            void focusSearch();
        }
        void revealActiveOption(true);
    }

    function closeMenu() {
        open = false;
        activeIndex = -1;
    }

    function portal(node: HTMLElement) {
        const destination = root.closest('dialog[open]') ?? document.body;
        destination.appendChild(node);
        return {
            destroy() {
                node.remove();
            }
        };
    }

    function positionPanel() {
        if (window.innerWidth < 600) {
            panelStyle = '';
            return;
        }

        const rect = root.getBoundingClientRect();
        const spaceBelow = window.innerHeight - rect.bottom;
        const openAbove = spaceBelow < 280 && rect.top > spaceBelow;
        panelStyle = [
            `left: ${rect.left}px`,
            `width: ${rect.width}px`,
            openAbove
                ? `bottom: ${window.innerHeight - rect.top + 4}px`
                : `top: ${rect.bottom + 4}px`
        ].join('; ');
    }

    function select(option: SelectOption) {
        if (option.disabled) {
            return;
        }
        const changed = value !== option.value;
        value = option.value;
        closeMenu();
        if (changed) {
            onchange?.(value);
        }
    }

    function clearSelection() {
        if (!value) {
            return;
        }
        value = '';
        closeMenu();
        onchange?.('');
    }

    function moveActive(direction: -1 | 1) {
        const enabled = visibleOptions.filter(({ option }) => !option.disabled);
        if (enabled.length === 0) {
            return;
        }
        const position = enabled.findIndex(({ index }) => index === activeIndex);
        const nextPosition =
            position < 0
                ? direction > 0
                    ? 0
                    : enabled.length - 1
                : (position + direction + enabled.length) % enabled.length;
        activeIndex = enabled[nextPosition].index;
        void revealActiveOption(false);
    }

    async function focusSearch() {
        await tick();
        searchInput?.focus();
    }

    function handleSearchInput() {
        const selectedVisible = visibleOptions.some(({ index }) => index === activeIndex);
        if (!selectedVisible) {
            activeIndex = visibleOptions.find(({ option }) => !option.disabled)?.index ?? -1;
        }
    }

    function handleSearchKeydown(event: KeyboardEvent) {
        if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
            event.preventDefault();
            moveActive(event.key === 'ArrowDown' ? 1 : -1);
        } else if (event.key === 'Enter') {
            event.preventDefault();
            const option = options[activeIndex];
            if (option) {
                select(option);
            }
        } else if (event.key === 'Escape') {
            event.preventDefault();
            closeMenu();
        }
    }

    async function revealActiveOption(center: boolean) {
        await tick();
        const menuElement = menu;
        const option = menuElement?.querySelector<HTMLElement>(
            `[data-option-index="${activeIndex}"]`
        );
        if (!menuElement || !option) {
            return;
        }

        if (center) {
            menuElement.scrollTop = Math.max(
                0,
                option.offsetTop - (menuElement.clientHeight - option.offsetHeight) / 2
            );
        } else {
            option.scrollIntoView({ block: 'nearest' });
        }
    }

    function handleKeydown(event: KeyboardEvent) {
        if (disabled || busy) {
            return;
        }
        if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
            event.preventDefault();
            if (!open) {
                openMenu();
            } else {
                moveActive(event.key === 'ArrowDown' ? 1 : -1);
            }
        } else if ((event.key === 'Enter' || event.key === ' ') && open) {
            event.preventDefault();
            const option = options[activeIndex];
            if (option) {
                select(option);
            }
        } else if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            openMenu();
        } else if (event.key === 'Escape') {
            event.preventDefault();
            closeMenu();
        }
    }

    function handleFocusOut(event: FocusEvent) {
        const next = event.relatedTarget;
        if (!(next instanceof Node) || (!root.contains(next) && !panel?.contains(next))) {
            closeMenu();
        }
    }

    function handleWindowPointerDown(event: PointerEvent) {
        const target = event.target;
        if (open && target instanceof Node && !root.contains(target) && !panel?.contains(target)) {
            closeMenu();
        }
    }

    function handleMenuPointerDown(event: PointerEvent) {
        suppressSelection = false;
        suppressClick = false;
        if (event.pointerType !== 'touch') {
            return;
        }
        pointerStartX = event.clientX;
        pointerStartY = event.clientY;
    }

    function handleMenuPointerMove(event: PointerEvent) {
        if (event.pointerType !== 'touch') {
            return;
        }
        if (
            Math.abs(event.clientX - pointerStartX) >= TOUCH_SCROLL_THRESHOLD ||
            Math.abs(event.clientY - pointerStartY) >= TOUCH_SCROLL_THRESHOLD
        ) {
            suppressSelection = true;
        }
    }

    function handleMenuPointerCancel(event: PointerEvent) {
        if (event.pointerType === 'touch') {
            suppressSelection = true;
            suppressClick = true;
        }
    }

    function finishTouch(event: PointerEvent) {
        if (event.pointerType !== 'touch') {
            return;
        }
        suppressClick = suppressSelection;
    }

    function selectFromMenu(event: MouseEvent, option: SelectOption) {
        if (suppressClick || suppressSelection) {
            event.preventDefault();
            suppressClick = false;
            return;
        }
        select(option);
    }
</script>

<svelte:window
    onpointerdown={handleWindowPointerDown}
    onresize={() => {
        if (open) {
            positionPanel();
        }
    }} />

<div bind:this={root} class={`relative min-w-0 ${className}`} onfocusout={handleFocusOut}>
    <button
        {id}
        class={`border-border-strong bg-surface-inset text-foreground flex w-full min-w-0 items-center justify-between gap-2 border text-left transition-colors ${disabled ? 'cursor-not-allowed opacity-50' : busy ? 'cursor-wait' : 'hover:border-accent/40 cursor-pointer'} ${compact ? `h-9 rounded-md pl-2 text-xs ${clearable && value ? 'pr-8' : 'pr-2'}` : `h-10 rounded-lg pl-2.5 text-sm ${clearable && value ? 'pr-9' : 'pr-2.5'}`} ${open ? 'border-accent/50' : ''}`}
        type="button"
        role="combobox"
        aria-label={ariaLabel}
        aria-controls={id ? `${id}-options` : undefined}
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-activedescendant={open && activeIndex >= 0 && id
            ? `${id}-option-${activeIndex}`
            : undefined}
        disabled={disabled || busy}
        aria-busy={busy}
        title={selected ? selectedText : undefined}
        onclick={() => (open ? closeMenu() : openMenu())}
        onkeydown={handleKeydown}>
        <span class={`min-w-0 flex-1 truncate ${selected ? '' : 'text-foreground-muted'}`}>
            {selectedText}
        </span>
        <Icon
            class={`text-foreground-muted shrink-0 transition-transform ${open ? 'rotate-180' : ''}`}
            icon="lucide:chevron-down"
            width={compact ? '0.875rem' : '1rem'} />
    </button>

    {#if clearable && value && !disabled && !busy}
        <Button
            variant="ghost"
            size={compact ? 'xs' : 'sm'}
            square
            class="absolute top-1/2 right-1 z-10 -translate-y-1/2"
            type="button"
            aria-label="Clear selection"
            title="Clear"
            onclick={clearSelection}>
            <Icon icon="lucide:x" width="1rem" />
        </Button>
    {/if}

    {#if open}
        <button
            use:portal
            class="fixed inset-0 z-[99] cursor-default border-0 bg-black/35 min-[37.5rem]:hidden"
            type="button"
            aria-label="Close options"
            onclick={closeMenu}>
        </button>
        <div
            use:portal
            bind:this={panel}
            id={id ? `${id}-options` : undefined}
            style={panelStyle}
            class="border-border-strong bg-surface-card fixed inset-3 z-[100] flex min-h-0 flex-col overflow-hidden rounded-xl border shadow-lg min-[37.5rem]:inset-auto min-[37.5rem]:max-h-none min-[37.5rem]:rounded-lg">
            <div
                class="border-border flex min-h-12 items-start justify-between gap-3 border-b px-3 py-2 min-[37.5rem]:hidden">
                <div class="min-w-0 flex-1">
                    <span class="text-foreground-strong block text-sm font-semibold">
                        {ariaLabel ?? placeholder}
                    </span>
                    {#if selected}
                        <span
                            class="text-foreground-muted mt-0.5 block text-xs break-all"
                            title={selectedText}>
                            {selectedText}
                        </span>
                    {/if}
                </div>
                <Button
                    variant="ghost"
                    size="md"
                    square
                    type="button"
                    aria-label="Close options"
                    onclick={closeMenu}>
                    <Icon icon="lucide:x" width="1.125rem" />
                </Button>
            </div>
            {#if searchable}
                <div class="border-border border-b p-1.5">
                    <div class="relative">
                        <Icon
                            class="text-foreground-muted pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2"
                            icon="lucide:search"
                            width="0.875rem" />
                        <input
                            bind:this={searchInput}
                            bind:value={searchQuery}
                            class="border-border-strong bg-surface-inset text-foreground placeholder:text-foreground-muted focus:border-accent/50 h-9 w-full rounded-md border pr-2.5 pl-8 text-sm outline-none"
                            type="search"
                            placeholder={searchPlaceholder}
                            aria-label={searchPlaceholder}
                            oninput={handleSearchInput}
                            onkeydown={handleSearchKeydown} />
                    </div>
                </div>
            {/if}
            <div
                bind:this={menu}
                class="min-h-0 flex-1 touch-pan-y overflow-y-auto overscroll-contain p-1 min-[37.5rem]:max-h-56"
                role="listbox"
                tabindex="-1"
                aria-label={ariaLabel}
                onpointerdown={handleMenuPointerDown}
                onpointermove={handleMenuPointerMove}
                onpointercancel={handleMenuPointerCancel}>
                {#each visibleOptions as { option, index } (option.value)}
                    {#if option.group && (index === 0 || options[index - 1]?.group !== option.group)}
                        <span
                            class="text-foreground-muted block px-2.5 pt-2 pb-1 text-xs font-semibold uppercase">
                            {option.group}
                        </span>
                    {/if}
                    <button
                        id={id ? `${id}-option-${index}` : undefined}
                        data-option-index={index}
                        class={`text-foreground hover:bg-surface-hover block w-full cursor-pointer rounded-md border-0 px-2.5 py-2 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-40 ${index === activeIndex ? 'bg-surface-hover' : 'bg-transparent'} ${option.value === value ? 'text-accent' : ''}`}
                        type="button"
                        role="option"
                        aria-selected={option.value === value}
                        title={[option.label, option.description].filter(Boolean).join(' · ')}
                        disabled={option.disabled}
                        onpointerenter={(event) => {
                            if (event.pointerType === 'mouse' && !option.disabled) {
                                activeIndex = index;
                            }
                        }}
                        onpointerup={finishTouch}
                        onclick={(event) => selectFromMenu(event, option)}>
                        <span class="block text-sm font-semibold break-all min-[37.5rem]:truncate">
                            {option.label}
                        </span>
                        {#if option.description}
                            <span
                                class="text-foreground-muted mt-0.5 block text-xs break-all min-[37.5rem]:truncate">
                                {option.description}
                            </span>
                        {/if}
                    </button>
                {:else}
                    <span class="text-foreground-muted block px-2.5 py-2 text-sm">
                        {searchQuery ? noMatchesText : 'No options available'}
                    </span>
                {/each}
            </div>
        </div>
    {/if}
</div>
