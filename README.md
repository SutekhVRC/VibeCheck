### VibeCheck is in Alpha Testing and will be getting updates frequently.
## VibeCheck is undergoing a rewrite. (Bug fixes / optimizations / more features)
**Please report bugs/issues or feature requests if you have them!**

# VibeCheck

An app to connect bluetooth sex toys to your VRChat avatar using VRChat's OSC implementation.


## Buttplug IO Supported Toys
- [Supported Toys (All)](https://iostindex.com/?filter0ButtplugSupport=4)
- [Supported Toys (Lovense)](https://iostindex.com/?filter0ButtplugSupport=4&filter1Brand=Lovense)

## Common Windows OS Bluetooth radio issues
- [Windows OS only supports the use of 1 Bluetooth radio](https://docs.microsoft.com/en-us/windows-hardware/drivers/bluetooth/bluetooth-faq#how-many-bluetooth-radios-can-windows-support)
- Motherboard integrated bluetooth radios commonly have connection issues without an antenna, as well as can only handle one connected device.
- Motherboard integrated bluetooth radios will interfere with external bluetooth adapters (USB adapters / etc.).
- [Motherboard integrated bluetooth radios interfering](https://kb.plugable.com/bluetooth-adapter/your-computer-has-had-a-different-bluetooth-adapter-previously-or-has-a-built-in-adapter)

## Dependencies
- [Intiface Desktop](https://intiface.com/desktop/)
- A bluetooth LE adapter. I recommend [this one](https://www.amazon.com/dp/B09DMP6T22?psc=1&ref=ppx_yo2ov_dt_b_product_details)

## TL;DR
- Install [Dependencies](https://github.com/SutekhVRC/VibeCheck#dependencies) and have a bluetooth adapter.
- [Download Installer](https://github.com/SutekhVRC/VibeCheck/releases/latest) and Install VibeCheck.
- Setup avatar to send a synced float parameter (or multiple float parameters) to utilize OSC and send input to VibeCheck. You probably want to drive the parameters with [Avatar Dynamics Contacts](https://docs.vrchat.com/docs/contacts) scripts.
- Start VibeCheck
- Turn on toy(s) (If toy is supported it will autoconnect)
- Configure toy(s) in VibeCheck.
- Save the toy configuration when done.
- Press "Enable" to start receiving input from VRChat, (Make sure VRChat has OSC enabled and your avatar OSC file is up to date)
- Note: You can change toy configurations while the app is enabled.

## Install Instructions

### Install Intiface

- Install Intiface Desktop.
- Note: You will not need to ever open Intiface Desktop, only have it installed.

### Install VibeCheck

- Download and install the MSI from the [latest version of VibeCheck](https://github.com/SutekhVRC/VibeCheck/releases/latest).

### Avatar Setup

#### VibeCheck Standard Tags
- Using standard tags allows people to generally be setup for people without having to match tags and re-upload avatars. I recommend unchecking 'Allow Self' to avoid triggering your own toys.
- Standard tag: 'vibecheck_tag'

#### Steps
![Contact Receiver On Hip](./images/Contact_Receiver_Hip.png)
- Setup your avatar to have a [VRChat Contact Receiver](https://docs.vrchat.com/docs/contacts#vrccontactreceiver) wherever you want the toy(s) to be controlled from.
- Ex. A contact receiver sphere in the hips area would control the toy(s) when a contact sender with a matching collision tag enters the receiver sphere. The contact receiver **MUST** be in proximity mode to function correctly, because VibeCheck expects float input from VRChat. **NOTE:** (Capsule mode does not work with Proximity)
- If you want another person to be able to interact with your receiver make sure they add the matching collision tags to their [VRChat Contact Senders](https://docs.vrchat.com/docs/contacts#vrccontactsender).
- The parameter(s) you enter into the contact receivers are the parameters you will assign to toys in the VibeCheck app.

## VibeCheck App Setup

1. Run VibeCheck.
2. If VibeCheck is your only OSC app that receives data from VRChat, skip step 3.
3. If you are using multiple OSC apps that **Receive** data from VRChat consider using my OSC router app: [VOR](https://github.com/SutekhVRC/VOR/releases/latest). Then go to the 'Settings' tab and setup VibeCheck's OSC bind host/port to listen on.
4. Make sure bluetooth adapter is plugged in/enabled, and turn on your toy(s).
5. Once your toy(s) are connected, configure them to use the parameters you put in the Contacts Receiver scripts. Click on the toy you want to configure, and click 'Edit'. The edit mode will look different depending on the toy being used. A toy's "features" will be organized into "feature types". The feature types supported are: Vibration/Rotation/Linear. There will be a feature mode for each feature type. The two feature modes are Auto and Custom. Auto mode means one parameter will control every feature in that feature type. Custom mode allows you to set unique parameters for each feature within a feature type. You have full control of the parameter address to listen for. So you will have to input the whole parameter address. You will almost always only need to use the avatar parameters address. So inputting your parameter will follow this format: /avatar/parameters/YOUR_CONTACTS_RECEIVER_PARAMETER_HERE. So if I put the parameter name 'vtest' into my contacts receiver on my avatar I would input '/avatar/parameters/vtest' for my parameter in the toy's configuration. Once you are done press 'Save'.

![Toy Config](./images/Toy_config.png)

6. Once your toy is configured press 'Enable' to start using VibeCheck with VRChat.
7. Once you are in VRChat you will need to enable OSC in the expressions menu. If you have use OSC before with your avatar, remember to refresh the OSC config for that avatar (In the OSC expressions menu).
8. You should be all set now. Enjoyyyyyy ;}

### Feature Modifiers
- Idle: The idle level of the feature. Idle starts once the feature has been triggered for the first time.
- Minimum: The minimum level the feature is allowed to be active at. Minimum does not influence idle.
- Maximum: The maximum level the feature is allowed to be active at. Maximum does not influence idle.

## General

VibeCheck functions by receiving OSC input from VRChat. 

## Thanks to the people below for testing and suggestions!

- Tinitre
- Buneskapp
- Nitro
- MikuLove
- Googii
- Kali
