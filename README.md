# VibeCheck
[![GitHub all releases](https://img.shields.io/github/downloads/SutekhVRC/VibeCheck/total?color=pink&logoColor=pink&style=for-the-badge)](https://github.com/SutekhVRC/VibeCheck/releases/latest)
[![Discord](https://img.shields.io/discord/1031167339246407721?color=pink&label=Discord&logo=discord&logoColor=pink&style=for-the-badge)](https://discord.gg/g6kUFtMtpw)

An app to connect bluetooth sex toys to your VRChat avatar using VRChat's OSC implementation.


[Supported Toys](https://iostindex.com/?filter0ButtplugSupport=4)

### VibeCheck 0.2 is in Beta Testing and will be getting updates frequently.

**Please report bugs/issues or feature requests if you have them!**

## Getting Started

- Get requirements for [Bluetooth](./docs/Bluetooth.md#requirements) or [Lovense Connect](./docs/LovenseConnect.md#requirements)
- [Download Installer](https://github.com/SutekhVRC/VibeCheck/releases/latest) and Install VibeCheck.
- Setup avatar your own custom way or use a [VibeCheck prefab]()
- Start VibeCheck and turn on toy(s).
- Search for toy(s) and configure them.
- Join the discord for help!

# VibeCheck App Setup

1. Run VibeCheck.
2. If VibeCheck is your only OSC app that receives data from VRChat, skip step 3.
3. If you are using multiple OSC apps that **Receive** data from VRChat consider using my OSC router app: [VOR](https://github.com/SutekhVRC/VOR/releases/latest). Then go to the 'Settings' tab and setup VibeCheck's OSC bind host/port to listen on.
4. Setup connection via [Bluetooth](./docs/Bluetooth.md) or [Lovense Connect](./docs/LovenseConnect.md), and turn on your toy(s).
5. Once your toy(s) are connected, configure them to use the parameters you want them to listen for (Floats only).

![Toy Config](./docs/Toy_config.png)

6. Once your toy is configured/saved press on switch in the bottom left to start using VibeCheck with VRChat.
7. Once you are in VRChat you will need to enable OSC in the expressions menu. If you have used OSC before with your avatar, remember to refresh the OSC config for that avatar (In the OSC expressions menu OR delete the avatar's OSC config file adn re-load the avatar).
8. You should be all set now. Enjoyyyyyy ;}

### Enable/Disable OSC command

- VibeCheck will listen for the boolean parameter 'vibecheck/state' to be true or false. If true it will enable and scan for 10 seconds. If false it will disable the app.

### Toy Settings (Wrench Icon)

- OSC Data: Click the checkbox to enable a float to be sent to VRChat that is the battery life of the toy. You can use this parameter in your avatar's animation controllers. Click the address bar to copy the parameter.

# Credits

- TutiDore: Frontend developer.
- [DigiGhost](https://twitter.com/digi_ghost): Commissioned Artist (Icons and Banners).
- [Tini](https://vrchat.com/home/user/usr_7d526959-f3ab-4226-aa82-dba613df998e): Helped with frontend UI mockups and testing.

Thanks to the people below for testing and suggestions!

- Googii
- Buneskapp
- MikuLove
- Kali
- Nitro
