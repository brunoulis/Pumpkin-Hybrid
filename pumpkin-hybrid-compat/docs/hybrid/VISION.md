# Visión de Pumpkin Hybrid

**JAR IN → NATIVE CODE OUT → PUMPKIN RUST ENGINE**

Pumpkin Hybrid compilará automáticamente el bytecode de mods y plugins Java originales a módulos de código máquina. Pumpkin ejecutará esos módulos y conservará la autoridad sobre el mundo, la simulación, los jugadores y el guardado. El compilador y el runtime de compatibilidad vivirán en el crate independiente para facilitar la actualización de Pumpkin.

El administrador elige Fabric, Forge o NeoForge para cada instancia y coloca los JAR originales en `mods/` y `plugins/`. El sistema comprueba versiones, mappings y dependencias, aplica las transformaciones compatibles y compila cuando cambian las entradas. Reutiliza el módulo de caché cuando su identidad y ABI coinciden. El administrador y los autores usan los JAR publicados. El servidor final debe poder ejecutar esos módulos sin instalar una JVM tradicional.

Los plugins Bukkit, Spigot y Paper tendrán cobertura por API y versión y podrán combinarse con el cargador elegido. Se conserva la extensibilidad propia de Pumpkin. Una extensión que requiera semántica todavía ausente debe generar un diagnóstico concreto. Un resultado positivo de compatibilidad exige comprobar el comportamiento del JAR y el estado del mundo en el motor.

La JVM tiene un papel de referencia para ejecutar el JAR original, medir resultados y localizar divergencias. Los pilotos existentes prueban contratos de compatibilidad antes de trasladarlos al runtime nativo. Cualquier fallback transitorio se evalúa aparte y se identifica en los resultados.

El primer hito de investigación es compilar automáticamente un método de un JAR real, cargar su código máquina desde Rust y comparar los resultados con Java. El primer hito del servidor es ejecutar nativamente un mod de un cargador conocido y un plugin Java original en la misma partida sin JVM, con una acción de juego comprobable y persistencia al reiniciar. Los objetivos de consumo se validarán con benchmarks reproducibles.

[Arquitectura](ARCHITECTURE.md) · [Investigación AOT](AOT.md) · [Hoja de ruta](ROADMAP.md)
