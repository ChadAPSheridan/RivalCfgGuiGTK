#!/usr/bin/env bash
# COSMIC Tray Icon Diagnostics Script
# This script helps diagnose system tray icon issues on COSMIC desktop

set -euo pipefail

echo "========================================="
echo "COSMIC Tray Icon Diagnostics"
echo "========================================="
echo ""

echo "1. Desktop Environment:"
echo "   XDG_CURRENT_DESKTOP: ${XDG_CURRENT_DESKTOP:-Not set}"
echo "   XDG_SESSION_TYPE: ${XDG_SESSION_TYPE:-Not set}"
echo ""

echo "2. COSMIC Version:"
pacman -Qi cosmic-session 2>/dev/null | grep -E "^(Name|Version)" || echo "   cosmic-session not found"
echo ""

echo "3. libayatana-appindicator Version:"
pacman -Qi libayatana-appindicator 2>/dev/null | grep -E "^(Name|Version)" || echo "   libayatana-appindicator not found"
echo ""

echo "4. StatusNotifierWatcher D-Bus Service:"
if busctl --user list | grep -q "org.kde.StatusNotifierWatcher"; then
    echo "   ✓ StatusNotifierWatcher is available"
    busctl --user list | grep "org.kde.StatusNotifierWatcher"
else
    echo "   ✗ StatusNotifierWatcher NOT available"
fi
echo ""

echo "5. Registered Status Notifier Items:"
if items=$(busctl --user get-property org.kde.StatusNotifierWatcher /StatusNotifierWatcher org.kde.StatusNotifierWatcher RegisteredStatusNotifierItems 2>/dev/null); then
    echo "   $items"
else
    echo "   Failed to query registered items"
fi
echo ""

echo "6. Running COSMIC Applets:"
ps aux | grep "[c]osmic-applet" | awk '{print "   " $11}'
echo ""

echo "7. rivalcfg-tray Status:"
if pgrep -x rivalcfg-tray > /dev/null; then
    echo "   ✓ rivalcfg-tray is running (PID: $(pgrep -x rivalcfg-tray))"
    if busctl --user list | grep -q rivalcfg-tray; then
        echo "   ✓ rivalcfg-tray is registered on D-Bus"
        busctl --user list | grep rivalcfg-tray
    else
        echo "   ✗ rivalcfg-tray NOT found on D-Bus"
    fi
else
    echo "   ✗ rivalcfg-tray is NOT running"
fi
echo ""

echo "8. Panel Configuration:"
if [ -f ~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings ]; then
    echo "   Panel plugins configured:"
    cat ~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings | grep -o '"[^"]*"' | sed 's/^/   - /'
else
    echo "   Panel configuration not found"
fi
echo ""

echo "9. Temp Icon Files:"
ls -lh /tmp/rivalcfg*.{svg,png} 2>/dev/null | sed 's/^/   /' || echo "   No temp icon files found"
echo ""

echo "========================================="
echo "Diagnostics Complete"
echo "========================================="
echo ""
echo "If the icon is not visible despite being registered,"
echo "this is likely a COSMIC desktop rendering issue."
echo "Please report to: https://github.com/pop-os/cosmic-applets/issues"
