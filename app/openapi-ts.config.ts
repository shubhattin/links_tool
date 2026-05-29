import { defineConfig } from '@hey-api/openapi-ts';

export default defineConfig({
  input: './openapi/schema.json',
  output: 'src/lib/api/generated',
  plugins: [
    '@hey-api/typescript',
    {
      name: '@hey-api/sdk',
      operations: {
        containerName: 'Api',
        strategy: 'single',
        nesting: 'operationId'
      }
    },
    {
      name: 'zod',
      compatibilityVersion: 4
    },
    {
      name: '@hey-api/client-ky',
      runtimeConfigPath: './src/lib/api/api-client-runtime'
    }
  ]
});
