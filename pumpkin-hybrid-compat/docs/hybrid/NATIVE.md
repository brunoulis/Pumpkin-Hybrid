# Línea base de Pumpkin y pruebas con JVM

El control vanilla verifica el motor Rust. Las pruebas con PatchBukkit, Fabric y LuckPerms verifican comportamiento mediante una JVM de referencia. Los criterios para código generado y ejecución sin JVM están en el [diseño AOT](AOT.md).

## Selección de la versión

El checkout `87f8822ac` compila Pumpkin para Minecraft Java 26.3, protocolo 777. Fabric Meta ofrece Fabric Loader 0.20.1 estable para 26.3. NeoForge también publica para esa versión. El piloto usa Fabric y reserva el perfil 1.21.1 con NeoForge para las [pruebas de referencia](REFERENCE.md).

La versión `f750eebaf36a` del plugin upstream `pumpkin-java-multiversion` se intentó compilar como componente Wasm contra el checkout. El SDK rechaza `register_java_versions`, los eventos de conexión y `PacketSentEventData.extra_packets` que requiere ese commit. La ruta 1.21.1 sigue pendiente de una combinación compatible y una prueba de sincronización de registros. [Fabric Meta 26.3](https://meta.fabricmc.net/v2/versions/loader/26.3) · [Plugin de traducción](https://github.com/Pumpkin-MC/pumpkin-java-multiversion/tree/f750eebaf36ae14df22044e8c0150e865c2bfb07)

## Control vanilla

Compilé `cargo build --locked -p pumpkin` con Rust 1.100.0. Un cliente Minecraft Java 26.3 entró a un mundo local. La consola teletransportó al jugador, confirmó un `setblock` de oro y completó el guardado. El directorio de guardado contiene `level.dat` y el archivo de región. Una consulta al bloque tras reiniciar se encontró con el chunk descargado, así que la lectura puntual del bloque queda por repetir con un cliente en la zona.

## Candidato Fabric

Teleport Commands 1.3.5, SHA-256 `bc4d0b93a27fd8e9ef54b5960e4d0ee5df0c2c9c3fe95d84abb8dedc111b7773`, arrancó sin cambios de código en la JVM de referencia con Fabric Loader 0.20.1, Minecraft 26.3 y Temurin 25.0.5+8. Con un cliente conectado, `sethome verified` guardó la posición `(12.5, -60, 12.5)`. Después de enviar al jugador a `(44.5, -60, 44.5)`, `home verified` lo devolvió a la primera posición. La consola midió los tres estados mediante `data get entity <jugador> Pos`. El archivo `world/TeleportCommands/storage.json` guardó el UUID, las coordenadas y la dimensión `minecraft:overworld`.

El JAR declara una entrada Fabric, Java 25 y tres Mixins que se aplican a registro de comandos, arranque y muerte del jugador. El primer contrato del puente debe permitir registrar la rama Brigadier, resolver el jugador conectado y la dimensión, leer su posición y aplicar una solicitud de teletransporte. El criterio final es que la posición cambie en el mundo de Pumpkin y que un permiso de LuckPerms regule la acción del mismo jugador. [JAR de Teleport Commands](https://modrinth.com/mod/teleport-commands/version/EDcoFIQF) · [Fuentes del mod](https://github.com/MrSn0wy/TeleportCommands)

El cliente Minecraft 26.3 con Fabric Loader 0.20.1 también entró a la instancia de Pumpkin con PatchBukkit y LuckPerms. El [piloto integrado](TELEPORT.md) verifica la carga del JAR desde `mods/`, el registro por Mixins y `sethome` y `home` en esa instancia.

### Contratos observados para la prueba AOT

- Resolver la identidad de clases a partir de mappings y transformaciones del perfil. El piloto Java observó redefiniciones con ByteBuddy y fallos de Mixins contra firmas que Paper modifica.
- La referencia comprobó registro de comandos, permisos y teletransporte. El punto de entrada Rust para comandos es [Context::register_command_with_aliases](../../../crates/pumpkin-core/src/plugin/api/context.rs). Las operaciones de jugador terminan en [Player::request_teleport o Player::teleport_world](../../../crates/pumpkin-core/src/entity/player.rs).
- Repetir el ciclo de `sethome` y `home` con los módulos AOT, medir coordenadas y dimensión desde la sesión Rust y comprobar permiso y destino después de reiniciar. Un fallo del módulo debe dejar la posición intacta y devolver un diagnóstico al jugador.

## PatchBukkit

El binario Linux de PatchBukkit 0.2.2 arrancó la JVM y Paper 26.3 build 205 en modo headless con Pumpkin upstream 0.2.2+26.3-26.60. Al entrar el cliente, el proceso terminó con SIGSEGV. El stack del sistema localiza la caída en la serialización de `TextComponent` durante `AsyncPlayerPreLoginEvent`.

La reconstrucción de ambos lados de PatchBukkit desde `d6b55cfcb4ffbaa0edd3ca06e895aead71ce9c78`, con las dependencias Rust apuntando al checkout `87f8822ac`, permitió cargar LuckPerms. Un cliente 26.3 entró, el plugin devolvió `true` para un permiso heredado y la consola de Pumpkin teletransportó al jugador y guardó un cambio de bloque en ese mundo. Tras reiniciar, el cliente volvió a entrar, el permiso siguió siendo `true` y el filtro de bloques confirmó que el oro se había guardado. La copia intermedia que combinaba el JAR embebido de la release y la biblioteca reconstruida falló durante `HeadlessPaperServer.boot`.

### Repetir la prueba

1. Compilar el binario de Pumpkin en `87f8822ac` con `cargo +1.100.0 build --locked -p pumpkin --features plugin-v0_2`.
2. Clonar PatchBukkit en otro directorio y fijar `d6b55cfcb4ffbaa0edd3ca06e895aead71ce9c78`. En `rust/Cargo.toml`, cambiar las siete dependencias `pumpkin*` para que usen rutas absolutas a sus crates correspondientes en el checkout de Pumpkin, conservando el campo `package`. Copiar el `Cargo.lock` de Pumpkin al directorio `rust/` de PatchBukkit como punto de partida para la resolución.
3. Con `JAVA_HOME` apuntando a Java 25, ejecutar `./gradlew jar --no-daemon` en `java/`. Ejecutar `RUST_MIN_STACK=33554432 cargo +1.100.0 build` en `rust/`. El build Rust embebe `java/patchbukkit/build/libs/patchbukkit.jar`.
4. En una instancia de prueba separada, colocar `libpatchbukkit.so` en `plugins/` y el JAR de LuckPerms del [perfil de referencia](../../artifacts.lock) en `plugins/data/patchbukkit/patchbukkit-plugins/`. Arrancar Pumpkin con Java 25 disponible para que PatchBukkit prepare Paper 26.3 build 205.
5. Tras `Successfully enabled` y `PatchBukkit background initialization complete`, entrar con un cliente 26.3. Usar el nombre de ese jugador en los comandos de consola:

```text
lp creategroup rust-smoke
lp group rust-smoke permission set compat.native.marker true
lp user PatchProbe parent add rust-smoke
lp user PatchProbe permission check compat.native.marker
tp PatchProbe 1 100 1
```

El último resultado del permiso debe ser `true`. En el mundo desechable, `setblock 1 99 1 minecraft:gold_block` prueba una mutación del motor tras la llegada del jugador. Ejecutar `save-all flush` y `stop`, arrancar y repetir el permiso. Con el cliente en la zona, `fill 1 99 1 1 99 1 minecraft:iron_block replace minecraft:gold_block` verifica que el bloque guardado se volvió a cargar.

Para las pruebas locales usé loopback y autenticación desactivada. El resultado cubre un plugin Java y operaciones del motor Rust en una misma sesión. El piloto acota la integración de Fabric a los contratos ejercitados por Teleport Commands.
