# Teleport Commands y LuckPerms en Pumpkin 26.3

Esta referencia Java fija el comportamiento que deberá repetir el [perfil AOT](AOT.md). Fabric Loader 0.20.1 corre dentro de la JVM de PatchBukkit, descubre el JAR original de Teleport Commands 1.3.5 en `mods/` y aplica sus Mixins. LuckPerms 5.5.71 controla el permiso `pumpkinhybrid.home` que comprueba el adaptador al exponer `/sethome` y `/home`. Pumpkin 26.3 conserva la conexión, la posición y el mundo.

## Preparación

1. Completar la [preparación de PatchBukkit y LuckPerms](NATIVE.md) y detener la instancia.
2. Con un JDK 25, desde el directorio del crate ejecutar `cargo run --locked -- prepare-teleport /ruta/a/instancia`. El comando instala Fabric y el mod con hashes fijados. Compila el agente y el adaptador con el Paper de la instancia.
3. En `plugins/data/patchbukkit/patchbukkit.config.toml`, usar `"-javaagent:fabric-agent.jar"` en la lista `extra-args` de `[jvm]`. Si la lista contiene el agente experimental `teleport-hook.jar`, sustituir esa entrada y conservar los demás argumentos. Arrancar desde este crate con `cargo run --locked -- run-fabric /ruta/a/instancia /ruta/al/binario/pumpkin`.
4. Esperar `Teleport Commands registered by Fabric Mixins` y entrar con un cliente 26.3. Las pruebas locales usaron Fabric Loader 0.20.1 en el cliente y una instancia desechable en loopback.

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

En una instancia preparada desde cero el cargador resolvió `teleport_commands@1.3.5`. Con el permiso en falso, el cliente recibió el rechazo y el archivo de datos quedó sin el hogar `blocked`. Con el permiso concedido, el JAR guardó un hogar en el bloque `(12, 49, 12)`. Desde `(44.5, 120, 44.5)`, Pumpkin confirmó el regreso a `(12.5, 49, 12.5)` y mantuvo al jugador conectado. Después del reinicio, LuckPerms confirmó el permiso y el mod volvió a llevar al jugador al destino guardado.

`run-fabric` ejecuta el mismo resolvedor de Fabric en un proceso Java breve antes de arrancar Pumpkin. Se puede invocar solo esa etapa con `cargo run --locked -- check-fabric /ruta/a/instancia`. Con Random MOTD y sin `fabric-api` la prueba devolvió código 1, nombró la dependencia ausente y dejó el puerto libre. La verificación cubre la resolución de dependencias. La prueba con Fabric API 0.166.2+26.3 completo pasó la resolución y falló al aplicar Mixins a Paper. Los errores incluyeron las firmas de `PlayerList`, `PackRepository` y `CommandSourceStack`, además de inyecciones en los procesos de registro de comandos y congelación de registros.

## Alcance

El agente comparte las clases de Minecraft con PatchBukkit. Conserva los bytes de clases transformadas para las redefiniciones posteriores de ByteBuddy y reduce el arranque Java a una llamada que mantiene el punto de inyección usado por el mod. Se invocan las entradas declaradas en Fabric sin un segundo bucle de simulación. El adaptador publica dos comandos de un dispatcher separado y envía el teletransporte a Pumpkin a través de PatchBukkit.

Quedan por validar los ciclos de tick y parada de Fabric API, las transformaciones fuera de las clases de Minecraft y Mojang, el registro de comandos de otros mods, los teletransportes entre dimensiones y los efectos visuales. Las pruebas son funcionales sobre una compilación de desarrollo. El consumo del puente sigue por medir.
