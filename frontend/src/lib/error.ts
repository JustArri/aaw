import { unreachable } from './util';

type GatewayErrorOptions = ErrorOptions & {
    responseBody?: string;
};

export class GatewayError extends Error {
    constructor(
        public readonly code:
            | 'not-responding'
            | 'internal-server-error'
            | 'forbidden'
            | 'bad-request',
        options?: GatewayErrorOptions
    ) {
        let message = options?.responseBody?.trim();

        if (!message) {
            switch (code) {
                case 'not-responding':
                    message = 'Gateway not responding';
                    break;
                case 'internal-server-error':
                    message = 'Internal server error';
                    break;
                case 'forbidden':
                    message = 'Not allowed';
                    break;
                case 'bad-request':
                    message = 'Bad request';
                    break;
                default:
                    unreachable(code);
            }
        }

        message = `Gateway error: ${message}`;

        super(message, options);
    }
}

export function gatewayNotResponding(cause?: unknown) {
    return new GatewayError('not-responding', { cause });
}

export type GatewayErrorCode = 400 | 401 | 500;

export function gatewayErrorByStatus(status: GatewayErrorCode, responseBody?: string) {
    const options = { responseBody };

    switch (status) {
        case 400:
            return new GatewayError('forbidden', options);
        case 401:
            return new GatewayError('bad-request', options);
        case 500:
            return new GatewayError('internal-server-error', options);
        default:
            unreachable(status);
    }
}

export class GraphNodeError extends Error {
    constructor(public readonly code: 'unknown-id' | 'operation-failed') {
        let message;

        switch (code) {
            case 'unknown-id':
                message = 'Graph node does not exist';
                break;
            case 'operation-failed':
                message = 'Graph node operation failed';
                break;
            default:
                unreachable(code);
        }

        super(message);
    }
}

export type GraphNodeErrorCode = 404 | 500;

export function graphNodeErrorByStatus(status: GraphNodeErrorCode) {
    switch (status) {
        case 404:
            return new GraphNodeError('unknown-id');
        case 500:
            return new GraphNodeError('operation-failed');
        default:
            unreachable(status);
    }
}
