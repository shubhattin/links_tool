## app/ (Sveltekit App - Frontend)

- Use bun package manager always
- Instead of npx use bunx command.
- Use shadcn components, add needed shadcn components with this command. `bunx shadcn-svelte@latest add <component>`

## RCP Sync between server and client

- In the root directory run the commands `./scripts/sync-api.sh`. this genrates the typescript client.
- And then if there is a change in routes (added or removed) then also update `app/src/lib/api/query.ts` which organizes the generated trpc client options into a hierarchical format.