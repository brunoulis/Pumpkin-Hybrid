# Pumpkin Hybrid Compat

These instructions cover this standalone Cargo workspace. Follow the parent Pumpkin repository's instructions when working elsewhere.

## Architecture

- Preserve the product target **JAR in → native code out → Pumpkin Rust engine**. Install from original mod and plugin JARs. Generate native modules automatically and validate their ABI before loading.
- Keep Pumpkin authoritative for worlds, players, ticks, entities, registries, and saves. Use the public plugin interfaces where they suffice. Keep the compiler, compatibility runtime, and loader adapters in this workspace.
- Require one of Fabric, Forge, or NeoForge per modded instance. Document Bukkit, Spigot, and Paper support per API and profile. Keep Java runtime experiments and JVM-based behavioral baselines explicit in documentation and results.
- Treat class identity, Java initialization, object lifetimes, dispatch, exceptions, Java and Minecraft APIs, Mixins, reflection, and synchronization as compatibility contracts. Reject unsupported contracts with the profile, JAR or class, and relevant bytecode offset where available.
- Preserve synchronous mutation ordering in the server tick. Keep compilation and network or disk I/O away from the tick. Measure allocation, memory, CPU, tail latency, cold compilation and warm cache before claiming improvements.

## Evidence and delivery

- Consult the existing Graphify index, check its commit, and verify findings in the actual source. Keep architecture proposals distinct from observed behavior.
- Pin original JARs, loader versions, mappings, executable tools and input hashes for each case. Compare the generated module against the original JVM behavior. Record which execution runtime each result used and include a without-JVM verification for native milestones.
- Treat the compatibility launcher, the Java reference profiles, the AOT prototype, and the native server as separate execution paths in evidence. Keep test worlds, downloaded binaries and generated machine code out of commits.
- Update the compatibility evidence and roadmap at each verified milestone. Prefer small commits confined to this workspace on `feature/compat-runtime` and push them to the configured fork when authorized.

Read [vision](docs/hybrid/VISION.md), [architecture](docs/hybrid/ARCHITECTURE.md), [AOT experiment](docs/hybrid/AOT.md), and [roadmap](docs/hybrid/ROADMAP.md) before changing the native runtime direction.
