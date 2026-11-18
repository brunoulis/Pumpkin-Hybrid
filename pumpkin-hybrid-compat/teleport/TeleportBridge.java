package org.pumpkinhybrid.teleport;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import java.util.Objects;
import java.util.logging.Level;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import org.bukkit.command.CommandSender;
import org.bukkit.craftbukkit.CraftServer;
import org.bukkit.craftbukkit.entity.CraftPlayer;
import org.bukkit.plugin.java.JavaPlugin;
import org.pumpkinhybrid.fabric.EmbeddedLauncher;
import org.pumpkinhybrid.teleport.agent.TeleportHook;

public final class TeleportBridge extends JavaPlugin {
    private CommandDispatcher<CommandSourceStack> dispatcher;

    @Override
    public void onEnable() {
        TeleportHook.install();
        var server = ((CraftServer) getServer()).getServer();
        EmbeddedLauncher.start(server);
        dispatcher = new CommandDispatcher<>();
        // Paper constructs its own command tree via a different overload and
        // Bukkit later replaces matching names with wrappers.
        var modCommands = new Commands(Commands.CommandSelection.DEDICATED,
            Commands.createValidationContext(server.registryAccess())).getDispatcher();
        for (String name : new String[] {"sethome", "home"}) {
            var modCommand = modCommands.getRoot().getChild(name);
            if (modCommand == null) {
                throw new IllegalStateException("Fabric Mixin did not register " + name);
            }
            dispatcher.getRoot().addChild(modCommand);
            Objects.requireNonNull(getCommand(name), name)
                .setExecutor((sender, command, label, args) -> execute(sender, name, args));
        }
        getLogger().info("Teleport Commands registered by Fabric Mixins.");
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

}
