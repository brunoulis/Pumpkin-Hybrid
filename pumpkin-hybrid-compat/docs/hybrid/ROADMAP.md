# Hoja de ruta de compatibilidad

## 0. Referencia con mods y plugin originales

Fijar Minecraft 1.21.1, NeoForge 21.1.192, Arclight FeudalKings 1.0.1 y Java 21. Ejecutar Clumps, Farmer’s Delight, Create y LuckPerms conjuntamente. Medir la fusión de orbes, el resultado de una receta en una olla, la propagación cinética, el permiso de LuckPerms y la persistencia. El crate Rust prepara y arranca esta instancia. Arclight posee ese mundo de referencia.

## 1. Contrato mínimo con Pumpkin

1. Buscar una revisión de Pumpkin compatible con 1.21.1 y compararla con el checkout actual. Arrancar en un directorio desechable y verificar conexión, comando y guardado. Conservar un perfil vanilla de control.
2. Auditar la entrada de un mod Fabric que utilice un comando o evento sencillo, el ciclo de vida de LuckPerms y la viabilidad de PatchBukkit. Fijar las versiones y enumerar las clases de Minecraft que necesita cada JAR.
3. Diseñar un contrato de comandos, permisos y eventos para una única operación real. Ejecutarla a través del punto de entrada público más estrecho de Pumpkin. Probar rechazo sin mutación, timeout y reinicio antes de añadir operaciones.

## 2. Primera prueba con Rust como motor

Ejecutar el JAR Fabric original y el JAR Bukkit original en la misma sesión con el mundo administrado por Pumpkin. Un cliente entra, activa la operación del mod y comprueba que un permiso persistente del plugin cambia el resultado. Comparar los resultados con la instancia de referencia correspondiente y medir el coste del puente. Este paso cumple el mínimo del motor.

## 3. AOT y expansión

1. Evaluar Cranelift y HotSpot con un método extraído de una ruta realmente ejecutada. Añadir caché y fallback solo tras demostrar equivalencia, invalidación y medición reproducible.
2. Extender Fabric a registros y Mixins. Para Clumps probar fusión por valor, recogida y NBT. Para Farmer’s Delight probar sincronización de registros, cocción e inventario persistido. Para Create probar red cinética, desconexión y movimiento de una contraption.
3. Añadir perfiles NeoForge y Forge al motor y ampliar cobertura Bukkit, Spigot y Paper por contratos. Cada instancia selecciona un cargador.

## 4. Rendimiento y publicación

Por perfil, comparar tiempo de arranque, MSPT y TPS bajo carga, CPU, RSS, asignaciones, pausas de GC, coste del puente, tiempo AOT con caché fría/caliente y comportamiento al reiniciar. Usar mismo hardware, mundo, cliente y escenario. Registrar versiones exactas, resultados y límites. Publicar cada hito verificado en `feature/compat-runtime`.
