export type NotificationKind = 'info' | 'success' | 'warning' | 'error';

export interface NotificationOptions {
    title?: string;
    kind?: NotificationKind;
    duration?: number | null;
    loading?: boolean;
}

export interface Notification extends NotificationOptions {
    id: number;
    message: string;
    kind: NotificationKind;
    duration: number | null;
    loading: boolean;
}

export interface NotificationHandle {
    id: number;
    close(): void;
    replace(message: string, options?: NotificationOptions): void;
    info(message: string, options?: KindOptions): void;
    success(message: string, options?: KindOptions): void;
    warning(message: string, options?: KindOptions): void;
    error(message: string, options?: KindOptions): void;
}

type KindOptions = Omit<NotificationOptions, 'kind'>;

const minimumReadingDuration = 1000;
const maximumReadingDuration = 15000;
const readingWordsPerMinute = 220;
const readingStartDelay = 500;
const titleRecognitionDelay = 450;
const sentencePause = 300;
const clausePause = 150;
const timers = new Map<number, ReturnType<typeof setTimeout>>();
let nextId = 1;

export const notifications = $state<Notification[]>([]);

function clearTimer(id: number) {
    const timer = timers.get(id);

    if (timer !== undefined) {
        clearTimeout(timer);
        timers.delete(id);
    }
}

function scheduleDismiss(notification: Notification) {
    clearTimer(notification.id);

    if (notification.duration !== null && notification.duration > 0) {
        const timer = setTimeout(() => {
            dismissNotification(notification.id);
        }, notification.duration);

        timers.set(notification.id, timer);
    }
}

function readingDuration(message: string, title?: string) {
    const text = `${title ?? ''} ${message}`.trim();
    const words = text.match(/[\p{L}\p{N}]+(?:['’.-][\p{L}\p{N}]+)*/gu)?.length ?? 0;
    const sentencePauses = text.match(/[.!?…]+/g)?.length ?? 0;
    const clausePauses = text.match(/[,;:]+/g)?.length ?? 0;
    const duration =
        readingStartDelay +
        (title ? titleRecognitionDelay : 0) +
        words * (60_000 / readingWordsPerMinute) +
        sentencePauses * sentencePause +
        clausePauses * clausePause;
    return Math.round(Math.min(maximumReadingDuration, Math.max(minimumReadingDuration, duration)));
}

function createNotification(
    id: number,
    message: string,
    options: NotificationOptions = {},
    previous?: Notification
): Notification {
    const title = options.title ?? previous?.title;

    return {
        id,
        message,
        title,
        kind: options.kind ?? 'info',
        duration:
            options.duration === undefined ? readingDuration(message, title) : options.duration,
        loading: options.loading ?? false
    };
}

export function dismissNotification(id: number) {
    clearTimer(id);

    const index = notifications.findIndex((notification) => notification.id === id);
    if (index !== -1) {
        notifications.splice(index, 1);
    }
}

export function clearNotifications() {
    for (const timer of timers.values()) {
        clearTimeout(timer);
    }

    timers.clear();
    notifications.splice(0, notifications.length);
}

export function replaceNotification(
    id: number,
    message: string,
    options: NotificationOptions = {}
) {
    const index = notifications.findIndex((notification) => notification.id === id);
    if (index === -1) {
        return;
    }

    const previous = notifications[index];
    const notification = createNotification(id, message, options, previous);
    notifications[index] = notification;
    scheduleDismiss(notification);
}

function createHandle(id: number): NotificationHandle {
    return {
        id,

        close() {
            dismissNotification(id);
        },

        replace(message, options) {
            replaceNotification(id, message, options);
        },

        info(message, options) {
            replaceNotification(id, message, {
                ...options,
                kind: 'info'
            });
        },

        success(message, options) {
            replaceNotification(id, message, {
                ...options,
                kind: 'success'
            });
        },

        warning(message, options) {
            replaceNotification(id, message, {
                ...options,
                kind: 'warning'
            });
        },

        error(message, options) {
            replaceNotification(id, message, {
                ...options,
                kind: 'error'
            });
        }
    };
}

export function pushNotification(
    message: string,
    options: NotificationOptions = {}
): NotificationHandle {
    const id = nextId++;
    const notification = createNotification(id, message, options);
    notifications.push(notification);
    scheduleDismiss(notification);
    return createHandle(id);
}

function pushKind(
    kind: NotificationKind,
    message: string,
    options: KindOptions = {}
): NotificationHandle {
    return pushNotification(message, {
        ...options,
        kind
    });
}

export const notify = {
    info(message: string, options?: KindOptions) {
        return pushKind('info', message, options);
    },

    success(message: string, options?: KindOptions) {
        return pushKind('success', message, options);
    },

    warning(message: string, options?: KindOptions) {
        return pushKind('warning', message, options);
    },

    error(message: string, options?: KindOptions) {
        return pushKind('error', message, options);
    }
};
