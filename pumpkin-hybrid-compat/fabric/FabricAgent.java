package org.pumpkinhybrid.fabric;

import java.lang.instrument.Instrumentation;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.jar.JarFile;

public final class FabricAgent {
    private static Instrumentation instrumentation;

    public static void premain(String arguments, Instrumentation agent) throws Exception {
        try (var files = Files.list(Path.of("fabric-libraries"))) {
            for (Path jar : files.filter(path -> path.toString().endsWith(".jar")).sorted().toList()) {
                agent.appendToSystemClassLoaderSearch(new JarFile(jar.toFile()));
            }
        }
        EmbeddedLauncher.initialize(agent);
        instrumentation = agent;
    }

    public static Instrumentation instrumentation() {
        if (instrumentation == null) {
            throw new IllegalStateException("Start PatchBukkit with -javaagent:fabric-agent.jar");
        }
        return instrumentation;
    }
}
