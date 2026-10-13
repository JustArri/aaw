import { defineConfig } from 'orval';

export default defineConfig({
    api: {
        input: {
            target: './openapi.json'
        },
        output: {
            target: './src/lib/generated/api.ts',
            schemas: './src/lib/generated/models',
            client: 'fetch',
            clean: true
        }
    }
});
