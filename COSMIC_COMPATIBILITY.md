# COSMIC Desktop Compatibility

## Implementation Change (v1.2.0+)

**As of version 1.2.0, rivalcfg-tray has migrated from libappindicator3 to the tray-icon library.** This change was made to improve compatibility with the COSMIC desktop environment, which has ongoing development and changes to its system tray implementation.

### Why the Change?

The COSMIC desktop environment underwent changes that affected how system tray icons are rendered. The tray-icon library provides:

1. **Better Cross-Platform Support**: Works natively across multiple desktop environments
2. **Modern Implementation**: Uses current system tray protocols
3. **Simplified Dependencies**: Removes the dependency on libayatana-appindicator
4. **Direct Icon Management**: Better control over icon rendering and updates

## Technical Details (Historical - AppIndicator Era)

### What Was Working (Pre-v1.2.0 with AppIndicator)

- ✅ The application runs without errors
- ✅ Battery monitoring and updates work correctly  
- ✅ D-Bus registration is successful (`org.kde.StatusNotifierWatcher`)
- ✅ Icons are generated and converted properly (SVG → PNG)
- ✅ The app registers as `/org/ayatana/NotificationItem/rivalcfg_tray`

### What's Not Working

- ❌ The tray icon is not visible in COSMIC's panel/status-area
- ❌ COSMIC's `cosmic-applet-status-area` doesn't render the icon

## Investigation Results

### Verified System State

```bash
# COSMIC version
$ pacman -Qi cosmic-session
Version: 1:1.0.1-1

# StatusNotifierWatcher service is running
$ busctl --user list | grep StatusNotifierWatcher
org.kde.StatusNotifierWatcher   152699 cosmic-applet-s ...

# rivalcfg-tray is properly registered
$ busctl --user get-property org.kde.StatusNotifierWatcher /StatusNotifierWatcher \
    org.kde.StatusNotifierWatcher RegisteredStatusNotifierItems
as 5 ... ":1.867/org/ayatana/NotificationItem/rivalcfg_tray"

# Icon properties are correctly set
$ busctl --user introspect :1.867 /org/ayatana/NotificationItem/rivalcfg_tray
.IconName          property  s  "rivalcfg-tray-xyz123"
.IconThemePath     property  s  "/run/user/1000/rivalcfg-tray"
.Status            property  s  "Active"
```

### Changes Made (v1.0.2)

The application has been updated to use the proper StatusNotifier icon theme approach:

1. **Icon Theme Path**: Icons are now stored in `$XDG_RUNTIME_DIR/rivalcfg-tray/` instead of `/tmp`
2. **Proper Icon Naming**: Uses `set_icon_theme_path()` with just the icon name (no extension)
3. **Follows StatusNotifier Spec**: Matches the pattern used by other working tray applications (Steam, Sunshine)

Previous approach (didn't work in COSMIC):

```rust
indicator.set_icon("/tmp/rivalcfg-tray-xyz.png");  // Absolute path
```

Current approach (better COSMIC compatibility):

```rust
indicator.set_icon_theme_path("/run/user/1000/rivalcfg-tray");
indicator.set_icon("rivalcfg-tray-xyz");  // Name only, no path/extension
```

## Current Status

**This appears to be a COSMIC desktop bug/limitation**, not an issue with the application itself. The tray icon implementation follows the freedesktop.org StatusNotifier specification correctly.

## Workarounds

### Option 1: Wait for COSMIC Update

The COSMIC desktop team is actively developing the desktop environment. This issue may be resolved in a future update. Track progress:

- <https://github.com/pop-os/cosmic-applets>
- <https://github.com/pop-os/cosmic-panel>

### Option 2: Run Diagnostics Script

A diagnostic script is included to help verify the issue and report it upstream:

```bash
./scripts/cosmic-tray-diagnostics.sh
```

### Option 3: Alternative Desktop Environments

The tray icon works correctly on:

- ✅ GNOME (with AppIndicator extension)
- ✅ KDE Plasma
- ✅ XFCE
- ✅ Cinnamon
- ✅ MATE

### Option 4: Use Systemd Service

Even without the visible icon, you can still interact with the app:

```bash
# Start the service
systemctl --user start rivalcfg-tray.service

# Check if it's running
systemctl --user status rivalcfg-tray.service

# View logs
journalctl --user -u rivalcfg-tray.service -f
```

Access the configuration by running from terminal and using the menu:

```bash
rivalcfg-tray &
# Right-click the (invisible) tray area or check logs for menu interaction
```

### Option 5: Report to COSMIC Team

If you're experiencing this issue, please report it to help the COSMIC team:

1. Run diagnostics: `./scripts/cosmic-tray-diagnostics.sh > cosmic-tray-debug.txt`
2. Create issue at: <https://github.com/pop-os/cosmic-applets/issues>
3. Include:
   - COSMIC version (`pacman -Qi cosmic-session`)
   - Architecture (x86_64)
   - The output from the diagnostics script
   - Note that other AppIndicator apps (Steam, Sunshine) ARE visible

## Temporary Manual Check

To verify the app is working despite the invisible icon:

```bash
# Check if running
pgrep -f rivalcfg-tray

# Check D-Bus registration  
busctl --user call org.kde.StatusNotifierWatcher /StatusNotifierWatcher \
    org.kde.StatusNotifierWatcher.RegisteredStatusNotifierItems

# View generated icons
ls -lh $XDG_RUNTIME_DIR/rivalcfg-tray/

# Check battery status directly
rivalcfg --battery-level
```

## Version History

- **v1.0.0-1.0.1**: Used absolute PNG paths, invisible in COSMIC
- **v1.0.2**: Updated to use icon theme path approach (current version)

## Related Issues

- COSMIC Desktop: v1.0.1-1 (Arch Linux)
- libayatana-appindicator: 0.5.94-1
- May affect other StatusNotifier-based tray applications

## Future Improvements

Potential solutions being considered:

1. Direct COSMIC protocol integration (when documented)
2. Fallback notification system when tray is unavailable
3. Alternative UI presentation for COSMIC users
4. D-Bus monitoring to detect COSMIC-specific requirements

---

**Last Updated**: January 2026  
**Affects**: COSMIC Desktop 1.0.1 on Arch Linux  
**Status**: Under investigation by COSMIC team
