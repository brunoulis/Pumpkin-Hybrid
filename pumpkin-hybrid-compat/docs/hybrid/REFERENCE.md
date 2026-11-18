# Prueba funcional de la instancia de referencia

Ejecutada el 18 de noviembre de 2026 con Temurin Java 21.0.13+9. Un cliente real NeoForge 21.1.192 con los tres mods entró al mundo del perfil. La autenticación se desactivó temporalmente durante esa conexión de prueba en loopback. La configuración entregada vuelve a usar `online-mode=true`.

| Extensión | Resultado observado |
| --- | --- |
| Clumps 19.0.0.1 | Dos orbes de valores 3 y 7 pasaron a una entidad con `clumpedMap: {3: 1, 7: 1}`. El mapa sobrevivió a un reinicio. Al recoger el orbe, el jugador pasó de 0 XP a nivel 1 y 3 puntos de progreso, un total de 10 XP. |
| Farmer’s Delight 1.2.9 | La olla consumió arroz crudo y produjo `farmersdelight:cooked_rice` en la ranura 6. El resultado se conservó tras el reinicio. |
| Create 6.0.6 | El motor y el eje conectado registraron 16 RPM. Al romper el motor, el eje pasó a 0 RPM y volvió a 16 al reponerlo. El estado también se leyó a 16 RPM tras reiniciar. |
| LuckPerms Bukkit 5.5.71 | Un permiso de grupo persistió tras reiniciar y se heredó al asignar el grupo al jugador. Cambiar `bukkit.command.version` de falso a verdadero permitió ejecutar `/bukkit:version` con ese mismo jugador. |
| Protocolo Java | La consulta de estado devolvió el protocolo 767 y el cliente modificado completó la entrada al mundo. |

Con Farmer’s Delight 1.3.0 aparecieron errores de carga en dos recetas de integración con Create. El JAR empleaba el tipo `neoforge:tag` y Create 6.0.6 usa `fluid_tag` para ese serializador. La versión 1.2.9 usa el formato y la etiqueta de chocolate del Create fijado. El arranque de esa combinación completó sin los dos errores de recetas. La consulta externa de actualizaciones que hace `/bukkit:version` falló después de mostrar la versión local.

## Repetición en un mundo de prueba

Pega estos comandos en la consola de una instancia desechable. Tras unos segundos, comprueba el inventario de la olla, la velocidad y la fusión.

```text
forceload add 0 0
fill 0 119 0 10 119 5 minecraft:stone
setblock 2 120 0 minecraft:campfire
setblock 2 121 0 farmersdelight:cooking_pot{Inventory:{Size:8,Items:[{Slot:0,id:"farmersdelight:rice",count:1}]}}
setblock 5 120 0 create:creative_motor[facing=east]
setblock 6 120 0 create:shaft[axis=x]
summon minecraft:experience_orb 8.5 120.2 1.5 {Value:3s,Tags:["compat_orb"],NoGravity:1b}
summon minecraft:experience_orb 8.5 120.2 1.5 {Value:7s,Tags:["compat_orb"],NoGravity:1b}
lp creategroup compat-reference
lp group compat-reference permission set compat.smoke true
```

```text
data get block 2 121 0 Inventory
data get block 6 120 0 Speed
execute if entity @e[type=minecraft:experience_orb,tag=compat_orb]
data get entity @e[type=minecraft:experience_orb,tag=compat_orb,limit=1] clumpedMap
lp group compat-reference permission check compat.smoke
```

Ejecuta `save-all flush` y `stop`, arranca y repite las cinco consultas. Después, con un jugador conectado, compara `experience query <jugador> levels` y `experience query <jugador> points` antes y después de `tp <jugador> 8.5 120 1.5`. Prueba `/bukkit:version` desde el cliente con `lp user <jugador> permission set bukkit.command.version false` y de nuevo con `true`. Para comprobar la red de Create, quita el motor, mide la velocidad del eje y vuelve a poner el motor.

La prueba cubre estas operaciones de los JAR en Arclight. La [prueba del motor Pumpkin](ROADMAP.md) queda pendiente, igual que las contraptions móviles y el rendimiento bajo carga.
