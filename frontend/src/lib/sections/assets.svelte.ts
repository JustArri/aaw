import { getAssetsWebDavUrl } from '$lib/generated/api';

export class AssetsSection {
    copied = $state(false);
    url = this.makeDavUrl();

    private makeDavUrl() {
        const resolved = new URL(getAssetsWebDavUrl(), window.location.href);
        return resolved.href.replace(/^https?:/, (protocol) =>
            protocol === 'https:' ? 'davs:' : 'dav:'
        );
    }

    async copyUrl() {
        try {
            if (navigator.clipboard?.writeText) {
                await navigator.clipboard.writeText(this.url);
            } else {
                const textarea = document.createElement('textarea');
                textarea.value = this.url;
                textarea.style.position = 'fixed';
                textarea.style.opacity = '0';
                document.body.appendChild(textarea);
                textarea.focus();
                textarea.select();
                const success = document.execCommand('copy');
                textarea.remove();
                if (!success) throw new Error('Copy command failed');
            }
            this.copied = true;
            setTimeout(() => (this.copied = false), 2000);
        } catch (error) {
            console.error('Failed to copy URL:', error);
        }
    }
}
