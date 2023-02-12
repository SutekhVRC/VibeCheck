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

## Lovense Connect
  - Download the [Lovense Connect](https://www.lovense.com/cam-model/guides/pc-dongle) app (Lovense Connect is a different app than Lovense Remote) either on your smartphone or on PC. If using the PC version you must have the Lovense adapter.
  - Open the Lovense Connect app and connect your toys to it.
  - Start VibeCheck
  - VibeCheck will ask the Lovense Connect API for your devices. I recommend using the `Lovense Connect Override` option to tell VibeCheck to connect to the device directly instead of using the Lovense Connect API.

## Bluetooth
  - The bluetooth interface you are using must support LE.
  - I recommend using an adapter that can handle more than one device and has a **strong/long-range** connection.
  - Make sure bluetooth is enabled.
  - If using a bluetooth adapter make sure to disable your onboard bluetooth device in device manager if you have one.
  - Plug in the bluetooth adapter.
  - Start VibeCheck!

# Pros, Cons, Lovense Connect or Bluetooth?

## Bluetooth Mode

### Pros
    - Generally has a faster response time
    - Supports many different toys.

### Cons
    - Can be less stable than Lovense Connect if the Bluetooth LE adapter in use isn't very strong.

## Lovense Connect Mode

### Pros
    - Can be a more stable alternative if you dont have a very good bluetooth LE adapter
    - Don't need to buy any adapters if you have a smart phone and WiFi.
    - Can use Lovense Connect on phone to take advantage of your WiFi's range.

### Cons
    - Can only be used with Lovense Toys.

# Avatar Configuration

## OSC

  - VibeCheck can be configured so that each feature/motor of a toy is assigned to different OSC addresses.
  - VibeCheck only reads Float parameters.
  - If you add a parameter to an avatar remember to refresh the OSC config. I do this by deleting the OSC configuration files for my avatars and then changing out and back in to my avatar. The button in game never works for me.
