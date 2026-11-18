# Hoja de ruta native-first

Cada hito se verifica y queda en un commit de `feature/compat-runtime`. El compilador, el runtime y sus adaptadores se desarrollan en este crate independiente. Pumpkin conserva la simulación y el estado del mundo.

## 0. Evidencia y diseño

- Consultados Graphify y el código de Pumpkin para verificar el tick, la carga de plugins, la API de eventos y los puntos de mutación del mundo.
- Probados tres tamaños de mod con LuckPerms sobre Arclight 1.21.1. Verificados como referencia una fusión de orbes, la cocción en una olla, una red cinética y la persistencia. [Resultados Arclight](REFERENCE.md)
- Probados Teleport Commands Fabric y LuckPerms dentro de un entorno Java auxiliar con Pumpkin 26.3 propietario del mundo. Verificados `/sethome`, `/home`, cambios de permisos y reinicio. Este entorno fija el resultado esperado para el futuro piloto nativo. [Resultados con JVM](TELEPORT.md)
- Investigadas alternativas AOT y propuesta una IR tipada y una prueba con Cranelift a partir de bytecode de un mod original. Verificada la referencia Java para los casos normales y extremos del método inicial. [Investigación y prueba propuesta](AOT.md)

## 1. Primer módulo AOT sin JVM

1. Leer el JAR fijado de Teleport Commands y resolver automáticamente un método real con descriptor exacto. Conservar los offsets de bytecode para los diagnósticos.
2. Verificar la pila, los tipos y la inicialización de objetos del subconjunto de prueba. Traducir a la IR y emitir un objeto con Cranelift.
3. Publicar el módulo y su manifiesto. Cargar y ejecutar desde un proceso Rust sin `java` ni `libjvm`, mediante la interfaz de compatibilidad versionada.
4. Aprobar únicamente cuando el módulo supere toda la tabla diferencial de Java, los rechazos de bytecode incompatible y las pruebas de caché, integridad y ABI. Registrar tiempo de compilación fría y reutilización en caliente. [Casos y criterios](AOT.md)

## 2. Primer perfil nativo de juego

1. Extender el runtime a los objetos, llamadas y bibliotecas del flujo mínimo de comandos. Resolver un JAR Fabric y un JAR Bukkit originales de pivote. Implementar las transformaciones necesarias para esos binarios y sus dependencias antes de compilar.
2. Conectar ciclo de vida, comandos, permisos y persistencia. Pumpkin valida y ejecuta teletransportes, obtiene la posición y controla el orden de eventos. Las escrituras de cada extensión y las mutaciones del mundo tienen propietarios explícitos.
3. Umbral funcional del servidor. Un cliente real entra al mismo mundo Pumpkin, usa una acción originada en el módulo del mod y observa que el módulo del plugin la permite o deniega. Los dos JAR se compilan automáticamente y los dos artefactos ejecutados son nativos. Repetir después de reiniciar con caché caliente y sin JVM disponible.

## 3. Compatibilidad incremental

- Extender Fabric a contenido y API de eventos. Mantener casos verificables para fusión y NBT, cocción e inventario, y red cinética, entidades de bloque y sincronización del cliente.
- Incorporar adaptadores Forge y NeoForge, con un cargador por instancia. Los Mixins, transformaciones de bytecode y mappings se validan por perfil. Ampliar Bukkit, Spigot y Paper por contratos observados.
- Añadir despacho virtual, inicialización de clases, memoria gestionada, excepciones y bibliotecas Java según el corpus. Incorporar reflexión, monitores y carga dinámica con pruebas diferenciales propias.
- Revisar extensiones sin soporte y emitir motivos accionables. Si se estudia un fallback JVM, documentar su coste y sus resultados por separado.

## 4. Consumo, rendimiento y publicación

Comparar el motor nativo con Fabric, Paper y Arclight bajo el mismo hardware, escenario, versión y carga en la medida en que los perfiles permitan equivalencia. Registrar TPS, MSPT y percentiles, CPU, RSS, asignaciones, pausas de GC o gestión equivalente, arranque, coste del runtime y tiempos de compilación y caché. Probar comportamiento y persistencia con la misma carga. El [banco de pruebas existente](PERFORMANCE.md) instrumenta el coste fijo de los perfiles de referencia. El rendimiento del diseño AOT se medirá cuando exista un módulo nativo ejecutable.
