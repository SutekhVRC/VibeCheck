# Lovense Connect

The Lovense Connect API can be finnicky at times. I recommend using VibeCheck's `Lovense Connect Override` option in the settings menu.

## How To: Lovense Connect Override

1. Enable by clicking the checkbox.
2. If using the Lovense Connect app for PC you can just leave the default IP address `127.0.0.1`.
3. If using a smartphone with Lovense Connect, the smartphone must be on the same LAN as your PC running VibeCheck. Then input the smartphone's LAN IP address.

## Pros

- Can be a more stable alternative if you dont have a very good bluetooth LE adapter
- Don't need to buy any adapters if you have a smart phone and WiFi.
- Can use Lovense Connect on phone to take advantage of your WiFi's range.

## Cons

- Can only be used with Lovense Toys.
- Slower response time than Bluetooth LE.

# Requirements

- [Lovense USB Bluetooth Adapter](https://www.lovense.com/bluetooth-adapter)
- Download Lovense Connect for phone or desktop.
- Connect toys to Lovense Connect
- Start VibeCheck (Sometimes you may need to wait like 10 - 20 seconds if the Lovense API is being slow)

# Common Lovense Connect Issues

- I recommend using VibeCheck's Lovense Connect Host Override setting. Enter the LAN IP address for the device running Lovense Connect. If you are using desktop Lovense Connect you can use 127.0.0.1 as the IP address.
- Start Lovense Connect **before** starting VibeCheck (If you are using VibeCheck 0.1.x).
- Sometimes Lovense API servers are slow or are down, this can cause Lovense Connect to not function correctly. I have added to my fork of the buttplug library to allow Lovense Connect host overriding. This will bypass relying on the Lovense API to discover LAN devices.
- The device running Lovense Connect is on a different subnet than the computer running VibeCheck. Make sure your phone or computer running Lovense Connect are on the same network as the computer running VibeCheck.
- Lovense Connect Desktop app failing to register the correct interface can be caused by VPN / VM network interfaces and others alike. Remove virtual networks and disable VPN's.
