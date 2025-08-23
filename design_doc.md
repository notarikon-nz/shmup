Of course. Based on the provided `design_doc.md` and `steam.md` files, here is a comprehensive Game Design Document (GDD) with an integrated feature checklist.

---

# **Game Design Document: Tidal Pool Cosmos**

**Version:** 1.0
**Date:** 2025
**Project:** Tidal Pool Cosmos
**Lead Designer:** Matt Orsborn (Notarikon)
**Status:** In Development

---

## **1. High Concept**

A microscopic shoot-'em-up where players pilot an evolving microorganism through a living, breathing alien tidal pool ecosystem. Combat is defined by biological and environmental mechanics like fluid dynamics, pH levels, and oxygen stratification, as players battle a cosmic infection to save their world and prevent a galactic catastrophe.

## **2. Core Pillars**

1.  **Biological Warfare:** Every game mechanic is rooted in a real or speculative biological concept (pH, osmosis, symbiosis, evolution).
2.  **Living Environment:** The world is not a static backdrop but a dynamic, flowing, and reactive battlefield that is both an obstacle and a weapon.
3.  **Seamless Scale:** The player's sense of perspective is a core gameplay feature, shifting from microscopic to cellular and back.
4.  **Evolving Protagonist:** The player's growth from a simple bacterium into a unique cosmic guardian is the core progression loop.

## **3. Core Gameplay Loop**

1.  **Navigate** a level using current-based movement.
2.  **Combat** corrupted microorganisms and navigate environmental hazards.
3.  **Collect** genetic material and energy from defeated enemies and the environment.
4.  **Evolve** between or during levels by purchasing new biological upgrades.
5.  **Adapt** strategy to the level's unique tidal, chemical, and seasonal conditions.
6.  **Complete** the level objective (destroy all enemies, defeat a boss, survive, etc.).
7.  **Progress** the story and unlock new zones and abilities.

## **4. The Player Character & Evolution (Power-ups)**

The player starts as a basic, customizable microbe. Progression is achieved by permanently acquiring evolutionary traits.

### **Feature Checklist: Evolution & Abilities**

| Category | Feature | Status | Description |
| :--- | :--- | :--- | :--- |
| **Movement** | Flagella / Cilia | ✅ Defined | Permanent speed and acceleration boost. Improved maneuverability in currents. |
| | Pseudopod Extension | 🔄 To Design | Temporary ability to anchor to surfaces or pull towards objects. |
| **Defense** | Enhanced Cell Wall | ✅ Defined | Provides a regenerating health shield. Reduces environmental damage. |
| | Contractile Vacuole | 🔄 To Design | Active ability to rapidly purge toxins/acid, creating a damaging burst. |
| **Offense** | Mitochondria Overcharge | ✅ Defined | Upgrades primary weapon (e.g., from firing single particles to a stream or spread). |
| | Toxic Vesicles | 🔄 To Design | Fires a slow-moving projectile that creates a temporary damaging pH cloud. |
| **Support** | Photosynthesis | ✅ Defined | Passive energy regeneration when in well-lit areas or near the surface. |
| | Chemosynthesis | 🔄 To Design | Alternative energy regeneration in dark, deep, or chemically-rich zones. |
| **Ultimate** | Symbiotic Merge | ✅ Defined | Temporary fusion with an AI-controlled ally, combining abilities for a duration. |

## **5. World & Level Design**

### **Setting Layers**
-   **Macro:** Alien coastline with crystalline rocks under a binary star system.
-   **Micro:** The tidal pool, divided into distinct biomes: Surface Film, Water Column, Sandy Bottom, Rocky Crevices.
-   **Ultra-Micro:** Interiors of larger infected organisms (e.g., anemone digestive tract, crab bloodstream, octopus neural network).

### **Environmental Mechanics**

#### **Feature Checklist: Environment**

| Mechanic | Feature | Status | Description |
| :--- | :--- | :--- | :--- |
| **Currents** | Drift Mechanics | ✅ Defined | All movement is affected by constant water flow. |
| | Strategic Positioning | ✅ Defined | Using eddies, backflows, and thermal currents for tactical advantage. |
| | Tidal Shifts | ✅ Defined | Mid-level reversal of current direction, changing navigation. |
| | Procedural Currents | ✅ Defined | Levels feature uniquely generated flow patterns. |
| **pH Zones** | Acidic (Red) Zones | ✅ Defined | Damages player over time, dissolves calcium-based enemies. |
| | Alkaline (Blue) Zones | ✅ Defined | Boosts photosynthesis, neutralizes acidic enemies. |
| | Neutral Zones | ✅ Defined | Safe havens for regen, but attract enemy clusters. |
| | Dynamic Chemistry | ✅ Defined | Enemy deaths and player actions alter local pH. |
| **Oxygen** | Oxygen-Rich Surface | ✅ Defined | Fast movement, rapid healing. High enemy presence. |
| | Hypoxic Depths | ✅ Defined | Slower regen, anaerobic enemies are sluggish. |
| | Dead Zones | ✅ Defined | Temporary invincibility to aerobic enemies, but constant health drain. |
| | Bubble Mechanics | ✅ Defined | Race enemies to collect randomly spawning O2 pockets. |
| **Seasons** | Spring Blooms | ✅ Defined | Dense algae forests provide cover but limit visibility. |
| | Summer Heat | ✅ Defined | Evaporation creates concentrated zones of enemies/power-ups. |
| | Storm Seasons | ✅ Defined | Chaotic currents, debris obstacles, powerful temporary upgrades. |
| | Winter Dormancy | ✅ Defined | Slower enemies but scarce resources. |
| **Scale** | Seamless Zoom | ✅ Defined | Transition between macro and micro perspectives. |
| | Relative Physics | ✅ Defined | Currents feel stronger, objects larger when zoomed in. |
| | Size-Based Damage | ✅ Defined | Certain attacks only affect enemies of a specific scale. |

## **6. Enemies & Bosses**

### **Standard Enemy Types**

| Enemy Type | Behavior | Weakness |
| :--- | :--- | :--- |
| **Viral Swarms** | Form geometric patterns. Overwhelm with numbers. | pH shifts, area-of-effect attacks. |
| **Corrupted Algae** | Stationary. Spawn smaller enemies. Create tangling hazards. | Fire, acidic zones. |
| **Parasitic Ciliates** | Fast, aggressive, hunter-type behavior. | Lure into strong currents, alkaline zones. |
| **Infected Rotifers** | Mini-bosses. Use spinning feeding apparatuses as weapons. | Attack from behind, attack the apparatus. |

### **Bosses & Story Enemies**

| Boss | Arena | Key Mechanics |
| :--- | :--- | :--- |
| **Infected Sea Anemone** | Its own digestive tract. | Acidic stomach juices, peristaltic currents. |
| **Corrupted Octopus** | Its neural pathways. | Electrical synapses that stun, maze-like navigation. |
| **Infected Crab** | Its bloodstream. | Immune cell enemies, clotting hazards. |
| **The Spore (Final Boss)** | Reality-warping multi-scale arena. | Shifts scale mid-fight. Attacks exist at micro and macro levels simultaneously. Must use tidal mechanics to redirect its energy. |

## **7. Story & Narrative Arc**

**Act I: The Awakening**
-   **Inciting Incident:** A King Tide brings a crystalline cosmic spore into the tidal pool.
-   **Player Goal:** Survive the initial chaos and discover first evolutionary traits.
-   **Hook:** The ecosystem's symbiotic network is collapsing.

**Act II: The Resistance**
-   **Revelation:** The player is part of an ancient biological defense network.
-   **Player Goal:** Seek out other enhanced microbes, learn the spore's true purpose (to build a transmission array).
-   **Escalation:** Battles move inside larger corrupted organisms.

**Act III: The Convergence**
-   **Climax:** The spore becomes a reality-warping entity. The player must fight across multiple scales.
-   **Resolution:** The player uses tidal forces and gravity to redirect the transmission into a rejection signal.
-   **Outcome:** The pool finds a new equilibrium. The player becomes a permanent guardian.

## **8. Game Modes**

### **Feature Checklist: Modes**

| Mode | Description | Status |
| :--- | :--- | :--- |
| **Campaign** | 30+ handcrafted levels telling the full story. | ✅ Defined |
| **Tidal Rush** | Endless survival mode with shifting tidal conditions. | ✅ Defined |
| **Symbiosis (Co-op)** | 2-4 player co-op with temporary ability merging. | ✅ Defined |
| **Laboratory** | Challenge mode with specific evolutionary constraints. | ✅ Defined |

## **9. Art & Audio Direction**

-   **Visual Aesthetic:** Translucent, organic, bioluminescent. "Blown glass and soap bubbles." Refractive water distortion.
-   **Key Art Features:** Bioluminescent lighting, dense particle effects, seamless scale-shifting, fluid and organic animations.
-   **Audio Direction:** Ethereal, biological, and cosmic soundscapes. Sounds should feel like they are underwater. Music intensifies with combat and scale shifts.

## **10. Technical Specifications**

### **Feature Checklist: Tech**

| Category | Feature | Status |
| :--- | :--- | :--- |
| **Engine** | Bevy (Rust) | ✅ Defined |
| **Physics** | Fluid Dynamics Simulation for currents | ✅ Defined |
| **Simulation** | Real-time pH and Oxygen systems | ✅ Defined |
| **Rendering** | Seamless Scale Transitions (LOD) | ✅ Defined |
| **VFX** | Organic Particles, Bioluminescent Lighting | ✅ Defined |
| **Platforms** | PC (Steam) | ✅ Defined |
| **Performance** | Min/Req Specs Defined | ✅ Defined |

### **System Requirements (Recap)**
-   **MIN:** Win 10 64-bit, i5-8400 / R5 2600, 8GB RAM, GTX 1060 / RX 580, 4GB space
-   **REC:** Win 11 64-bit, i7-10700K / R7 3700X, 16GB RAM, RTX 3070 / RX 6700 XT, 4GB SSD

## **11. Development Roadmap & Checklist**

| Phase | Priority | Features | Status |
| :--- | :--- | :--- | :--- |
| **Pre-Production** | P0 | Core Loop Prototype (Movement + 1 Enemy + 1 Upgrade) | 🔄 In Progress |
| | P0 | Fluid Dynamics & Current System MVP | 🔄 In Progress |
| | P0 | Art Style Bible & Tech Art Tests | ⏳ Pending |
| **Production (Alpha)** | P1 | Core Evolution System (Flagella, Cell Wall, Weapon) | ⏳ Pending |
| | P1 | Core Environmental Systems (pH, O2, Currents) | ⏳ Pending |
| | P1 | 5 Enemy Types + 1 Mini-Boss | ⏳ Pending |
| | P1 | Campaign Acts I & II (10-15 Levels) | ⏳ Pending |
| **Production (Beta)** | P1 | All Evolution Abilities | ⏳ Pending |
| | P1 | All Environmental Systems (Seasons, Scale) | ⏳ Pending |
| | P1 | All Bosses & Campaign Act III | ⏳ Pending |
| | P1 | Co-op Symbiosis Mode | ⏳ Pending |
| **Polishing (Gold)** | P1 | Tidal Rush & Laboratory Modes | ⏳ Pending |
| | P1 | Audio Integration & Polish | ⏳ Pending |
| | P1 | Performance Optimization | ⏳ Pending |
| **Post-Launch** | P2 | Additional Challenge Levels | ⏳ Pending |
| | P2 | New Evolutionary Paths | ⏳ Pending |

---
**Document History:**
-   v1.0: Initial GDD created from design_doc.md and steam.md sources.
---