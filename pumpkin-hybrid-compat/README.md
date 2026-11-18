# Pumpkin Hybrid Compat

Crate independiente para preparar y ejecutar la instancia de referencia de Pumpkin Hybrid. El perfil usa **Minecraft Java 1.21.1, NeoForge 21.1.192 y Arclight FeudalKings 1.0.1**. La JVM ejecuta esta instancia. El [objetivo del motor Rust](docs/hybrid/VISION.md) tiene sus propios criterios de aceptación.

## Arranque

Desde este directorio, con Cargo, Java 21, `curl` y `sha256sum`:

```bash
cargo run --locked -- prepare
cargo run --locked -- prepare-client
```

Si aceptas la [EULA de Minecraft](https://www.minecraft.net/eula), ejecuta:

```bash
cargo run --locked -- prepare --accept-eula
cargo run --locked -- run
```

En un checkout ya preparado también puedes ejecutar `./target/debug/pumpkin-hybrid-compat run`. En la consola del servidor, `stop` guarda el mundo y cierra el proceso.

- `.instance/` contiene el servidor, el mundo, `server.properties`, `mods/` y `plugins/`. La configuración inicial escucha en `127.0.0.1:25575` con autenticación online. `prepare` conserva la configuración existente.
- Instala el cliente Minecraft 1.21.1 con NeoForge 21.1.192. Copia los tres JAR de `.client/mods/` a la carpeta de mods de ese perfil y conecta a `127.0.0.1:25575`. El JAR de LuckPerms reside en el servidor.
- El lanzador busca Java en `PUMPKIN_COMPAT_JAVA`, en `.local/java/bin/java` y en `PATH`, por ese orden. Arclight descarga las dependencias de Minecraft y NeoForge en el primer arranque. Los JAR fijados en [artifacts.lock](artifacts.lock) se comprueban por SHA-256 al preparar y al ejecutar.

## Estado

[Referencia con tres mods](docs/hybrid/REFERENCE.md) · [Prueba de LuckPerms sobre Pumpkin](docs/hybrid/NATIVE.md) · [Piloto de Teleport Commands](docs/hybrid/TELEPORT.md) · [Arquitectura](docs/hybrid/ARCHITECTURE.md) · [Hoja de ruta](docs/hybrid/ROADMAP.md)
