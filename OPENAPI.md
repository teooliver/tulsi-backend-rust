# OpenAPI & TypeScript Type Generation

## Why OpenAPI (utoipa) instead of ts-rs?

We evaluated several approaches for sharing Rust backend types with the TypeScript frontend:

### Options considered

1. **`ts-rs`** — Derive macro that generates `.ts` files directly from Rust structs.
2. **`utoipa` + OpenAPI spec** — Generates an OpenAPI 3.1 specification from Rust types and route annotations, serves Swagger UI, and enables TypeScript type generation via `openapi-typescript` on the client side.
3. **Manual endpoint** — Serve types as text from a custom endpoint.
4. **`specta`** — Type export library tightly coupled with `rspc`.

### Why we chose utoipa

**`ts-rs` is simpler** — one derive per struct, no handler annotations — but it only generates data shapes. It doesn't describe endpoints, HTTP methods, parameters, request/response mappings, or status codes. The frontend gets types but no API contract.

**`utoipa` gives us both API documentation and type generation from a single source of truth:**

- **Swagger UI** at `/swagger-ui/` — interactive API documentation that any team member can use without reading code.
- **OpenAPI JSON** at `/api-docs/openapi.json` — a machine-readable spec that can generate typed clients in any language, not just TypeScript.
- **TypeScript types** — generated on the frontend via `openapi-typescript`, including both data models and endpoint signatures (paths, parameters, response types).

The tradeoff is more annotation work on the Rust side (`#[utoipa::path(...)]` on every handler), but since all our handlers follow consistent CRUD patterns, the annotations are repetitive and low-effort.

### Generating TypeScript types

From the frontend project:

```bash
# With the server running
npx openapi-typescript http://localhost:3000/api-docs/openapi.json -o src/api-types.ts

# Or from a saved spec file (no server needed)
curl http://localhost:3000/api-docs/openapi.json > openapi.json
npx openapi-typescript openapi.json -o src/api-types.ts
```

### Summary

| | ts-rs | utoipa + openapi-typescript |
|---|---|---|
| Data type generation | Yes | Yes |
| API endpoint docs | No | Yes (Swagger UI) |
| Typed endpoint signatures | No | Yes (paths, params, responses) |
| Multi-language support | TypeScript only | Any language with OpenAPI tooling |
| Rust-side effort | 1 derive per struct | 1 derive per struct + 1 macro per handler |
| Frontend tooling needed | None (direct .ts files) | `openapi-typescript` (npx, no install) |
