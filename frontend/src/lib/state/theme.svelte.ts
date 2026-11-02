import { onMount } from 'svelte';

export type ThemeMode = 'light' | 'dark' | 'auto';
export type Accent = { r: number; g: number; b: number };

let accentState = $state({ r: 96, g: 199, b: 255 });
let themeModeState = $state<ThemeMode>('auto');

export function accent() {
    return accentState;
}

export function themeMode() {
    return themeModeState;
}

export function accentHex() {
    return `#${[accentState.r, accentState.g, accentState.b]
        .map((value) => Math.max(0, Math.min(255, value)).toString(16).padStart(2, '0'))
        .join('')}`;
}

export function setAccent(next: Accent) {
    accentState = next;
    document.documentElement.style.setProperty('--accent-rgb', `${next.r} ${next.g} ${next.b}`);
    localStorage.setItem('aaw-accent', JSON.stringify(next));
}

export function setAccentHex(hex: string) {
    const value = hex.replace('#', '');

    if (!/^[0-9a-f]{6}$/i.test(value)) {
        return;
    }

    setAccent({
        r: Number.parseInt(value.slice(0, 2), 16),
        g: Number.parseInt(value.slice(2, 4), 16),
        b: Number.parseInt(value.slice(4, 6), 16)
    });
}

export function applyTheme(
    mode: ThemeMode,
    systemPrefersDark = matchMedia('(prefers-color-scheme: dark)').matches
) {
    themeModeState = mode;
    document.documentElement.classList.toggle(
        'dark',
        mode === 'dark' || (mode === 'auto' && systemPrefersDark)
    );
    localStorage.setItem('aaw-theme', mode);
}

export function autoloadTheme() {
    onMount(() => {
        const storedTheme = localStorage.getItem('aaw-theme');
        const initialTheme: ThemeMode =
            storedTheme === 'light' || storedTheme === 'dark' || storedTheme === 'auto'
                ? storedTheme
                : 'auto';
        const colorScheme = matchMedia('(prefers-color-scheme: dark)');
        const syncAutomaticTheme = (event: MediaQueryListEvent) => {
            if (themeModeState === 'auto') applyTheme('auto', event.matches);
        };

        applyTheme(initialTheme, colorScheme.matches);
        colorScheme.addEventListener('change', syncAutomaticTheme);

        const storedAccent = localStorage.getItem('aaw-accent');
        if (storedAccent) {
            try {
                const next = JSON.parse(storedAccent) as { r: number; g: number; b: number };
                if (
                    [next.r, next.g, next.b].every(
                        (value) => Number.isInteger(value) && value >= 0 && value <= 255
                    )
                ) {
                    setAccent(next);
                }
            } catch {
                // Ignore malformed local preferences.
            }
        }

        return () => colorScheme.removeEventListener('change', syncAutomaticTheme);
    });
}
