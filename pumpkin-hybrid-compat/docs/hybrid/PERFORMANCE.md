# Medición del consumo

El banco de pruebas Linux compara tres instancias del mismo binario optimizado de Pumpkin. La primera ejecuta Pumpkin solo, la segunda añade PatchBukkit y LuckPerms, y la tercera incorpora Fabric Loader y Teleport Commands. Se requiere un entorno con la preparación del [perfil nativo](NATIVE.md) y el [piloto Fabric](TELEPORT.md).

Prepara tres directorios de prueba con la misma copia del mundo, semilla y configuración de red. Las instancias se arrancan secuencialmente y pueden usar el mismo puerto de loopback. Usa los mismos valores de heap Java en los dos perfiles que incorporan PatchBukkit. Deja listas las cachés de Paper y dependencias Java para que la red quede fuera de la medición.

```bash
python3 benchmarks/idle.py /ruta/al/binario/pumpkin \
  --vanilla /ruta/a/vanilla \
  --patchbukkit /ruta/a/patchbukkit \
  --fabric /ruta/a/fabric \
  --logs /ruta/nueva/a/logs > idle-results.json
```

Por defecto el programa rota el orden durante tres rondas. En cada instancia espera al cargador correspondiente, da 15 segundos de estabilización y mide 30 segundos con cero jugadores. La salida JSON contiene el tiempo desde el lanzamiento hasta el mensaje de disponibilidad, la mediana de RSS durante la ventana, el máximo de RSS desde el arranque y el CPU como porcentaje de un núcleo lógico. Los valores de P50, P95 y P99 provienen de los últimos 100 ticks registrados por el propio motor al acabar la ventana. La JVM embebida forma parte del proceso medido. El arranque medido parte del binario de Pumpkin y excluye la comprobación previa del lanzador.

Esta comparación establece el coste fijo del puente. Para medir el coste por jugador y por mod hace falta repetir con la misma carga, movimientos y acciones en los tres escenarios. Todavía faltan esas mediciones y la comparación con un servidor JVM equivalente.
