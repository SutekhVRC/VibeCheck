**README & instructions not finished.**


# VibeCheck

An app to connect bluetooth sex toys to your VRChat avatar using VRChat's OSC implementation.

## Buttplug IO Supported Toys
- [Supported Toys (All)](https://iostindex.com/?filter0ButtplugSupport=4)
- [Supported Toys (Lovense)](https://iostindex.com/?filter0ButtplugSupport=4&filter1Brand=Lovense)

## Dependencies
- [Intiface Desktop](https://intiface.com/desktop/)
- A bluetooth LE adapter. I recommend [this one](https://www.amazon.com/dp/B09DMP6T22?psc=1&ref=ppx_yo2ov_dt_b_product_details)

## TL;DR
- Install Dependencies and have a bluetooth adapter.
- [Download Installer](https://github.com/SutekhVRC/VibeCheck/releases/latest) and Install VibeCheck.
- Setup avatar to send a synced float parameter (or multiple float parameters) to utilize OSC and send input to VibeCheck. You probably want to drive the parameters with [Avatar Dynamics Contacts](https://docs.vrchat.com/docs/contacts) scripts.
- Start VibeCheck
- Turn on toy(s) (If toy is supported it will autoconnect)
- Edit toy configuration in VibeCheck (Auto/Custom)
- A toy feature in the context of VibeCheck is a literal feature of the toy (Vibrators/Rotators/Linears). A toy can have multiple features as well as multiple features of the same feature type. Ex. A Lovense Lush has one vibrator. That is one feature type (Vibrator) and one feature total. But there are other toys that have multiple feature types as well as multiple features of the same feature type (Ex. multiple vibrators or multiple rotators).
- You can set each toy feature type to a toy feature mode (Auto or Custom)
- Toy feature modes: (Auto) One parameter will activate every instance of a toy feature type / (Custom) Allows you to set a parameter per feature in a feature type.
- Save the toy configuration when done.
- Press "Enable" to start receiving input from VRChat
- Note: You can change toy configurations while the app is enabled.

## Install Instructions

### Install Intiface

- Install Intiface Desktop.
- Note: You will not need to ever open Intiface Desktop, only have it installed.

### Install VibeCheck

- Download and install the MSI from the [latest version of VibeCheck](https://github.com/SutekhVRC/VibeCheck/releases/latest).

### Avatar Setup

- Setup your avatar to have a [VRChat Contact Receiver](https://docs.vrchat.com/docs/contacts#vrccontactreceiver) wherever you want the toy(s) to be controlled from.
- Ex. A contact receiver sphere in the hips area would control the toy(s) when a contact sender with a matching collision tag enters the receiver sphere. The contact receiver **MUST** be in proximity mode to function correctly, because VibeCheck expects float input from VRChat. **NOTE:** (Capsule mode does not work with Proximity)
- If you want another person to be able to interact with your receiver make sure they add the matching collision tags to their senders.
- The parameter(s) you enter into the contact receivers are the parameters you will assign to toys in the VibeCheck app.

## VibeCheck App Setup

1. Run VibeCheck.
2. Minimize the IntifaceCLI engine.
3. If VibeCheck is your only OSC app that receives data from VRChat, skip step 4.
4. If you are using multiple OSC apps that **Receive** data from VRChat consider using my OSC router app: [VOR](https://github.com/SutekhVRC/VOR/releases/latest). Then go to the 'Config' tab and setup VibeCheck's OSC bind host/port to listen on.
5. Make sure bluetooth adapter is plugged in/enabled, and turn on your toy(s).
6. Once your toy(s) are connected, configure them to use the parameters you put in the Contacts Receiver scripts. Click on the toy you want to configure, and click 'Edit'. The edit mode will look different depending on the toy being used. A toy's "features" will be organized into "feature types". The feature types supported are: Vibration/Rotation/Linear. There will be a feature mode for each feature type. The two feature modes are Auto and Custom. Auto mode means one parameter will control every feature in that feature type. Custom mode allows you to set unique parameters for each feature within a feature type. You have full control of the parameter address to listen for. So you will have to input the whole parameter address. You will almost always only need to use the avatar parameters address. So inputting your parameter will follow this format: /avatar/parameters/YOUR_CONTACTS_RECEIVER_PARAMETER_HERE. So if I put the parameter name 'vibe' into my contacts receiver on my avatar I would input '/avatar/parameters/vibe' for my parameter in the toy's configuration. Once you are done press 'Save'.
7. Once your toy is configured press 'Enable' to start using VibeCheck with VRChat.
8. Once you are in VRChat you will need to enable OSC in the expressions menu. If you have use OSC before with your avatar, remember to refresh the OSC config for that avatar (In the OSC expressions menu).
9. You should be all set now. Enjoyyyyyy ;}

## General

VibeCheck functions by receiving OSC input from VRChat. 