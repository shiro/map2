"""
Example demonstrating the new focus/blur event API for window monitoring.

This shows how to:
1. Listen for focus events when a window gains focus
2. Listen for blur events when a window loses focus
3. Handle window info dictionaries containing class, instance, and name
4. Subscribe and unsubscribe from events
"""
import map2
import threading
import time

# Keep track of currently focused window
current_focused = None

def on_focus(window_info):
    """Called when a window gets focus"""
    global current_focused
    current_focused = window_info
    print(f"✓ FOCUS: {window_info['class']}")
    print(f"  Instance: {window_info['instance']}")
    print(f"  Title: {window_info['name']}")

def on_blur(window_info):
    """Called when a window loses focus (e.g., when switching to desktop)"""
    global current_focused
    current_focused = None
    print(f"✗ BLUR: {window_info['class']}")
    print(f"  Instance: {window_info['instance']}")
    print(f"  Title: {window_info['name']}")

# Create window monitor
window = map2.Window()

# Subscribe to focus events
focus_subscription = window.on("focus", on_focus)
print("Subscribed to focus events")

# Subscribe to blur events  
blur_subscription = window.on("blur", on_blur)
print("Subscribed to blur events")

print("\nMonitoring window events... (Press Ctrl+C to stop)")
print("Try switching between windows to see focus/blur events!\n")

try:
    # Keep the listener running
    map2.wait()
except KeyboardInterrupt:
    print("\n\nShutting down...")
    # Unsubscribe from events (optional - will happen automatically on exit)
    window.off(focus_subscription)
    window.off(blur_subscription)
    print("Unsubscribed from all events")
