package org.pumpkinhybrid.fabric;

import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.lang.instrument.ClassFileTransformer;
import java.lang.instrument.Instrumentation;
import java.net.JarURLConnection;
import java.nio.file.Path;
import java.security.ProtectionDomain;
import java.util.Arrays;
import java.util.Collection;
import java.util.HashMap;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.jar.JarFile;
import java.util.jar.Manifest;
import net.fabricmc.api.DedicatedServerModInitializer;
import net.fabricmc.api.EnvType;
import net.fabricmc.api.ModInitializer;
import net.fabricmc.loader.api.entrypoint.PreLaunchEntrypoint;
import net.fabricmc.loader.impl.FabricLoaderImpl;
import net.fabricmc.loader.impl.game.GameProvider;
import net.fabricmc.loader.impl.game.patch.GameTransformer;
import net.fabricmc.loader.impl.launch.FabricLauncher;
import net.fabricmc.loader.impl.launch.FabricLauncherBase;
import net.fabricmc.loader.impl.launch.FabricMixinBootstrap;
import net.fabricmc.loader.impl.metadata.BuiltinModMetadata;
import net.fabricmc.loader.impl.transformer.FabricTransformer;
import net.fabricmc.loader.impl.util.Arguments;
import org.objectweb.asm.AnnotationVisitor;
import org.objectweb.asm.ClassReader;
import org.objectweb.asm.ClassVisitor;
import org.objectweb.asm.ClassWriter;
import org.objectweb.asm.MethodVisitor;
import org.objectweb.asm.Opcodes;
import org.spongepowered.asm.mixin.MixinEnvironment;
import org.spongepowered.asm.mixin.transformer.IMixinTransformer;

public final class EmbeddedLauncher extends FabricLauncherBase {
    private static final String MINECRAFT_SERVER = "net/minecraft/server/MinecraftServer";
    private static final String LOCATOR = "net/bytebuddy/dynamic/ClassFileLocator$ForClassLoader";
    private static final ConcurrentHashMap<String, byte[]> DEFINITIONS = new ConcurrentHashMap<>();

    private final Instrumentation agent;
    private final ClassLoader loader = ClassLoader.getSystemClassLoader();
    private static Path directory;
    private static boolean started;

    private EmbeddedLauncher(Instrumentation agent) {
        this.agent = agent;
    }

    public static void initialize(Instrumentation agent) throws Exception {
        var launcher = new EmbeddedLauncher(agent);
        directory = Path.of("").toAbsolutePath().normalize();
        setProperties(new HashMap<>());
        var provider = new PumpkinGame(directory, launcher.minecraftJar());
        var fabric = FabricLoaderImpl.INSTANCE;
        fabric.setGameProvider(provider);
        fabric.load();
        fabric.freeze();
        fabric.loadClassTweakers();
        FabricMixinBootstrap.init(EnvType.SERVER, fabric);
        finishMixinBootstrapping();
        MixinEnvironment.getDefaultEnvironment().setSide(MixinEnvironment.Side.SERVER);
        var transformer = (IMixinTransformer) MixinEnvironment.getCurrentEnvironment().getActiveTransformer();
        if (transformer == null) {
            throw new IllegalStateException("No active Fabric Mixin transformer");
        }
        agent.addTransformer(new ClassFileTransformer() {
            @Override
            public byte[] transform(ClassLoader definingLoader, String name, Class<?> type,
                    ProtectionDomain domain, byte[] bytes) {
                if (definingLoader != launcher.loader || name == null || type != null) {
                    return null;
                }
                try {
                    if (name.equals(LOCATOR)) {
                        return patchClassFileLocator(bytes);
                    }
                    if (!name.startsWith("net/minecraft/") && !name.startsWith("com/mojang/")) {
                        return null;
                    }
                    String binaryName = name.replace('/', '.');
                    byte[] headless = name.equals(MINECRAFT_SERVER) ? replaceServerLoop(bytes) : bytes;
                    byte[] tweaked = FabricTransformer.transform(false, EnvType.SERVER, binaryName, headless);
                    byte[] mixed = transformer.transformClassBytes(binaryName, binaryName, tweaked);
                    if (mixed == bytes) {
                        return null;
                    }
                    byte[] definition = mixed == tweaked ? tweaked : removeMergeMarkers(mixed);
                    DEFINITIONS.put(name + ".class", definition);
                    return definition;
                } catch (Throwable e) {
                    System.err.println("Pumpkin Fabric transformation failed for " + name);
                    e.printStackTrace();
                    // Exceptions thrown by a Java transformer leave the original class eligible to load.
                    return new byte[0];
                }
            }
        });
        fabric.invokeEntrypoints("preLaunch", PreLaunchEntrypoint.class, PreLaunchEntrypoint::onPreLaunch);
        var loaded = fabric.getAllMods().stream().map(mod ->
            mod.getMetadata().getId() + "@" + mod.getMetadata().getVersion().getFriendlyString()
        ).sorted().toList();
        System.out.println("Pumpkin Fabric: resolved " + String.join(", ", loaded));
    }

    public static synchronized void start(Object server) {
        if (started) {
            throw new IllegalStateException("Fabric entrypoints already started");
        }
        if (!DEFINITIONS.containsKey(MINECRAFT_SERVER + ".class")) {
            throw new IllegalStateException("Headless Fabric startup was not installed");
        }
        started = true;
        var fabric = FabricLoaderImpl.INSTANCE;
        fabric.prepareModInit(directory, server);
        fabric.invokeEntrypoints("main", ModInitializer.class, ModInitializer::onInitialize);
        fabric.invokeEntrypoints("server", DedicatedServerModInitializer.class, DedicatedServerModInitializer::onInitializeServer);
        try {
            var startup = Class.forName("net.minecraft.server.MinecraftServer").getDeclaredMethod("runServer");
            startup.setAccessible(true);
            startup.invoke(server);
        } catch (ReflectiveOperationException exception) {
            throw new IllegalStateException("Unable to fire Fabric headless server startup", exception);
        }
        System.out.println("Pumpkin Fabric: server entrypoints invoked");
    }

    public static InputStream classBytesResource(ClassLoader loader, String name) {
        var definition = loader == ClassLoader.getSystemClassLoader() ? DEFINITIONS.get(name) : null;
        return definition == null ? loader.getResourceAsStream(name) : new ByteArrayInputStream(definition);
    }

    private static byte[] patchClassFileLocator(byte[] original) {
        // PatchBukkit redefines the same classes and must keep members added by Fabric.
        var reader = new ClassReader(original);
        var writer = new ClassWriter(reader, 0);
        reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
            @Override
            public MethodVisitor visitMethod(int flags, String name, String desc, String signature, String[] exceptions) {
                var method = super.visitMethod(flags, name, desc, signature, exceptions);
                if (!name.equals("locate")) return method;
                return new MethodVisitor(Opcodes.ASM9, method) {
                    @Override
                    public void visitMethodInsn(int opcode, String owner, String called, String descriptor, boolean itf) {
                        if (opcode == Opcodes.INVOKEVIRTUAL && owner.equals("java/lang/ClassLoader")
                            && called.equals("getResourceAsStream") && descriptor.equals("(Ljava/lang/String;)Ljava/io/InputStream;")) {
                            super.visitMethodInsn(Opcodes.INVOKESTATIC, "org/pumpkinhybrid/fabric/EmbeddedLauncher",
                                "classBytesResource", "(Ljava/lang/ClassLoader;Ljava/lang/String;)Ljava/io/InputStream;", false);
                        } else {
                            super.visitMethodInsn(opcode, owner, called, descriptor, itf);
                        }
                    }
                };
            }
        }, 0);
        return writer.toByteArray();
    }

    private static byte[] replaceServerLoop(byte[] original) {
        // shortcut: retain the startup join point used by 26.3 Mixins. Pumpkin owns the tick loop.
        var reader = new ClassReader(original);
        var writer = new ClassWriter(reader, ClassWriter.COMPUTE_MAXS);
        boolean[] replaced = {false};
        reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
            @Override
            public MethodVisitor visitMethod(int flags, String name, String desc, String signature, String[] exceptions) {
                var method = super.visitMethod(flags, name, desc, signature, exceptions);
                if (!name.equals("runServer") || !desc.equals("()V")) {
                    return method;
                }
                replaced[0] = true;
                method.visitCode();
                method.visitVarInsn(Opcodes.ALOAD, 0);
                method.visitMethodInsn(Opcodes.INVOKEVIRTUAL, MINECRAFT_SERVER, "buildServerStatus",
                    "()Lnet/minecraft/network/protocol/status/ServerStatus;", false);
                method.visitInsn(Opcodes.POP);
                method.visitInsn(Opcodes.RETURN);
                method.visitMaxs(0, 0);
                method.visitEnd();
                return null;
            }
        }, 0);
        if (!replaced[0]) {
            throw new IllegalStateException("Unsupported Paper server startup method");
        }
        return writer.toByteArray();
    }

    private static byte[] removeMergeMarkers(byte[] original) {
        // ByteBuddy validates @Target; MixinMerged has no declared Java annotation targets.
        var reader = new ClassReader(original);
        var writer = new ClassWriter(reader, 0);
        reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
            @Override
            public MethodVisitor visitMethod(int flags, String name, String desc, String signature, String[] exceptions) {
                return new MethodVisitor(Opcodes.ASM9, super.visitMethod(flags, name, desc, signature, exceptions)) {
                    @Override
                    public AnnotationVisitor visitAnnotation(String descriptor, boolean visible) {
                        if (descriptor.equals("Lorg/spongepowered/asm/mixin/transformer/meta/MixinMerged;")) return null;
                        return super.visitAnnotation(descriptor, visible);
                    }
                };
            }
        }, 0);
        return writer.toByteArray();
    }

    private Path minecraftJar() throws Exception {
        var location = loader.getResource("net/minecraft/server/Main.class");
        if (location == null || !(location.openConnection() instanceof JarURLConnection jar)) {
            throw new IllegalStateException("Paper Minecraft jar missing");
        }
        return Path.of(jar.getJarFileURL().toURI());
    }

    @Override public void addToClassPath(Path path, String... prefixes) {
        if (prefixes.length != 0) throw new UnsupportedOperationException("restricted Fabric classpath");
        try {
            agent.appendToSystemClassLoaderSearch(new JarFile(path.toFile()));
        } catch (IOException exception) {
            throw new IllegalStateException("Cannot add Fabric artifact " + path, exception);
        }
    }

    @Override public void setAllowedPrefixes(Path path, String... prefixes) { throw new UnsupportedOperationException(); }
    @Override public void setValidParentClassPath(Collection<Path> paths) { throw new UnsupportedOperationException(); }
    @Override public EnvType getEnvironmentType() { return EnvType.SERVER; }
    @Override public boolean isClassLoaded(String name) {
        return Arrays.stream(agent.getAllLoadedClasses()).anyMatch(type -> type.getName().equals(name));
    }
    @Override public Class<?> loadIntoTarget(String name) throws ClassNotFoundException { return loader.loadClass(name); }
    @Override public InputStream getResourceAsStream(String name) { return loader.getResourceAsStream(name); }
    @Override public ClassLoader getTargetClassLoader() { return loader; }
    @Override public byte[] getClassByteArray(String name, boolean runTransformers) throws IOException {
        try (var stream = getResourceAsStream(name.replace('.', '/') + ".class")) {
            return stream == null ? null : stream.readAllBytes();
        }
    }
    @Override public Manifest getManifest(Path path) {
        try (var jar = new JarFile(path.toFile())) {
            return jar.getManifest();
        } catch (IOException exception) {
            return null;
        }
    }
    @Override public String getEntrypoint() { return "net.minecraft.server.Main"; }
    @Override public List<Path> getClassPath() {
        return Arrays.stream(System.getProperty("java.class.path").split(File.pathSeparator))
            .map(Path::of).toList();
    }

    private record PumpkinGame(Path directory, Path paper) implements GameProvider {
        @Override public String getGameId() { return "minecraft"; }
        @Override public String getGameName() { return "Minecraft"; }
        @Override public String getRawGameVersion() { return "26.3"; }
        @Override public String getNormalizedGameVersion() { return "26.3"; }
        @Override public Collection<BuiltinMod> getBuiltinMods() {
            return List.of(new BuiltinMod(List.of(paper),
                new BuiltinModMetadata.Builder(getGameId(), getNormalizedGameVersion()).setName(getGameName()).build()));
        }
        @Override public String getEntrypoint() { return "net.minecraft.server.Main"; }
        @Override public Path getLaunchDirectory() { return directory; }
        @Override public boolean requiresUrlClassLoader() { return false; }
        @Override public Set<BuiltinTransform> getBuiltinTransforms(String name) { return Set.of(BuiltinTransform.CLASS_TWEAKS); }
        @Override public boolean isEnabled() { return true; }
        @Override public boolean locateGame(FabricLauncher launcher, String[] args) { return true; }
        @Override public void initialize(FabricLauncher launcher) {}
        @Override public GameTransformer getEntrypointTransformer() { throw new UnsupportedOperationException(); }
        @Override public void unlockClassPath(FabricLauncher launcher) {}
        @Override public void launch(ClassLoader loader) { throw new UnsupportedOperationException("Pumpkin owns server startup"); }
        @Override public Arguments getArguments() { return new Arguments(); }
        @Override public String[] getLaunchArguments(boolean sanitize) { return new String[0]; }
        @Override public String getRuntimeNamespace(String fallback) { return "official"; }
        @Override public String getDefaultModDistributionNamespace(String fallback) { return "official"; }
    }
}
