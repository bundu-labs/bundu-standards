# Bundu Identity

> The sovereign identity gateway for the Bundu Family ecosystem.

**Status:** Scaffold (v0.1.0) — pre-production, foundational structure in place.
**License:** Apache 2.0
**Owner:** Bundu Foundation (Zimbabwe CLG)
**Reference architecture:** [Mukoko Architecture v4.0.3](../mukoko-platform/docs/IDENTITY_MODEL_AUTH_AMENDMENT_v4_0_3.md)

---

## What this is

Bundu Identity is the sole integration boundary between the Bundu ecosystem and the WorkOS authentication provider. It is a Rust HTTP service that:

1. Translates WorkOS sessions into platform JWTs
2. Resolves WorkOS users and organisations into Supabase `identity.person` and `identity.entity` rows
3. Manages family-of-one bootstrap on signup (per v4.0.3 identity model)
4. Publishes identity events to the service bus (Cloudflare Queues per v4.0.4)
5. Mints platform JWTs that every other service across the ecosystem trusts

Every other service in the Bundu ecosystem — FastAPI services, Cloudflare Workers, Next.js applications, Capacitor mobile shells, native iOS/Android/HarmonyOS apps, Rust services like NTL and SiafuDB — **validates platform JWTs only**. They never talk to WorkOS directly. Bundu Identity is the choke point.

## Why Rust

- **Single binary, ~30MB image, ~50MB RAM at idle.** Deploys to Fly.io, K3s, or bare metal at Teraco identically.
- **Memory safety and predictable latency** for an auth choke point that every authenticated request flows through.
- **Joins the Foundation Rust trio**: NTL (transfer layer), SiafuDB (graph database), Bundu Identity (auth gateway). Frontier infrastructure built in a sovereign language stack.
- **Apache 2.0 throughout** — owned, extensible, never licence-locked.

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                       WorkOS AuthKit                         │
│              (id.mukoko.com — hosted UI)                     │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ↓
┌──────────────────────────────────────────────────────────────┐
│                   Bundu Identity Gateway                     │
│                       (this repo)                            │
│                                                              │
│   /auth/signin     → builds AuthKit URL                     │
│   /auth/callback   → exchanges code for WorkOS session      │
│   /auth/exchange   → upserts identity.person + identity.    │
│                       entity, mints platform JWT             │
│   /auth/refresh    → rotates platform JWT                   │
│   /auth/logout     → revokes session                        │
│   /entity/switch   → updates active_entity_id claim         │
│                                                              │
│   Health, metrics, OpenTelemetry traces                     │
└────┬──────────────────┬───────────────────┬─────────────────┘
     │                  │                   │
     ↓                  ↓                   ↓
┌─────────┐    ┌──────────────────┐   ┌────────────────┐
│Supabase │    │  Cloudflare      │   │ Service Bus    │
│PRIMARY  │    │  Queues          │   │ (events fired) │
│identity │    │  (publish        │   │                │
│ entity  │    │   identity.*)    │   │                │
│ member  │    └──────────────────┘   └────────────────┘
└─────────┘
     │
     ↑
┌────────────────────────────────────────────────────────────┐
│  All other services consume Bundu Identity's platform JWT  │
│  via standard JWT validation libraries — no direct WorkOS  │
│  dependency anywhere downstream.                           │
└────────────────────────────────────────────────────────────┘
```

## Crate layout

```
crates/
├── bundu-identity-server/   The HTTP service (axum). Entry point.
├── bundu-identity-core/     Domain types: Person, Entity, Membership, Session.
├── bundu-workos/            WorkOS REST client. Sole integration with WorkOS.
├── bundu-jwt/               Platform JWT minting, validation, claim contract.
├── bundu-supabase/          Supabase REST client for identity.person / identity.
│                            entity / entity.membership upserts. Service-role auth.
└── bundu-service-bus/       Cloudflare Queues publisher. Event envelope, schema
                             registry validation, publish/consume interfaces.
```

## Endpoint contract

All endpoints return JSON. Errors follow RFC 7807 problem details.

| Method | Path                  | Purpose |
|--------|----------------------|---------|
| GET    | `/health`             | Liveness probe |
| GET    | `/health/ready`       | Readiness (Supabase + WorkOS reachable) |
| GET    | `/auth/signin`        | Returns AuthKit redirect URL |
| GET    | `/auth/callback`      | Handles WorkOS OAuth callback, returns code-exchange result |
| POST   | `/auth/exchange`      | Exchanges WorkOS code/session for platform JWT |
| POST   | `/auth/refresh`       | Refreshes platform JWT before expiry |
| POST   | `/auth/logout`        | Revokes WorkOS session, blacklists JWT |
| POST   | `/entity/switch`      | Switches `active_entity_id` claim |
| GET    | `/entity/list`        | Lists all entities the authenticated person belongs to |
| GET    | `/.well-known/jwks.json` | Public keys for JWT verification (when ML-DSA / ES256 used) |

## Platform JWT claim contract

```json
{
  "iss": "https://id.mukoko.com",
  "sub": "<identity.person.id>",
  "aud": ["mukoko", "nyuchi"],
  "iat": 1714501834,
  "exp": 1714588234,
  "jti": "<UUIDv7>",

  "email": "person@example.com",
  "email_verified": true,
  "name": "Bryan Fawcett",
  "workos_user_id": "user_01HXYZ...",

  "entities": [
    {
      "id": "<identity.entity.id>",
      "type": "family",
      "name": "Bryan's family",
      "role": "head",
      "permissions": []
    },
    {
      "id": "<identity.entity.id>",
      "type": "organization",
      "name": "Nyuchi Africa",
      "role": "owner",
      "permissions": ["*"]
    }
  ],

  "active_entity_id": "<identity.entity.id>",
  "active_entity_type": "family",
  "active_role": "head"
}
```

The contract is **transport-agnostic** — every service across the ecosystem uses a standard JWT library to validate it, regardless of which language they're written in.

## Configuration

All configuration via environment variables. See `.env.example` for the full list.

Required:
- `WORKOS_CLIENT_ID`
- `WORKOS_API_KEY`
- `WORKOS_REDIRECT_URI`
- `SUPABASE_URL`
- `SUPABASE_SERVICE_ROLE_KEY`
- `JWT_SIGNING_SECRET` (HS256 for now; ML-DSA migration tracked per v4.0.3 amendment post-quantum section)
- `CF_ACCOUNT_ID`
- `CF_QUEUES_API_TOKEN`

Optional:
- `LISTEN_ADDR` (default `0.0.0.0:8080`)
- `LOG_LEVEL` (default `info`)
- `OTEL_EXPORTER_OTLP_ENDPOINT`

## Development

```bash
# Run locally (requires .env populated)
cargo run -p bundu-identity-server

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Format
cargo fmt --all
```

## Deployment

Phase 1 (now): Fly.io — `fly deploy` from `crates/bundu-identity-server`.
Phase 3 (future): Self-hosted on K3s at Teraco bare metal. Same binary, different orchestrator.

See `docs/DEPLOYMENT.md` (TODO) for the full deployment runbook.

## Related projects

- [mukoko-platform](https://github.com/nyuchi/mukoko-platform) — backend platform (FastAPI, Supabase, ScyllaDB)
- [NTL](https://github.com/openNTL/ntl) — Nyuchi Transfer Layer
- [SiafuDB](https://github.com/siafuDB/siafudb) — embedded property graph database
- [SiafuDB-Kuzu](https://github.com/siafuDB/siafudb-kuzu) — Kuzu fork

## Licence

Apache 2.0. See [LICENSE](LICENSE).

---

*Bundu Identity. One identity. Many entity memberships. Built sovereign.*
