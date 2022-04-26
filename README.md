**README & instructions not finished.**


# VibeCheck

An app to connect bluetooth sex toys to your VRChat avatar using VRChat's OSC implementation.

## Dependencies
- [Intiface Desktop](https://intiface.com/desktop/)
- A bluetooth adapter. I recommend [this one](https://www.amazon.com/dp/B09DMP6T22?psc=1&ref=ppx_yo2ov_dt_b_product_details)

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
- Press start to start receiving input from VRChat
- Note: You can change toy configurations while the app is started.

## Install Instructions

### Install Intiface

- Install Intiface Desktop. Note: You will not need to ever open Intiface Desktop, only have it installed.

### Install VibeCheck

- Download and install the MSI from the [latest version of VibeCheck](https://github.com/SutekhVRC/VibeCheck/releases/latest)

## Avatar Setup

### General

VibeCheck functions by receiving OSC input from VRChat. 