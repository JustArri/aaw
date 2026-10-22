import type { GatewayErrorCode } from '$lib/error';
import * as api from '$lib/generated/api';
import { unreachable } from '$lib/util';
import { onMount } from 'svelte';

const AUTO_UPDATE_INTERVAL = 5000;

export type GatewayStatus = 'online' | 'internal-error' | 'offline';

const gatewayState = $state({
    status: 'offline' as GatewayStatus
});
let fetching = false;

export function gatewayOnline() {
    return gatewayState.status === 'online';
}

export function reportGatewayOffline() {
    gatewayState.status = 'offline';
}

export function reportGatewayError(status: GatewayErrorCode) {
    if (status === 500) {
        gatewayState.status = 'internal-error';
    }
}

export function autoUpdateGatewayState() {
    onMount(() => {
        void fetchGatewayState();
    });

    $effect(() => {
        if (gatewayOnline()) {
            return;
        }
        const timer = setInterval(() => {
            if (!gatewayOnline()) {
                void fetchGatewayState();
            }
        }, AUTO_UPDATE_INTERVAL);
        return () => clearInterval(timer);
    });
}

async function fetchGatewayState() {
    if (fetching) {
        return;
    }
    fetching = true;

    let rsp;

    try {
        rsp = await api.health();
    } catch (error) {
        reportGatewayOffline();
        return;
    } finally {
        fetching = false;
    }

    const status = rsp.status;

    switch (status) {
        case 200:
            gatewayState.status = 'online';
            break;
        case 500:
            reportGatewayOffline();
            break;
        default:
            unreachable(status);
    }
}
