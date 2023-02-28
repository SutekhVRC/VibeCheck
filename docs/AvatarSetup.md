# Avatar Setup

VibeCheck isn't attached to any unity prefabs/setups. You can use VibeCheck with any animator logic you want to make, as long as you are sending a float parameter to VibeCheck. This could be a single VRChat Contact Receiver script, or a complex feature set where an animator sends different float values to different motors on a toy. This can allow fun interactions with your avatar like slapping a butt or pulling a chain etc.

## OSC

- VibeCheck can be configured so that each feature/motor of a toy is assigned to different OSC addresses.
- VibeCheck only reads Float parameters.
- If you add a parameter to an avatar remember to refresh the OSC config. I do this by deleting the OSC configuration files for my avatars and then changing out and back in to my avatar. The button in game never works for me.

# Premade Prefabs

**When adding a prefab remember to refresh your avatar's OSC config!**

I have created some prefabs to make setup a bit more simple, if you don't want to setup your avatar completely yourself.

- [Simple Contact Receiver](./AvatarSetup.md#simple-contact-receiver-simple)
- [Constant system](./AvatarSetup.md#constant-to-float-conversion-constant)
- [Contact rate system](./AvatarSetup.md#contact-rate-system-rate)

## Custom setup

If you want to make your own system for VibeCheck the only requirement is just that VibeCheck receives only float parameters for the toy motor parameters.

**VibeCheck parameters**

- `vibecheck/state`: (Boolean sent to VibeCheck) Enables VibeCheck and scans for 10 seconds when true and disables VibeCheck when false.

- `{toy_name}/{toy duplicate id}/battery`: (Float sent to VRChat) The battery percentage of the toy. (Will sometimes say 0 even when charged, if toy is not fully initialized. Just wait 60 seconds for the next toy update).

## Simple Contact Receiver (Simple)

The simple system is good for a quick and easy setup and uses just one contact receiver.

1. Import [VibeCheck Prefab Package](https://github.com/SutekhVRC/VibeCheck/raw/main/UnityPrefabs/VibeCheck_Prefabs.unitypackage).
2. In the "Simple" folder there is a folder for the default Simple prefab and folder for the TPS prefab.
3. Drag the prefab onto your avatar's armature wherever you want it to be.
4. Scale and move the prefab as needed. Keep in mind, the center of the sphere will be the maximum float value / motor speed.
5. If you are setting it up for a penetrator, you will want to put the center of the Contact sphere at the base of the penetrator.
6. If you are setting it up for an orifice, adjust it so the bottom of the sphere is slightly below or even with the orifice opening.
7. Add/Remove tags you want or don't want in the contact receiver.
8. Check "Allow Self" if you want to be able to activate the contact yourself.
9. Put the `vibecheck/simple/out` parameter in the VibeCheck application.

## Constant system (Constant)

The constant system is great for when you just want to activate a toy to whatever level you have set in your expressions menu when someone enters the contact receiver. This system is great for things like butt slaps and boops etc. This system has adjustments for motor speed, cooldown speed, and "added active length time". 

1. Import [VibeCheck Prefab Package](https://github.com/SutekhVRC/VibeCheck/raw/main/UnityPrefabs/VibeCheck_Prefabs.unitypackage).
2. Import [AV3 Manager by VRLabs](https://github.com/VRLabs/Avatars-3.0-Manager/releases/latest).
3. Place `VibeCheck_Constant` prefab on your avatar wherever you want it.
4. Scale and position the prefab contact receiver how you want it.
5. Using AV3 Manager, merge the **"WD OFF"** or **"WD ON"** `VibeCheck_Constant_FX` controller with your avatar's FX controller.
6. Using AV3 Manager, copy the parameters from `VibeCheck_Constant_Parameters` into your avatar's parameters.
7. Make a new sub menu in your expressions menu on your avatar. Then input the `VibeCheck_Constant_Menu` into it.
8. Put the `vibecheck/constant/out` parameter in the VibeCheck application.
9. **IMPORTANT:** For the Constant system to function correctly you will need to disable the "Smoothing" option for every feature you want to work.
10. Once you have uploaded the avatar make sure to click the "Refresh OSC" button in VibeCheck (In settings). Then change out and back in to your avatar.

## Contact rate system (Rate) (Incomplete)

The contact rate system is a system I've been working on to allow a rate-like interaction with contacts. The faster you move a sender through the receivers the faster the motor speed will go. This system has an adjustment for the passive motor speed decrease period.

1. Import [VibeCheck Prefab Package]().
2. Import [AV3 Manager by VRLabs](https://github.com/VRLabs/Avatars-3.0-Manager/releases/latest).
3. Place the `VibeCheck_Rate_Orifice` or `VibeCheck_Rate_Penetrator` prefab where you want it.
5. Scale and position the prefab how you want it.
6. Using AV3 Manager, merge `VibeCheck_Rate_FX` controller with your avatar's FX controller.
7. Using AV3 Manager, copy the parameters from `VibeCheck_Rate_Parameters` into your avatar's parameters.
8. Make a new sub menu in your expressions menu on your avatar. Then put the `VibeCheck_Rate_Menu` into it.
9. Put the `vibecheck/rate/out` parameter in the VibeCheck application.

# TPS & VibeCheck

Using TPS with VibeCheck is as easy as setting the float parameters driven by TPS in your toy OSC parameters in the VibeCheck app.

## TPS **Orifice** with VibeCheck

Use the 'TPS_Internal/Orf/0/Depth_In' orifice parameter created by the TPS wizard.

![TPS Orifice](./VC_TPS_Orifice_Parameter.png)

## TPS **Penetrator** with VibeCheck

Use the 'TPS_Internal/Pen/0/RootRoot' penetrator parameter created by the TPS wizard.

![TPS Penetrator](./VC_TPS_Penetrator_Parameter.png)
