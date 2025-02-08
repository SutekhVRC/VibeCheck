# General VibeCheck Issues

# Problems with the app

- App instantly closes/crashes when starting the app.
  - You either have no WebView2 runtime installed OR your WebView2 runtime install is broken. [Install WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section) or if it is showing up in your program list in control panel as installed, try clicking repair on it.

- App fails to bind or seems to not receive any OSC data.
  - Make sure you don't have two instances of VibeCheck open at the same time!
  - You may have multiple OSC apps binding to the same ports. If that is the case and both apps must receive from VRChat you should use VOR to route VRChat data to both apps!

# Connection issues

## Common Lovense Connect Issues
  - Sometimes Lovense API servers are slow or are down, this can cause Lovense Connect to not function correctly. I have added to my fork of the buttplug library to allow Lovense Connect host overriding (Introduced in VibeCheck 2.0). This will bypass the use of relying on the Lovense API.
  - The device running Lovense Connect is on a different subnet than the computer running VibeCheck. Make sure your phone or computer running Lovense Connect are on the same network as the computer running VibeCheck.
  - Lovense Connect Desktop app failing to register the correct interface can be caused by VPN / VM network interfaces and others alike. Remove virtual networks and disable VPN's.

## Common Bluetooth Issues
  - Your Bluetooth adapter may just be really weak (Feel free to ask for Bluetooth adapter recommendations)
  - Make sure your Bluetooth adapter/dongle supports LE
  - If your computer has an onboard bluetooth interface, make sure it is disabled if you are using another bluetooth interface (USB Bluetooth dongle).
  - Windows updates will sometimes re-enable previously disabled devices. So if you disabled your onboard bluetooth before and are having issues where toys will disconnect you may need to re-disable your onboard bluetooth.
  - Windows only supports the use of 1 generic bluetooth interface at a time.

# How to connect

## Bluetooth
  - The bluetooth interface you are using must support LE.
  - I recommend using an adapter that can handle more than one device and has a **strong/long-range** connection.
  - Make sure bluetooth is enabled.
  - If using a bluetooth adapter make sure to disable your onboard bluetooth device in device manager if you have one.
  - Plug in the bluetooth adapter.
  - Start VibeCheck!

# Avatar Configuration

## OSC

  - VibeCheck can be configured so that each feature/motor of a toy is assigned to different OSC addresses.
  - VibeCheck only reads Float parameters.
  - If you add a parameter to an avatar remember to refresh the OSC config. Click the `Refresh OSC` button in the settings menu.
