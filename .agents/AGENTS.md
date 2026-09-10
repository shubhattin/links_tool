## app/ (Sveltekit App - Frontend)

- Use bun package manager always
- Instead of npx use bunx command.
- Use shadcn components, add needed shadcn components with this command. `bunx shadcn-svelte@latest add <component>`

## API types between server and client

- API request/response shapes are declared manually in `app/src/lib/api/api_types.ts` (zod schemas + inferred types).
- Call the Axum routes directly with the shared `ky` instance in `app/src/lib/api/ky.ts`.
- After changing a route or DTO, update `api_types.ts` by hand to match the Rust structs.