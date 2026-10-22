import type { Notification } from './generated/models';

export async function* notificationStream(
    url: URL,
    signal?: AbortSignal
): AsyncGenerator<Notification, void, unknown> {
    if (signal?.aborted) {
        return;
    }

    const socket = new WebSocket(url);
    const messages: Notification[] = [];
    let finished = false;
    let failure: unknown;
    let wake: (() => void) | undefined;

    function finish(error?: unknown) {
        failure = error;
        finished = true;
        wake?.();
    }

    function receive(event: MessageEvent) {
        if (finished) {
            return;
        }

        try {
            if (typeof event.data !== 'string') {
                throw new Error('Expected a text notification');
            }
            messages.push(JSON.parse(event.data) as Notification);
            wake?.();
        } catch (error) {
            finish(error);
        }
    }

    function close(event: CloseEvent) {
        if (!finished) {
            finish(
                event.wasClean
                    ? undefined
                    : new Error('Notification connection closed unexpectedly')
            );
        }
    }

    function error() {
        if (!finished) {
            finish(new Error('Notification connection failed'));
        }
    }

    function abort() {
        messages.length = 0;
        finish();
    }

    socket.addEventListener('message', receive);
    socket.addEventListener('close', close);
    socket.addEventListener('error', error);
    signal?.addEventListener('abort', abort, { once: true });

    try {
        while (true) {
            if (messages.length) {
                yield messages.shift()!;
            } else if (finished) {
                if (failure !== undefined) {
                    throw failure;
                }
                return;
            } else {
                await new Promise<void>((resolve) => {
                    wake = resolve;
                });
                wake = undefined;
            }
        }
    } finally {
        socket.removeEventListener('message', receive);
        socket.removeEventListener('close', close);
        socket.removeEventListener('error', error);
        signal?.removeEventListener('abort', abort);
        socket.close();
    }
}
