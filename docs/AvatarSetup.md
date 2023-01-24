# Avatar Setup (Incomplete)

VibeCheck isn't attached to any unity prefabs/setups. You can use VibeCheck with any animator logic you want to make, as long as you are sending a float parameter to VibeCheck. This could be a single VRChat Contact Receiver script, or a complex feature set where an animator sends different float values to different motors on a toy. This can allow fun interactions with your avatar like slapping a butt or pulling a chain etc.

# Premade Prefabs

I have created some prefabs to make setup a bit more simple, if you don't want to setup your avatar completely yourself.
- [Simple Contact Receiver]()
- [Constant to float conversion]()
- [Contact rate system]()

## Simple Contact Receiver

### Setup

1. Import [VibeCheck Prefab Package]().
2. In the "Simple" folder there is a folder for the default Simple prefab and folder for the TPS prefab.
3. Drag the prefab you want to use onto your Hips and unpack it completely.
4. If you are setting it up for a penetrator, you will want to put the center of the Contact sphere at the base of the penetrator. If you are setting if up for an oriface just adjust it inside the orifice where you want it.

## Constant to float conversion
1. Import [VibeCheck Prefab Package]().
2. 

## Contact rate system
1. Import [VibeCheck Prefab Package]().

# TPS & VibeCheck

Using TPS with VibeCheck is as easy as setting the float parameters driven by TPS in your toy OSC parameters in the VibeCheck app.

## TPS **Orifice** with VibeCheck

Use the 'TPS_Internal/Orf/0/Depth_In' orifice parameter created by the TPS wizard.

![TPS Orifice](./VC_TPS_Orifice_Parameter.png)

## TPS **Penetrator** with VibeCheck

Use the 'TPS_Internal/Pen/0/RootRoot' penetrator parameter created by the TPS wizard.

![TPS Penetrator](./VC_TPS_Penetrator_Parameter.png)
