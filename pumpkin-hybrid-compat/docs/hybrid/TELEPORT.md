# Teleport Commands y LuckPerms en Pumpkin 26.3

El piloto ejecuta el JAR original de Teleport Commands 1.3.5 en la JVM de PatchBukkit junto a LuckPerms 5.5.71. Pumpkin 26.3 conserva la conexión, la posición y el mundo. El adaptador expone `/sethome` y `/home` y consulta `pumpkinhybrid.home` mediante Bukkit antes de ejecutar el dispatcher original del mod.

## Preparación

1. Completar la [preparación de PatchBukkit y LuckPerms](NATIVE.md) y detener la instancia.
2. Con un JDK 25, desde el directorio del crate ejecutar `cargo run --locked -- prepare-teleport /ruta/a/instancia`. El comando instala los JAR originales con su hash fijado y compila el adaptador con el Paper de la instancia.
3. En `plugins/data/patchbukkit/patchbukkit.config.toml` de esa instancia, añadir `"-javaagent:teleport-hook.jar"` a la lista `extra-args` de la sección `[jvm]`, conservando cualquier argumento existente. Arrancar Pumpkin desde el directorio de la instancia.
4. Esperar `Original Teleport Commands initialized` y entrar con un cliente 26.3. Las pruebas locales usaron Fabric Loader 0.20.1 en el cliente y una instancia desechable en loopback.

## Prueba reproducible

Con el jugador conectado, usa su nombre en la consola de Pumpkin:

```text
lp user TeleportProbe permission set pumpkinhybrid.home false
```

En el cliente, ejecutar `/sethome blocked` y comprobar el rechazo por permisos. Habilitar el permiso desde la consola, colocar al jugador en una posición segura y leer sus coordenadas:

```text
lp user TeleportProbe permission set pumpkinhybrid.home true
tp TeleportProbe 12 120 12
data get entity TeleportProbe Pos
```

Ejecutar `/sethome verified` en el cliente. El mod guarda las coordenadas a nivel de bloque en `plugins/data/patchbukkit/paper-runtime/worlds/world/TeleportCommands/storage.json`. Para que la altura se mantenga estable, colocar una plataforma o activar el vuelo creativo antes de registrar el destino. Mover al jugador lejos con `tp`, pedir `/home verified` desde el cliente y verificar la vuelta con `data get entity TeleportProbe Pos`. Ejecutar `stop`, reiniciar y comprobar que el mismo permiso y destino siguen disponibles.

## Resultado observado

Con el permiso en falso, el cliente recibió el rechazo y el hogar `blocked` quedó ausente del archivo. Con LuckPerms en verdadero, el JAR guardó el hogar `finalproof` en el bloque `(12, 49, 12)` del mundo principal. Desde `(44.5, 120, 44.5)`, Pumpkin confirmó la vuelta a `(12.5, 49, 12.5)` y mantuvo al jugador conectado. Después de reiniciar, LuckPerms conservó el permiso y la misma orden volvió a mover al jugador al destino guardado.

## Alcance

El adaptador invoca la entrada Fabric original y los métodos que el mod llama desde sus Mixins de arranque y comandos. Carga el JAR sin modificar, en un cargador de clases que comparte los tipos de Minecraft con PatchBukkit. El agente Java redirige al callback de teletransporte de jugador de PatchBukkit para que Pumpkin emita el paquete y gestione su confirmación.

Los hooks restantes, el descubrimiento y la aplicación automática de Mixins por Fabric Loader en el servidor, los teletransportes entre dimensiones y los efectos visuales del mod quedan por validar. Las pruebas son funcionales sobre una compilación de desarrollo. No se midió el coste del puente.
