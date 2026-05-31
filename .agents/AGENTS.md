## app/ (Sveltekit App - Frontend)

- Use bun package manager always
- Instead of npx use bunx command.
- Use shadcn components, add needed shadcn components with this command. `bunx shadcn-svelte@latest add <component>`

## API sync between server and client

- After you can make a change in routes in axum app generate the new OpenAPI schema.
- In the repo root, run `./scripts/sync-api.sh` to generate the OpenAPI TypeScript client.
- If routes are added or removed, also update `app/src/lib/api/query.ts`, which organizes the generated OpenAPI client options into a hierarchical format.