# skrifheim CMS 1.0 Target

Status: planning document

The 1.0 database must be able to support a future Rust CMS as a thin typed application layer over facts, worlds, policies, projections, publication releases, and AI artifacts.

## Required Database Capabilities

- content as signed, versioned facts,
- site, content node, block, media, user, role, theme, plugin, release, and policy models,
- draft worlds,
- review worlds,
- public-live world,
- atomic publish and rollback by world promotion,
- dependency graph for render/cache invalidation,
- sanitized public projections,
- private internal worlds,
- content-addressed media,
- policy-aware search projections,
- AI artifacts with source world, source transaction, model, prompt hash, confidence, and reviewer fields.

## Security Rules

- Public site serving must not query private worlds.
- Plugins do not receive raw database connections.
- Plugin and theme execution must use capability-limited WASM when implemented.
- AI cannot directly publish, declassify, or rewrite authoritative content.
- Sensitive content requires approval workflows before release.

## 1.0 Fit

`skrifheim` 1.0 does not need a full CMS implementation, but it must expose the database primitives that make the CMS model possible:

- create/fork/promote worlds,
- store and query facts,
- record releases,
- track render dependencies,
- enforce public/private world boundaries,
- rebuild projections.
