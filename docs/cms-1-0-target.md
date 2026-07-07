# skrifheim Optional Publishing Extension Target

Status: planning document

This document records requirements from a future CMS-style application family.
Those requirements are inputs for optional publishing extension crates, not a
definition of the mandatory `skrifheim` core database.

The 1.0 database core must expose the generic facts, worlds, policies,
projections, publication-release, and AI artifact primitives that make a future
Rust CMS possible as a thin typed application layer. The CMS schema itself must
live outside the core.

## Required Database Capabilities

- content as signed, versioned facts,
- optional extension-owned site, content node, block, media, user, role, theme,
  plugin, release, and policy models,
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
- Collaborative text editing must use the selected `v0.38.4` convergence model;
  the optional publishing extension must not invent independent OT/CRDT
  semantics outside the database plan.

## 1.0 Fit

`skrifheim` 1.0 does not need a full CMS implementation. It must expose the
database primitives that make an optional publishing extension possible:

- create/fork/promote worlds,
- store and query facts,
- record releases,
- track render dependencies,
- enforce public/private world boundaries,
- rebuild projections.
