<script lang="ts">
    import Icon from '@iconify/svelte';
    import { AssetsSection } from '$lib/sections/assets.svelte';

    const section = new AssetsSection();
</script>

<section
    class="border-border bg-surface-card grid min-h-70 place-items-center content-center rounded-xl border text-center">
    <span class="bg-accent/10 text-accent grid size-12 place-items-center rounded-[0.625rem]">
        <Icon icon="lucide:archive" width="1.5625rem" />
    </span>
    <h2 class="mt-3 text-sm">Asset manager</h2>
    <div class="text-foreground-muted mt-6 text-sm">
        Currently only a DAV server is available at
        <button
            type="button"
            onclick={() => section.copyUrl()}
            disabled={section.copied}
            class="enabled:hover:bg-accent/10 border-border-strong hover:border-accent disabled:border-border relative m-2 block cursor-pointer rounded-2xl border px-4 py-2 font-mono break-all transition-colors select-none">
            <span class:invisible={section.copied}>{section.url}</span>
            {#if section.copied}
                <span
                    class="text-foreground-muted absolute inset-0 flex items-center justify-center gap-2">
                    <Icon icon="lucide:check" width="1rem" />
                    Copied!
                </span>
            {/if}
        </button>
    </div>
</section>
