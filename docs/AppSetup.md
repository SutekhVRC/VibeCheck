# VibeCheck App Setup

1. Run VibeCheck.
2. If VibeCheck is your only OSC app that receives data from VRChat, skip step 3.
3. If you are using multiple OSC apps that **Receive** data from VRChat consider using my OSC router app: [VOR](https://github.com/SutekhVRC/VOR/releases/latest). Then go to the 'Settings' tab and setup VibeCheck's OSC bind host/port to listen on.
4. Setup connection via [Bluetooth](./Bluetooth.md) or [Lovense Connect](./LovenseConnect.md), and turn on your toy(s).
5. Once your toy(s) are connected, configure them to use the parameters you want them to listen for (Floats only).

![Toy Config](./Toy_config.png)

6. Once your toy is configured/saved press on switch in the bottom left to start using VibeCheck with VRChat.
7. Once you are in VRChat you will need to enable OSC in the expressions menu. If you have used OSC before with your avatar, remember to refresh the OSC config for that avatar (In the OSC expressions menu OR delete the avatar's OSC config file adn re-load the avatar).
8. You should be all set now. Enjoyyyyyy ;}

### Enable/Disable OSC command

- VibeCheck will listen for the boolean parameter 'vibecheck/state' to be true or false. If true it will enable and scan for 10 seconds. If false it will disable the app.

### Toy Settings (Wrench Icon)

- OSC Data: Click the checkbox to enable a float to be sent to VRChat that is the battery life of the toy. You can use this parameter in your avatar's animation controllers. Click the address bar to copy the parameter.