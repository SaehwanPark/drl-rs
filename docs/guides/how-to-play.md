---
title: "How to Play & Controls"
description: "Comprehensive gameplay manual covering movement, targeting, combat, reloading, inventory, and dungeon navigation."
---

# How to Play & Controls

**drl-rs** is a turn-based tactical roguelike set in the demon-infested corridors of Phobos and beyond. Every turn is discrete: the world advances only when you take an action.

---

## 🕹️ Controls Overview

| Action | Primary Key | Numpad / Vi Keys | Description |
|---|---|---|---|
| **Move / Attack North** | `Up Arrow` / `W` | `8` / `K` | Step north or melee bump enemy |
| **Move / Attack South** | `Down Arrow` / `S` | `2` / `J` | Step south or melee bump enemy |
| **Move / Attack East** | `Right Arrow` / `D` | `6` / `L` | Step east or melee bump enemy |
| **Move / Attack West** | `Left Arrow` / `A` | `4` / `H` | Step west or melee bump enemy |
| **Move Diagonal NW** | `Q` | `7` / `Y` | Step north-west |
| **Move Diagonal NE** | `E` | `9` / `U` | Step north-east |
| **Move Diagonal SW** | `Z` | `1` / `B` | Step south-west |
| **Move Diagonal SE** | — | `3` / `N` | Step south-east |
| **Wait Turn** | `.` / `Space` | `5` | Pass one turn (100 energy units) |
| **Target & Fire** | `F` -> `Enter` | `F` | Target nearest enemy and fire |
| **Aimed Fire** | `Shift` + `F` | — | Precise aimed shot (+3 accuracy, 2× time) |
| **First-Level Chainfire** | `C` | — | Fire the available Chaingun/Minigun burst (3 or 6 projectiles) |
| **Reload Weapon** | `R` | `R` | Reload equipped ranged weapon |
| **Pick Up Item** | `G` / `,` | `G` | Pick up weapon, ammo, or item on ground |
| **Descend Stairs** | `>` / `.` | `>` | Take the exit stairs to next floor |
| **Cancel / Menu** | `Escape` | `Escape` | Cancel current targeting or close menu |

---

## ⚡ Turn Economy & Energy Scheduling

Actions in **drl-rs** consume energy points based on your character's speed and the specific action type:

- **Standard Base Cost**: 100 energy units for normal movement and attacks.
- **Fast Enemies & Speed**: Actors with speed `> 100` act more frequently (e.g., Pinky Demons move at speed 140, gaining extra turns over time).
- **Aimed Fire**: Costs 200 energy units in exchange for a +3 accuracy bonus.
- **Single-Shell Reload**: Shotguns and rocket launchers reload single ammunition rounds per turn, allowing agile tactical interruptions.

> [!TIP]
> Do not reload in the open when multiple fast enemies are closing in. Step behind a doorway or corner to reload safely.

---

## 🎯 Line of Sight, FOV & Fog of War

- **Field of View (FOV)**: Vision radiates outward from your character up to a standard radius of 9 tiles.
- **Raycasting**: Opaque tiles (walls, closed doors) block line of sight.
- **Explored Fog**: Tiles you have previously observed remain visible as dark memory tiles on the HUD minimap, but active monster positions update only within current line of sight.
- **Diagonal Movement**: Walkable diagonal tiles remain accessible even if adjacent cardinal tiles are walls, enabling nimble navigation around tight corners.

---

## 💥 Combat Mechanics

### 1. Melee Bump Combat
Walking directly into an adjacent monster tile initiates a melee attack with your equipped melee weapon or fists. Melee attacks never miss against adjacent stationary targets.

### 2. Ranged Combat & Aiming
Pressing `F` cycles through visible targets in order of proximity. Pressing `Enter` discharges your equipped weapon along a straight projectile line to the target.
- **Hit Profile**: Ranged weapons check distance accuracy drop-off, weapon accuracy modifiers, and cover.
- **Spread & Pellets**: Shotguns fire multiple simultaneous pellets, calculating individual pellet hits and applying radial knockback.
- **Blast Radius**: Rocket launchers, missiles, and BFG volleys create area-of-effect explosive blasts that damage all actors within the radius.

### 3. Knockback & Displacement
Shotguns and explosive weaponry exert kinetic knockback:
- Hits displace the target directly backwards away from the shooter.
- If the knockback trajectory is blocked by a solid wall or another monster, the target stops at the obstacle and suffers collision impact.

---

## 🎒 Inventory & Gear Management

- **Backpack Capacity**: Holds up to 20 items (weapons, armor, ammo packs, medical supplies, devices).
- **Equipment Slots**:
  - **Weapon Slot**: Active equipped firearm or melee weapon.
  - **Armor Slot**: Active equipped protective suit (e.g., Green Armor, Blue Armor, Lava Armor).
- **Ammunition Stacks**: Ammunition of the same type automatically merges into compact stacks up to category caps (200 for 9mm, 50 for shotgun shells, 30 for rockets, 100 for plasma cells).
- **Consumables**: Medpacks restore lost HP; Phase Devices instantly teleport you to a random cleared floor tile away from danger.
