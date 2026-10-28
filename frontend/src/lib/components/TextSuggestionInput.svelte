<script module lang="ts">
    export type TextSuggestion = {
        value: string;
        label: string;
        description?: string;
    };
</script>

<script lang="ts">
    import Icon from '@iconify/svelte';

    let {
        id,
        suggestions,
        value = $bindable(''),
        placeholder,
        disabled = false,
        clearable = false,
        noMatchesText = 'No matching options',
        ariaLabel
    }: {
        id: string;
        suggestions: TextSuggestion[];
        value?: string;
        placeholder?: string;
        disabled?: boolean;
        clearable?: boolean;
        noMatchesText?: string;
        ariaLabel?: string;
    } = $props();

    let open = $state(false);
    let activeIndex = $state(-1);
    let root: HTMLDivElement;
    let input: HTMLInputElement;
    let pointerStartX = 0;
    let pointerStartY = 0;
    let touchMoved = false;

    const TOUCH_SCROLL_THRESHOLD = 8;

    let matches = $derived(
        suggestions.filter((suggestion) => {
            const query = value.trim().toLocaleLowerCase();
            return (
                !query ||
                suggestion.value.toLocaleLowerCase().includes(query) ||
                suggestion.label.toLocaleLowerCase().includes(query) ||
                suggestion.description?.toLocaleLowerCase().includes(query)
            );
        })
    );

    function select(suggestion: TextSuggestion) {
        value = suggestion.value;
        open = false;
        activeIndex = -1;
        input.focus();
    }

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'ArrowDown') {
            event.preventDefault();
            open = true;
            activeIndex = Math.min(activeIndex + 1, matches.length - 1);
        } else if (event.key === 'ArrowUp') {
            event.preventDefault();
            activeIndex = Math.max(activeIndex - 1, 0);
        } else if (event.key === 'Enter' && open && activeIndex >= 0) {
            event.preventDefault();
            select(matches[activeIndex]);
        } else if (event.key === 'Escape') {
            open = false;
            activeIndex = -1;
        }
    }

    function handleFocusOut(event: FocusEvent) {
        const next = event.relatedTarget;
        if (!(next instanceof Node) || !root.contains(next)) {
            open = false;
            activeIndex = -1;
        }
    }

    function handlePointerDown(event: PointerEvent) {
        if (event.pointerType !== 'touch') {
            return;
        }
        pointerStartX = event.clientX;
        pointerStartY = event.clientY;
        touchMoved = false;
    }

    function handlePointerMove(event: PointerEvent) {
        if (
            event.pointerType === 'touch' &&
            (Math.abs(event.clientX - pointerStartX) >= TOUCH_SCROLL_THRESHOLD ||
                Math.abs(event.clientY - pointerStartY) >= TOUCH_SCROLL_THRESHOLD)
        ) {
            touchMoved = true;
        }
    }

    function selectFromPointer(event: PointerEvent, suggestion: TextSuggestion) {
        if (event.pointerType === 'touch' && !touchMoved) {
            select(suggestion);
        }
    }
</script>

<div bind:this={root} class="relative min-w-0" onfocusout={handleFocusOut}>
    <input
        bind:this={input}
        {id}
        class={`border-border-strong bg-surface-inset text-foreground h-10 w-full min-w-0 rounded-lg border pl-3 text-sm ${clearable ? 'pr-9' : 'pr-3'}`}
        type="text"
        role="combobox"
        aria-label={ariaLabel}
        aria-autocomplete="list"
        aria-controls={`${id}-suggestions`}
        aria-expanded={open}
        {placeholder}
        {disabled}
        autocomplete="off"
        bind:value
        onfocus={() => (open = true)}
        oninput={() => {
            open = true;
            activeIndex = -1;
        }}
        onkeydown={handleKeydown} />

    {#if clearable && value && !disabled}
        <button
            class="text-foreground-muted hover:bg-surface-hover hover:text-foreground absolute top-1/2 right-1 grid size-7 -translate-y-1/2 cursor-pointer place-items-center rounded-md border-0 bg-transparent transition-colors"
            type="button"
            aria-label="Clear input"
            title="Clear"
            onclick={() => {
                value = '';
                open = true;
                input.focus();
            }}>
            <Icon icon="lucide:x" width="1rem" />
        </button>
    {/if}

    {#if open && !disabled}
        <div
            id={`${id}-suggestions`}
            class="border-border-strong bg-surface-card absolute top-[calc(100%+0.25rem)] right-0 left-0 z-50 max-h-56 touch-pan-y overflow-y-auto overscroll-contain rounded-lg border p-1 shadow-lg"
            role="listbox"
            tabindex="-1"
            aria-label={ariaLabel}
            onpointerdown={handlePointerDown}
            onpointermove={handlePointerMove}>
            {#each matches as suggestion, index (suggestion.value)}
                <button
                    class={`text-foreground hover:bg-surface-hover block w-full rounded-md border-0 px-2.5 py-2 text-left transition-colors ${index === activeIndex ? 'bg-surface-hover' : 'bg-transparent'}`}
                    type="button"
                    role="option"
                    aria-selected={suggestion.value === value}
                    onpointerenter={(event) => {
                        if (event.pointerType === 'mouse') {
                            activeIndex = index;
                        }
                    }}
                    onpointerup={(event) => selectFromPointer(event, suggestion)}
                    onclick={() => {
                        if (!touchMoved) {
                            select(suggestion);
                        }
                    }}>
                    <span class="block truncate text-sm font-semibold">{suggestion.label}</span>
                    {#if suggestion.description}
                        <span class="text-foreground-muted mt-0.5 block truncate text-xs">
                            {suggestion.description}
                        </span>
                    {/if}
                </button>
            {:else}
                <span class="text-foreground-muted block px-2.5 py-2 text-sm">
                    {noMatchesText}
                </span>
            {/each}
        </div>
    {/if}
</div>
