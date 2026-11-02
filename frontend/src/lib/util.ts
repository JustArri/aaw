export function unreachable(x: never): never {
    throw new Error('An unreachable statement reached! This is a bug.');
}

export function isStatusSuccess(status: number) {
    return status >= 200 && status < 300;
}
