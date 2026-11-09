import type { Notification } from './generated/models';

export async function* notificationStream(
    url: URL,
    signal?: AbortSignal
): AsyncGenerator<Notification | null, void, unknown> {
    if (signal?.aborted) {
        return;
    }

    const source = new EventSource(url);
    const messages: (Notification | null)[] = [];
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

    function open() {
        messages.push(null);
        wake?.();
    }

    function error() {
        if (!finished) {
            source.close();
            finish(new Error('Notification connection failed'));
        }
    }

    function abort() {
        messages.length = 0;
        finish();
    }

    source.addEventListener('open', open);
    source.addEventListener('message', receive);
    source.addEventListener('error', error);
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
        source.removeEventListener('open', open);
        source.removeEventListener('message', receive);
        source.removeEventListener('error', error);
        signal?.removeEventListener('abort', abort);
        source.close();
    }
}
