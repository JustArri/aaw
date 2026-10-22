export interface RequestTracker {
    version: number;
    pendingVersion: number | null;
}

export function createRequestTracker(): RequestTracker {
    return { version: 0, pendingVersion: null };
}

export function invalidateRequest(request: RequestTracker) {
    request.version++;
}

export function beginRequest(request: RequestTracker) {
    if (request.pendingVersion === request.version) {
        return null;
    }
    request.pendingVersion = request.version;
    return request.version;
}

export function isCurrentRequest(request: RequestTracker, version: number) {
    return request.version === version;
}

export function finishRequest(request: RequestTracker, version: number) {
    if (request.pendingVersion === version) {
        request.pendingVersion = null;
    }
}
