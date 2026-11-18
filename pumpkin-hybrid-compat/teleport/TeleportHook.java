package org.pumpkinhybrid.teleport.agent;

import java.util.Set;
import net.bytebuddy.ByteBuddy;
import net.bytebuddy.asm.Advice;
import net.bytebuddy.dynamic.loading.ClassReloadingStrategy;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import org.bukkit.Location;
import org.bukkit.event.player.PlayerTeleportEvent.TeleportCause;
import org.patchbukkit.bridge.BridgeUtils;
import org.pumpkinhybrid.fabric.FabricAgent;
import patchbukkit.bridge.NativeBridgeFfi;
import patchbukkit.entity.TeleportEntityRequest;

import static net.bytebuddy.matcher.ElementMatchers.is;

public final class TeleportHook {
    private static final ThreadLocal<Boolean> ACTIVE = new ThreadLocal<>();
    private static boolean installed;

    public static synchronized void install() {
        if (installed) {
            return;
        }
        try {
            var method = ServerPlayer.class.getDeclaredMethod(
                "teleportTo", ServerLevel.class, double.class, double.class, double.class,
                Set.class, float.class, float.class, boolean.class, TeleportCause.class
            );
            new ByteBuddy().redefine(ServerPlayer.class)
                .visit(Advice.to(RouteTeleport.class).on(is(method)))
                .make()
                .load(ServerPlayer.class.getClassLoader(), ClassReloadingStrategy.of(FabricAgent.instrumentation()));
        } catch (NoSuchMethodException exception) {
            throw new IllegalStateException("Unsupported Paper teleport method", exception);
        }
        installed = true;
    }

    public static void enter() {
        if (isActive()) {
            throw new IllegalStateException("Nested teleport command");
        }
        ACTIVE.set(true);
    }

    public static void leave() {
        ACTIVE.remove();
    }

    public static boolean isActive() {
        return ACTIVE.get() != null;
    }

    public static class RouteTeleport {
        @Advice.OnMethodEnter(skipOn = Advice.OnNonDefaultValue.class)
        public static Boolean route(
            @Advice.This ServerPlayer player,
            @Advice.Argument(0) ServerLevel level,
            @Advice.Argument(1) double x,
            @Advice.Argument(2) double y,
            @Advice.Argument(3) double z,
            @Advice.Argument(4) Set<?> relatives,
            @Advice.Argument(5) float yaw,
            @Advice.Argument(6) float pitch
        ) {
            if (!TeleportHook.isActive()) {
                return null;
            }
            // shortcut: this PatchBukkit callback teleports the player within their current world.
            if (!level.dimension().equals(player.level().dimension()) || !relatives.isEmpty()) {
                throw new IllegalArgumentException("Only absolute teleports within the current world are supported");
            }
            if (!Double.isFinite(x) || !Double.isFinite(y) || !Double.isFinite(z) ||
                !Float.isFinite(yaw) || !Float.isFinite(pitch)) {
                throw new IllegalArgumentException("Invalid teleport destination");
            }
            var destination = new Location(null, x, y, z, yaw, pitch);
            var request = TeleportEntityRequest.newBuilder()
                .setUuid(BridgeUtils.convertUuid(player.getUUID()))
                .setLocation(BridgeUtils.convertLocation(destination))
                .build();
            // The protobuf callback uses Pumpkin's player teleport and sets its pending confirmation.
            if (NativeBridgeFfi.teleportEntity(request) == null) {
                throw new IllegalStateException("Pumpkin refused the teleport");
            }
            return true;
        }

        @Advice.OnMethodExit
        public static void assignResult(@Advice.Enter Boolean forwarded, @Advice.Return(readOnly = false) boolean value) {
            if (forwarded != null) {
                value = forwarded;
            }
        }
    }
}
