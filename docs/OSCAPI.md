# VibeCheck OSC API

VibeCheck features an OSC API that listens while the app is enabled. At the moment the API is early in dev and has only two endpoints.
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