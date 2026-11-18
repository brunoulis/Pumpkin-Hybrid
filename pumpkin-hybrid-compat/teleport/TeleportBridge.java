package org.pumpkinhybrid.teleport;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import java.io.IOException;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Path;
import java.util.Objects;
import java.util.logging.Level;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.server.MinecraftServer;
import org.bukkit.command.CommandSender;
import org.bukkit.craftbukkit.CraftServer;
import org.bukkit.craftbukkit.entity.CraftPlayer;
import org.bukkit.plugin.java.JavaPlugin;
import org.pumpkinhybrid.teleport.agent.TeleportHook;

public final class TeleportBridge extends JavaPlugin {
    private final CommandDispatcher<CommandSourceStack> dispatcher = new CommandDispatcher<>();
    private URLClassLoader extensions;

    @Override
    public void onEnable() {
        TeleportHook.install();
        var server = ((CraftServer) getServer()).getServer();
        Path jars = getDataFolder().toPath().resolve("jars");
        try {
            extensions = new URLClassLoader(new URL[] {
                jars.resolve("fabric-loader-0.20.1.jar").toUri().toURL(),
                jars.resolve("teleport_commands-merged-26.3-v1.3.5.jar").toUri().toURL(),
            }, getClass().getClassLoader());
            var entry = extensions.loadClass("dev.mrsnowy.teleport_commands.fabricInit");
            entry.getMethod("onInitialize").invoke(entry.getConstructor().newInstance());

            // shortcut: use the two Mixin targets' lifecycle calls until Fabric can
            // transform PatchBukkit's Minecraft classes before they are loaded.
            var mod = extensions.loadClass("dev.mrsnowy.teleport_commands.TeleportCommands");
            mod.getMethod("initializeMod", MinecraftServer.class).invoke(null, server);
            mod.getMethod("registerCommands", CommandDispatcher.class).invoke(null, dispatcher);
        } catch (IOException | ReflectiveOperationException exception) {
            throw new IllegalStateException("Cannot initialize the original Teleport Commands JAR", exception);
        }

        for (String name : new String[] {"sethome", "home"}) {
            Objects.requireNonNull(getCommand(name), name)
                .setExecutor((sender, command, label, args) -> execute(sender, name, args));
        }
        getLogger().info("Original Teleport Commands initialized.");
    }

    private boolean execute(CommandSender sender, String name, String[] args) {
        if (!(sender instanceof CraftPlayer player)) {
            sender.sendMessage("This command requires a connected player.");
            return true;
        }
        if (!player.hasPermission("pumpkinhybrid.home")) {
            player.sendMessage("Missing permission: pumpkinhybrid.home");
            return true;
        }

        var handle = player.getHandle();
        var location = player.getLocation();
        handle.setPos(location.getX(), location.getY(), location.getZ());
        handle.setYRot(location.getYaw());
        handle.setXRot(location.getPitch());
        String input = args.length == 0 ? name : name + " " + String.join(" ", args);
        TeleportHook.enter();
        try {
            dispatcher.execute(input, handle.createCommandSourceStack());
        } catch (CommandSyntaxException exception) {
            sender.sendMessage(exception.getMessage());
        } catch (RuntimeException exception) {
            getLogger().log(Level.WARNING, "Teleport Commands failed", exception);
            sender.sendMessage("Teleport Commands failed.");
        } finally {
            TeleportHook.leave();
        }
        return true;
    }

    @Override
    public void onDisable() {
        if (extensions != null) {
            try {
                extensions.close();
            } catch (IOException exception) {
                getLogger().log(Level.WARNING, "Cannot close extension JARs", exception);
            }
        }
    }
}
