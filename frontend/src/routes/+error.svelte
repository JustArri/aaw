<script lang="ts">
    import { page } from '$app/state';
    import '../app.css';
</script>

<svelte:head>
    <title>{page.status} · AAW Control</title>
</svelte:head>

<main class="bg-app-background text-foreground grid min-h-svh place-items-center p-4">
    <section
        class="border-border-strong bg-surface-card w-full max-w-md overflow-hidden rounded-xl border shadow-2xl">
        <header class="border-border flex items-center gap-3 border-b px-5 py-4">
            <div
                class="border-accent/35 bg-accent/9 text-accent grid size-9 place-items-center rounded-lg border">
                <span class="text-xs font-bold tracking-[0.12em]">AAW</span>
            </div>
            <div>
                <strong class="block text-sm">Arri's Audio Workspace</strong>
                <span class="text-foreground-muted block text-xs">Control surface error</span>
            </div>
        </header>

        <div class="px-5 py-8 sm:px-7">
            <span class="text-accent font-mono text-xs font-semibold tracking-wide uppercase">
                Error {page.status}
            </span>
            <h1 class="mt-3 text-2xl font-semibold tracking-tight">
                {page.status === 404 ? 'Page not found' : 'Something went wrong'}
            </h1>
            <p class="text-foreground-muted mt-2 text-sm leading-6">
                {page.error?.message ?? 'The control surface could not be loaded.'}
            </p>
        </div>

        <footer class="border-border grid gap-2 border-t p-4 sm:grid-cols-2 sm:px-7">
            <button
                type="button"
                class="text-accent-foreground bg-accent min-h-11 cursor-pointer rounded-lg border-0 px-4 text-sm font-semibold"
                onclick={() => window.location.reload()}>
                Try again
            </button>
            <a
                href="/"
                class="border-border-strong bg-surface-hover text-foreground-strong grid min-h-11 place-items-center rounded-lg border px-4 text-sm font-semibold no-underline">
                Back to workspace
            </a>
        </footer>
    </section>
</main>
