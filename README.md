### Interactive app for displaying VDB files with various settings using **Ray-Marching** algorithm.


# Requirement
- **Rust**

# Deploy
```
git clone https://github.com//Ibukhvalov/render
cd render
cargo run --release ./data/vdbAssets/wdas_cloud_sixteenth.vdb
```

# Interaction
## Camera movement
- **WASD** - front/back/left/right
- **QE** - up/down
- **Arrows** - direction

## Settings
### Step size
Adjust the size of probing inside a volume. Be aware about high performance effect

| Long                                 | Short                                |
| ------------------------------------ | ------------------------------------ |
| <img width="248" alt="Pasted image 20250530005821" src="https://github.com/user-attachments/assets/497bc9dd-124c-4e81-a751-b253ce6eba57" /> | <img width="242" alt="Pasted image 20250530005831" src="https://github.com/user-attachments/assets/884fb2ba-9042-4c79-a733-00bfc1b72837" /> |


### g (phase function)
Define direction of distribution of light inside volume.
Default function is **Henyey-Greenstein**
$$
P(\cos \theta) = \frac{1}{4\pi} \cdot \frac{1 - g^2}{(1 + g^2 - 2g\cos\theta)^{3/2}}
$$

- $\theta$ — the angle between the incoming and outgoing light directions  
- $g$ — the asymmetry parameter ($-1 \leq g \leq 1$)

| -0.2                                 | 0                                    | 0.6                                  |
| ------------------------------------ | ------------------------------------ | ------------------------------------ |
| <img width="153" alt="Pasted image 20250530003521" src="https://github.com/user-attachments/assets/c3fea58b-505c-43f1-8fca-bec9b493d36c" /> | <img width="150" alt="Pasted image 20250530003541" src="https://github.com/user-attachments/assets/ebc8d913-e6fc-491a-966a-81607bcd00b9" /> | <img width="197" alt="Pasted image 20250530003526" src="https://github.com/user-attachments/assets/b42091d2-eba2-4760-b328-2e17f4ecde18" /> |


### Scattering
Controls how much light is scattered inside the volume. Higher values make the volume look softer and more glowing.

| Low                                  | High                                 |
| ------------------------------------ | ------------------------------------ |
| <img width="233" alt="Pasted image 20250530004100" src="https://github.com/user-attachments/assets/440e3605-f633-4d12-ac08-480e8f70200c" /> | <img width="231" alt="Pasted image 20250530003856" src="https://github.com/user-attachments/assets/86616b12-45f2-4ef0-82b2-53f0a5a2ab3e" /> |


## Absorption
Determines how much light is absorbed as it passes through the volume. Higher values make the volume look denser and darker.
| Low                                  | High                                 |
| ------------------------------------ | ------------------------------------ |
| <img width="233" alt="Pasted image 20250530004100" src="https://github.com/user-attachments/assets/9896a211-1691-406a-9609-70547f633ee3" /> | <img width="241" alt="Pasted image 20250530004106" src="https://github.com/user-attachments/assets/a69d4bb4-5b76-46ec-a2e4-cdcf0711ba09" /> |

## Lightness
Light color scalar

## Colors
Convenient color pickers for colors adjustments
### Background
| <img width="218" alt="bg_black" src="https://github.com/user-attachments/assets/df1bf67c-ff43-47d3-8651-5d00c68f92f1" /> | <img width="247" alt="Pasted image 20250530010113" src="https://github.com/user-attachments/assets/50ec6e27-b844-4ee0-9025-8b2176ce371b" /> |
| ------------------------------------ | ------------------------------------ |

### Light

| <img width="212" alt="light_red" src="https://github.com/user-attachments/assets/adf90213-3beb-4dee-83bf-80e3619bfdcc" /> | <img width="252" alt="light_blue" src="https://github.com/user-attachments/assets/f56c22ce-c6e4-4df8-886c-b7cd448b9e1e" /> |
| ------------------------------------ | ------------------------------------ |
