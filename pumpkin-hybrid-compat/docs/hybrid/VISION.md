# Visión de Pumpkin Hybrid

Pumpkin Hybrid quiere ofrecer un servidor Minecraft Java con un mundo administrado por Pumpkin, capaz de ejecutar mods Java existentes junto a plugins Java y plugins nativos de Pumpkin. La instalación debe resultar familiar: se elige un cargador por instancia y se colocan los JAR en `mods/` y `plugins/`.

El primer hito del motor exige una versión conocida de Minecraft, un mod original y un plugin Bukkit original funcionando simultáneamente en el mismo mundo. La prueba debe conectar un cliente compatible, disparar una acción del mod, comprobar un cambio observable del plugin y verificar persistencia después de reiniciar.

La compatibilidad se amplía por versión y API. Fabric, NeoForge y Forge tienen ciclos de carga distintos. Bukkit, Spigot y Paper requieren pruebas propias. Se conserva el soporte para plugins nativos y Wasm de Pumpkin y se limita la integración al crate externo y a las extensiones públicas imprescindibles.

Pumpkin es dueño de los ticks, los chunks, las entidades y la persistencia. El runtime Java ejecuta extensiones y solicita operaciones al motor. La compilación automática a código nativo, con caché y fallback JVM, es una línea de investigación posterior a una prueba funcional. El rendimiento se compara con cargas equivalentes y datos reproducibles.

[Arquitectura](ARCHITECTURE.md) · [Hoja de ruta](ROADMAP.md)
