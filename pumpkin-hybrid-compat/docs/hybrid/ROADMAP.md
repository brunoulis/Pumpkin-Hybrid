# Hoja de ruta de compatibilidad

## 0. Referencia con mods y plugin originales

Fijar Minecraft 1.21.1, NeoForge 21.1.192, Arclight FeudalKings 1.0.1 y Java 21. Ejecutar Clumps, Farmer’s Delight, Create y LuckPerms conjuntamente. Medir la fusión de orbes, el resultado de una receta en una olla, la propagación cinética, el permiso de LuckPerms y la persistencia. El crate Rust prepara y arranca esta instancia. Arclight posee ese mundo de referencia.

## 1. Contrato mínimo con Pumpkin

1. Usar Minecraft 26.3 en el primer piloto del motor. Pumpkin acepta ese protocolo y existe Fabric Loader estable para la versión. El perfil 1.21.1 sigue siendo la referencia de Clumps, Farmer’s Delight y Create. Conservar un control vanilla en el commit de Pumpkin elegido.
2. Probar PatchBukkit y el JAR original de LuckPerms sobre un proceso Pumpkin con cliente real. Verificar qué operaciones de Paper utiliza el puente y qué almacenes escribe. Fijar versiones y compilador conjuntamente por el ABI Rust.
3. Usar Teleport Commands 1.3.5 como candidato Fabric. Fijar un caso de `/sethome` y `/home` en Fabric de referencia. Inventariar el registro de Brigadier, la identidad del jugador, el acceso a la dimensión y la llamada de teleportación.
4. Diseñar el contrato mínimo para llevar esa operación al servidor Rust. Ejecutarla a través del punto de entrada público más estrecho de Pumpkin. Probar rechazo sin mutación, timeout y reinicio antes de añadir operaciones.

## 2. Primera prueba con Rust como motor

Ejecutar el JAR Fabric original y el JAR Bukkit original en la misma sesión con el mundo administrado por Pumpkin. Un cliente entra, activa la operación del mod y comprueba que un permiso persistente del plugin cambia el resultado. Comparar los resultados con la instancia de referencia correspondiente y medir el coste del puente. Este paso cumple el mínimo del motor.

El [piloto de Teleport Commands](TELEPORT.md) usa Fabric Loader para descubrir el JAR original, resolver sus dependencias y aplicar sus Mixins en la JVM embebida. El adaptador conecta dos comandos con el jugador de Pumpkin y comprueba un permiso de LuckPerms. El siguiente paso es añadir otro JAR del perfil usando la misma infraestructura, comprobar eventos del tick y sustituir el adaptador específico por un registro de comandos configurable.

## 3. AOT y expansión

1. Evaluar Cranelift y HotSpot con un método extraído de una ruta realmente ejecutada. Añadir caché y fallback solo tras demostrar equivalencia, invalidación y medición reproducible.
2. Extender Fabric a registros y Mixins. Para Clumps probar fusión por valor, recogida y NBT. Para Farmer’s Delight probar sincronización de registros, cocción e inventario persistido. Para Create probar red cinética, desconexión y movimiento de una contraption.
3. Añadir perfiles NeoForge y Forge al motor y ampliar cobertura Bukkit, Spigot y Paper por contratos. Cada instancia selecciona un cargador.

## 4. Rendimiento y publicación

Por perfil, comparar tiempo de arranque, MSPT y TPS bajo carga, CPU, RSS, asignaciones, pausas de GC, coste del puente, tiempo AOT con caché fría/caliente y comportamiento al reiniciar. Usar builds optimizadas, mismo hardware, mundo, cliente y escenario. Medir Pumpkin solo, Pumpkin con plugin y Pumpkin con plugin y mod para atribuir el consumo incremental. Añadir la referencia Java cuando pueda ejecutar el mismo escenario. Registrar versiones exactas, mediana, p95 y p99 de tick, máximos de memoria y límites. Publicar cada hito verificado en `feature/compat-runtime`.

El [banco de pruebas](PERFORMANCE.md) recoge el coste fijo de arranque y reposo de los tres perfiles.
