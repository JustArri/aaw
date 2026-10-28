<script lang="ts">
    import { getAssetsWebDavUrl } from '$lib/generated/api';
    import Icon from '@iconify/svelte';

    let copied = $state(false);

    const url = makeDavUrl();

    function makeDavUrl() {
        const resolved = new URL(getAssetsWebDavUrl(), window.location.href);
        const url = resolved.href.replace(/^https?:/, (protocol) =>
            protocol === 'https:' ? 'davs:' : 'dav:'
        );
        return url;
    }

    async function copyUrl() {
        try {
            if (navigator.clipboard?.writeText) {
                await navigator.clipboard.writeText(url);
            } else {
                const textarea = document.createElement('textarea');
                textarea.value = url;
                textarea.style.position = 'fixed';
                textarea.style.opacity = '0';

                document.body.appendChild(textarea);
                textarea.focus();
                textarea.select();

                const success = document.execCommand('copy');
                textarea.remove();

                if (!success) {
                    throw new Error('Copy command failed');
                }
            }

            copied = true;
            setTimeout(() => (copied = false), 2000);
        } catch (error) {
            console.error('Failed to copy URL:', error);
        }
    }
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
            onclick={copyUrl}
            disabled={copied}
            class="enabled:hover:bg-accent/10 border-border-strong hover:border-accent disabled:border-border relative m-2 block cursor-pointer rounded-2xl border px-4 py-2 font-mono break-all transition-colors select-none">
            <span class:invisible={copied}>{url}</span>
            {#if copied}
                <span
                    class="text-foreground-muted absolute inset-0 flex items-center justify-center gap-2">
                    <Icon icon="lucide:check" width="1rem" />
                    Copied!
                </span>
            {/if}
        </button>
    </div>
</section>
