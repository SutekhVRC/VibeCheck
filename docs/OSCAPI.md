# VibeCheck OSC API

VibeCheck features an OSC API that listens while the app is enabled. At the moment the API is early in dev and has a few endpoints.
To use the API, the parameter on your avatar must be a synced parameter.

Synced parameters include:
- Parameters listed in the VRCExpressionParameters script.
- Contacts and Physbones script parameters.

## Endpoints

### State

Value Type: `Boolean`

Parameter Address:
`vibecheck/api/state`

This endpoint changes the app's OSC state. There are two states: enabled (true) and disabled (false).

### Panic

Value Type: `Boolean` (a float of 0.5 or more also counts as true)

Parameter Address:
`vibecheck/api/panic`

Setting this to true stops every toy immediately, the same as the **Stop all toys** button in the app or its hotkey (Ctrl+Alt+P). Toys stay stopped until you press **Resume** in the app; this endpoint can't resume them, so a stray parameter can't start toys again. After resuming, each toy waits for its next new input instead of jumping back to where it was.

### Anatomy

Value Type: `Boolean`

Parameter Address:
`vibecheck/api/anatomy/<ANATOMY_TYPE>/enabled`

Anatomy address parameter types (Not case-sensitive):
```
Anus
Breasts
Buttocks 
Chest
Clitoris 
Face
Feet
FootL
FootR
HandLeft 
HandRight
Hands
Labia
Mouth
NA
Nipples
Penis
Perineum 
Testicles
Vagina
Vulva
Wrist
```

This endpoint changes the toy's enabled state. The app user can specify the anatomy tag for each toy. When the API endpoint for a specified anatomy tag is hit it will change the all the toy's features to disabled or enabled. There are two states: enabled (true) and disabled (false).