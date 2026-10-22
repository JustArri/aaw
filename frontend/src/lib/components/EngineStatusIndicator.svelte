<script lang="ts">
    import { gatewayOnline } from '$lib/state/gateway.svelte';

    let online = $derived(gatewayOnline());
</script>

<span
    class={`inline-block size-1.75 shrink-0 rounded-full ${online ? 'bg-success animate-pulse shadow-[0_0_0.5rem_var(--success-glow)] motion-reduce:animate-none' : 'bg-indicator-off'}`}
    role="img"
    aria-label={`Engine ${online ? 'online' : 'offline'}`}
    title={`Engine ${online ? 'online' : 'offline'}`}>
</span>
