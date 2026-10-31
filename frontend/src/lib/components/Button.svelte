<script lang="ts">
    import type { Snippet } from 'svelte';
    import type { HTMLButtonAttributes } from 'svelte/elements';

    type Variant =
        | 'primary'
        | 'secondary'
        | 'ghost'
        | 'accent'
        | 'accent-outline'
        | 'danger'
        | 'danger-soft'
        | 'danger-ghost'
        | 'danger-outline';
    type Size = 'xs' | 'sm' | 'md' | 'lg';

    type Props = Omit<HTMLButtonAttributes, 'class' | 'children'> & {
        children: Snippet;
        variant?: Variant;
        size?: Size;
        square?: boolean;
        busy?: boolean;
        class?: string;
    };

    let {
        children,
        variant = 'secondary',
        size = 'md',
        square = false,
        busy = false,
        class: className = '',
        type = 'button',
        ...attributes
    }: Props = $props();

    const variants: Record<Variant, string> = {
        primary: 'border-transparent bg-accent text-accent-foreground enabled:hover:bg-accent/85',
        secondary:
            'border-border-strong bg-surface-button text-foreground enabled:hover:bg-surface-hover',
        ghost: 'border-transparent bg-transparent text-foreground-muted enabled:hover:bg-surface-hover enabled:hover:text-foreground',
        accent: 'border-transparent bg-transparent text-accent enabled:hover:bg-accent/10',
        'accent-outline': 'border-accent/40 bg-accent/10 text-accent enabled:hover:bg-accent/15',
        danger: 'border-transparent bg-danger text-white enabled:hover:bg-danger/85',
        'danger-soft': 'border-danger/40 bg-danger/8 text-danger enabled:hover:bg-danger/15',
        'danger-ghost': 'border-transparent bg-transparent text-danger enabled:hover:bg-danger/10',
        'danger-outline':
            'border-danger/40 bg-transparent text-danger enabled:hover:border-danger/60 enabled:hover:bg-danger/10'
    };

    const regularSizes: Record<Size, string> = {
        xs: 'min-h-7 rounded-md px-2.5 text-xs',
        sm: 'min-h-9 rounded-lg px-3 text-sm',
        md: 'min-h-10 rounded-lg px-3.5 text-sm',
        lg: 'min-h-11 rounded-lg px-4 text-sm'
    };

    const squareSizes: Record<Size, string> = {
        xs: 'size-7 rounded-md',
        sm: 'size-8 rounded-lg',
        md: 'size-9 rounded-lg',
        lg: 'size-10 rounded-lg'
    };
</script>

<button
    {...attributes}
    {type}
    aria-busy={busy || attributes['aria-busy']}
    class={`inline-flex shrink-0 cursor-pointer items-center justify-center gap-2 border font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${busy ? 'disabled:cursor-wait' : ''} ${variants[variant]} ${square ? squareSizes[size] : regularSizes[size]} ${className}`}>
    {@render children()}
</button>
