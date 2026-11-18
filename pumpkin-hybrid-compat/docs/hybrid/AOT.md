# Investigación AOT y primera prueba propuesta

## Alternativas investigadas

| Tecnología | Qué aporta al diseño | Límite para Pumpkin Hybrid |
| --- | --- | --- |
| GraalVM Native Image | Traduce Java a ejecutables y bibliotecas nativas. Ofrece análisis de alcance, metadatos para reflexión y modelos de inicialización. | Su hipótesis de mundo cerrado y los requisitos de configuración no resuelven por sí solos la carga de JAR arbitrarios, las transformaciones de loaders ni el enlace con las APIs de Pumpkin. Es una referencia para semántica, análisis y rendimiento. [Native Image](https://www.graalvm.org/latest/reference-manual/native-image/) · [Metadatos de alcance](https://www.graalvm.org/latest/reference-manual/native-image/metadata/) |
| OpenJDK Leyden y Eclipse OpenJ9 | Enseñan estrategias de preparación y caché. Leyden prepara carga y enlace de clases. OpenJ9 reutiliza métodos compilados AOT desde su caché compartida. | Ambos ejecutan mediante su JVM. Una caché del runtime Java no define el ABI ni los módulos cargables por el host Rust. [JEP 483](https://openjdk.org/jeps/483) · [OpenJ9 AOT](https://eclipse.dev/openj9/docs/aot/) |
| TeaVM | Muestra cómo traducir bytecode a otros runtimes y modelar bibliotecas Java para un destino concreto. | Sus destinos documentados y las restricciones de reflexión y bibliotecas no bastan para los módulos nativos y APIs de Minecraft que requiere el servidor. [TeaVM](https://teavm.org/docs/intro/overview.html) |
| Cranelift | Backend escrito en Rust que permite bajar una IR a código máquina. Es adecuado para probar emisión de objetos embebida. | El frontend Java, el runtime, la verificación, las transformaciones y los contratos con Minecraft siguen siendo trabajo del proyecto. LLVM se puede evaluar con la misma IR si aparecen necesidades concretas de destino u optimización. [Cranelift](https://cranelift.dev/) |

Para la primera prueba se propone Cranelift por su integración directa con el crate Rust. El resultado esperado es código máquina generado, con tipos, excepciones y operaciones Java definidos en el frontend y su runtime.

## JAR y camino de ejecución observado

El artefacto es Teleport Commands 1.3.5 para Minecraft 26.3. SHA-256 `bc4d0b93a27fd8e9ef54b5960e4d0ee5df0c2c9c3fe95d84abb8dedc111b7773`. El JAR declara Java 25, Fabric Loader >=0.19.5, una entrada principal y Mixins de arranque, comandos y muerte del jugador. Inspeccioné metadatos y bytecode del JAR publicado con `unzip` y `javap -c -p`. [Artefacto original](https://modrinth.com/mod/teleport-commands/version/EDcoFIQF)

| Ruta | Dependencias observadas |
| --- | --- |
| Entrada y arranque | Fabric `ModInitializer`, Mixins de `MinecraftServer.runServer` y constructor de `Commands`. El arranque obtiene la ruta del mundo y usa `Path`, E/S y Gson para datos del mod. |
| Registro y contexto | Brigadier registra y ejecuta ramas de comandos. `ServerPlayer`, UUID, `ServerLevel`, identificadores de dimensión y componentes de chat describen al jugador, el mundo y el resultado. |
| Acción de juego | `/sethome` registra la posición a nivel de bloque. `/home` resuelve dimensión y llama a `ServerPlayer.teleportTo`. También aparecen sonido, partículas, habilidades y un temporizador Java. |
| Método inicial | `dev/mrsnowy/teleport_commands/utils/tools.roundedBlockPos(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/core/BlockPos;`. Usa tres getters, `Math.floor`, `Math.round`, conversiones numéricas y el constructor de `BlockPos`. |

El método inicial contiene `new`, `dup`, `aload_0`, `invokevirtual`, `invokestatic`, `d2i`, `l2i`, `invokespecial` y `areturn`. En X y Z aplica piso y convierte de `double` a `int` con saturación. En Y redondea a `long` con empates hacia positivo y conserva los 32 bits bajos. El comportamiento de `Math.round` incluye NaN y saturación a `long`. Debe conservarse también cerca de los límites de redondeo de `double`. [JVMS §6](https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-6.html) · [Java 25 Math](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/lang/Math.html)

## IR inicial propuesta

Cada función incluye clase, método, descriptor y offsets de bytecode. El frontend mantiene una pila de valores tipados y locales al construir bloques básicos. Verifica tipos, valores sin inicializar, llamadas y retornos. Los tipos iniciales son `i32`, `i64`, `f64` y `ref<Class>`. Las operaciones distinguen conversiones Java, llamadas resueltas, reserva e inicialización de objetos y retornos. Las llamadas que puedan fallar tienen aristas explícitas hacia una excepción o un error del host. Una instrucción, clase o enlace fuera del perfil detiene la compilación del método y produce un diagnóstico con offset.

```text
roundedBlockPos(v: ref<Vec3>) -> ref<BlockPos> | exception
entry:
  p0 = new_uninit BlockPos
  x1 = call Vec3.x(v)
  x2 = java_d2i(java_floor(x1))
  y1 = call Vec3.y(v)
  y2 = java_l2i(java_round_f64_to_i64(y1))
  z1 = call Vec3.z(v)
  z2 = java_d2i(java_floor(z1))
  p1 = init BlockPos(p0, x2, y2, z2)
  return p1
```

`call Vec3.x` propaga la excepción de referencia nula. Verificar `new`, el uso de la copia sin inicializar y la llamada al constructor antes de bajar la función. La primera prueba permite ejecutar las llamadas resueltas a `Math` como primitivas Java del runtime y adaptar `Vec3` y `BlockPos` a valores gestionados por handles del host. Las conversiones con semántica demostrada pueden bajarse directamente a Cranelift. Esta selección evita exigir un GC completo para un caso de alcance acotado.

El ABI inicial usa argumentos y resultados con representación C y versión explícita. Los módulos reciben una tabla de importaciones y devuelven un estado y un valor. Los handles incluyen tipo, vida útil y ámbito. Se validan también las capacidades que requiere el módulo. Las llamadas a Minecraft se enlazarán a adaptadores de Pumpkin comprobados por versión. El formato de caché incluirá hash del módulo, identidad del compilador, ABI, CPU y entradas completas de análisis y transformación. [Diseño del runtime y caché](ARCHITECTURE.md)

## Referencia diferencial ejecutada

Ejecuté el método del JAR original con Temurin 25.0.5+8 y las clases vanilla de Minecraft 26.3. La salida fue:

| Entrada X, Y, Z | Resultado |
| --- | --- |
| `12.5, 49, 12.5` | `12, 49, 12` |
| `-0.01, -2.5, 44.999` | `-1, -2, 44` |
| `-0.0, 2.5, 0.0` | `0, 3, 0` |
| `1.9, 2147483648.0, -1.01` | `1, -2147483648, -2` |
| `NaN, NaN, NaN` | `0, 0, 0` |
| `+∞, +∞, -∞` | `2147483647, -1, -2147483648` |
| `-∞, -∞, +∞` | `-2147483648, 0, 2147483647` |
| Referencia nula | `NullPointerException` |

## Prueba Cranelift propuesta

1. Leer `.class` directamente del JAR fijado y seleccionar el método por descriptor. Registrar clase, offsets, hash del bytecode y el conjunto de importaciones. Los valores de prueba se extraen del oráculo Java. La traducción debe funcionar con una variante del método que cambie una constante o una operación reconocida y produzca el resultado correspondiente.
2. Bajar la IR mediante Cranelift, emitir un objeto y enlazarlo como módulo nativo. Exportar una entrada con versión, un manifiesto de importaciones y la función. Ejecutarla desde un binario Rust de prueba en un entorno sin `java` ni `libjvm`. Confirmar por símbolos o mapas de proceso que se cargó el módulo emitido.
3. Comparar valores y excepciones con la tabla anterior, valores vecinos a `±0.5` y enteros de 32 y 64 bits, y un conjunto reproducible de entradas aleatorias. Incluir un método fuera de perfil y bytecode inválido. Ambos deben fallar con causa y ubicación antes de publicar el módulo.
4. Repetir con caché fría y caliente y registrar tiempo, CPU, RSS, tamaño de artefacto y número de compilaciones. Cambiar JAR, ABI y opciones en ensayos separados. Verificar que una entrada dañada o incompleta se descarta y que el módulo anterior continúa disponible hasta publicar un reemplazo válido.
5. Tras la prueba del método, ampliar a un efecto del mod que use un servicio real de Pumpkin y a un plugin Java que coexista con él. La cobertura de un método es un hito del compilador. La compatibilidad del servidor se acepta con los criterios de juego del [roadmap](ROADMAP.md).

El estado actual es de diseño y referencia Java comprobada. Está pendiente la emisión y ejecución del módulo Cranelift.
