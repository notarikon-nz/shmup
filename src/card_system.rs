// src/card_system.rs
use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::wave_systems::*;
use crate::enemy_types::*;
use crate::despawn::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ===== CONSTANTS =====
const STAGE_WAVE_COUNT: u32 = 5;
const PERMANENT_CARD_DROP_CHANCE: f32 = 0.15; // 15% chance per stage
const TEMPORAL_CARD_DROP_CHANCE: f32 = 0.25; // 25% chance per stage
const CARD_BOX_SPAWN_CHANCE: f32 = 0.08; // 8% chance for green box per stage

// ===== CARD SYSTEM COMPONENTS =====

#[derive(Component)]
pub struct CardPickup {
    pub card_type: CardType,
    pub bob_timer: f32,
    pub collection_range: f32,
}

#[derive(Component)]
pub struct GreenCardBox {
    pub stage_number: u32,
    pub guaranteed_card: bool,
}

#[derive(Component)]
pub struct CardEffectActive {
    pub card: TemporalCard,
    pub remaining_time: f32,
}

// ===== CARD DEFINITIONS =====

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CardType {
    Permanent(PermanentCard),
    Temporal(TemporalCard),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PermanentCard {
    CellDivision,              // Extra life
    PowerfulDrones,            // Support drone boost
    CardDropRateIncrease,      // Better card drops
    LuckyShield,               // Once per stage shield
    SupportDroneRight,         // Right drone
    SupportDroneLeft,          // Left drone
    ReducedMissileReload,      // Faster missiles
    InitialFireRateUp,         // Start with metabolic upgrade
    IncreasedMissileDamage,    // Missile damage boost
    UpgradePricesReduced,      // 10% ATP discount
    HealthRegeneration,        // 1% health per 1.5 seconds
    EmergencySporeToBegin,     // Start with extra spore
    IncreasedShipSpeed,        // Movement speed boost
    ScoreBonus,                // Final card - 10% score bonus
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TemporalCard {
    CardDropRateIncrease { duration: f32 },
    IncreasedMagnetPower { duration: f32 },
    ExtraATP { duration: f32 },
    MaximumFireRate { duration: f32 },
    RandomPowerUp,
}

#[derive(Resource, Default)]
pub struct CardCollection {
    pub permanent_cards: HashSet<PermanentCard>,
    pub active_temporal_cards: Vec<(TemporalCard, f32)>,
    pub cards_found_this_run: u32,
    pub total_cards_found: u32,
}

#[derive(Resource)]
pub struct StageProgress {
    pub current_stage: u32,
    pub waves_in_current_stage: u32,
    pub enemies_destroyed_this_stage: u32,
    pub total_enemies_this_stage: u32,
    pub damage_taken_this_stage: bool,
    pub infrastructure_destroyed: u32,
    pub infrastructure_total: u32,
}

impl Default for StageProgress {
    fn default() -> Self {
        Self {
            current_stage: 1,
            waves_in_current_stage: 0,
            enemies_destroyed_this_stage: 0,
            total_enemies_this_stage: 0,
            damage_taken_this_stage: false,
            infrastructure_destroyed: 0,
            infrastructure_total: 0,
        }
    }
}

// ===== CARD IMPLEMENTATIONS =====

impl PermanentCard {
    pub fn get_display_info(&self) -> (&'static str, &'static str) {
        match self {
            PermanentCard::CellDivision => 
                ("Cell Division", "Player starts with one extra life"),
            PermanentCard::PowerfulDrones => 
                ("Symbiotic Enhancement", "Support organisms get 25% firepower boost"),
            PermanentCard::CardDropRateIncrease => 
                ("Genetic Diversity", "Higher chances of finding enhancement cards"),
            PermanentCard::LuckyShield => 
                ("Emergency Membrane", "Once per stage, escape losing a life"),
            PermanentCard::SupportDroneRight => 
                ("Right Symbiont", "Adds support organism on your right side"),
            PermanentCard::SupportDroneLeft => 
                ("Left Symbiont", "Adds support organism on your left side"),
            PermanentCard::ReducedMissileReload => 
                ("Rapid Reproduction", "Missiles deploy at more frequent intervals"),
            PermanentCard::InitialFireRateUp => 
                ("Metabolic Boost", "Start with enhanced cellular metabolism"),
            PermanentCard::IncreasedMissileDamage => 
                ("Toxic Payload", "Your guided missiles deal greater damage"),
            PermanentCard::UpgradePricesReduced => 
                ("Efficient Evolution", "10% ATP discount on all upgrades"),
            PermanentCard::HealthRegeneration => 
                ("Cellular Repair", "Health gradually restores at 1% per 1.5 seconds"),
            PermanentCard::EmergencySporeToBegin => 
                ("Reproductive Readiness", "Start with an extra emergency spore"),
            PermanentCard::IncreasedShipSpeed => 
                ("Enhanced Motility", "Increased movement speed for better evasion"),
            PermanentCard::ScoreBonus => 
                ("Full Genome", "10% score bonus when all cards collected"),
        }
    }

    pub fn is_prerequisite_met(&self, collection: &CardCollection) -> bool {
        match self {
            PermanentCard::ScoreBonus => {
                // Only available when all other permanent cards are collected
                collection.permanent_cards.len() >= 12
            }
            _ => true,
        }
    }
}

impl TemporalCard {
    pub fn get_display_info(&self) -> (&'static str, &'static str, f32) {
        match self {
            TemporalCard::CardDropRateIncrease { duration } => 
                ("Enhanced Senses", "Increased card detection chance", *duration),
            TemporalCard::IncreasedMagnetPower { duration } => 
                ("Magnetic Field", "Maximum ATP attraction strength", *duration),
            TemporalCard::ExtraATP { duration } => 
                ("Energy Abundance", "Double ATP from all sources", *duration),
            TemporalCard::MaximumFireRate { duration } => 
                ("Adrenaline Rush", "Maximum firing rate for all weapons", *duration),
            TemporalCard::RandomPowerUp => 
                ("Symbiotic Gift", "Grants a random power-up immediately", 0.0),
        }
    }

    pub fn get_default_duration() -> f32 {
        30.0 // 30 seconds for most temporal cards
    }
}

// ===== CARD SPAWNING SYSTEM =====

pub fn stage_completion_system(
    mut stage_progress: ResMut<StageProgress>,
    mut card_collection: ResMut<CardCollection>,
    wave_manager: Res<WaveManager>,
    mut spawn_card_events: EventWriter<SpawnCardEvent>,
    mut spawn_box_events: EventWriter<SpawnGreenBoxEvent>, // Add this
    enemy_query: Query<&Enemy>,
) {
    // Check if stage is complete (5 waves completed)
    if wave_manager.current_wave > 0 && wave_manager.current_wave % STAGE_WAVE_COUNT == 0 {
        if !wave_manager.wave_active && enemy_query.iter().count() == 0 {
            complete_stage(&mut stage_progress, &mut card_collection, &mut spawn_card_events, &mut spawn_box_events);
        }
    }
}

fn complete_stage(
    stage_progress: &mut StageProgress,
    card_collection: &mut CardCollection,
    spawn_card_events: &mut EventWriter<SpawnCardEvent>,
    spawn_box_events: &mut EventWriter<SpawnGreenBoxEvent>, 
) {
    let stage_number = stage_progress.current_stage;
    
    // Calculate if card should drop
    let mut rng_seed = (stage_number as f32 * 123.456).sin();
    let permanent_roll = rng_seed.abs();
    let temporal_roll = ((rng_seed * 234.567).sin()).abs();
    
    let card_type = if permanent_roll < PERMANENT_CARD_DROP_CHANCE {
        Some(generate_random_permanent_card(card_collection, stage_number))
    } else if temporal_roll < TEMPORAL_CARD_DROP_CHANCE {
        Some(CardType::Temporal(generate_random_temporal_card(stage_number)))
    } else {
        None
    };

    if let Some(card) = card_type {
        spawn_card_events.write(SpawnCardEvent {
            position: Vec3::new(0.0, 300.0, 0.0),
            card_type: card,
        });
    }

    // Maybe spawn green box for next stage
    let box_roll = ((stage_number as f32 * 345.678).sin()).abs();
    if box_roll < CARD_BOX_SPAWN_CHANCE {
        spawn_box_events.write(SpawnGreenBoxEvent {
            position: Vec3::new((stage_number as f32 * 100.0).sin() * 250.0, 350.0, 0.0),
            stage_number: stage_number + 1,
        });
    }

    // Reset stage progress
    stage_progress.current_stage += 1;
    stage_progress.waves_in_current_stage = 0;
    stage_progress.enemies_destroyed_this_stage = 0;
    stage_progress.total_enemies_this_stage = 0;
    stage_progress.damage_taken_this_stage = false;
    stage_progress.infrastructure_destroyed = 0;
    stage_progress.infrastructure_total = 0;
}

fn generate_random_permanent_card(collection: &CardCollection, stage: u32) -> CardType {
    let available_cards = vec![
        PermanentCard::CellDivision,
        PermanentCard::PowerfulDrones,
        PermanentCard::CardDropRateIncrease,
        PermanentCard::LuckyShield,
        PermanentCard::SupportDroneRight,
        PermanentCard::SupportDroneLeft,
        PermanentCard::ReducedMissileReload,
        PermanentCard::InitialFireRateUp,
        PermanentCard::IncreasedMissileDamage,
        PermanentCard::UpgradePricesReduced,
        PermanentCard::HealthRegeneration,
        PermanentCard::EmergencySporeToBegin,
        PermanentCard::IncreasedShipSpeed,
        PermanentCard::ScoreBonus,
    ];

    // Filter out already collected cards and check prerequisites
    let valid_cards: Vec<_> = available_cards
        .into_iter()
        .filter(|card| !collection.permanent_cards.contains(card))
        .filter(|card| card.is_prerequisite_met(collection))
        .collect();

    if valid_cards.is_empty() {
        // Fallback to temporal card
        return CardType::Temporal(generate_random_temporal_card(stage));
    }

    let index = ((stage as f32 * 456.789).sin().abs() * valid_cards.len() as f32) as usize;
    let selected_card = valid_cards[index.min(valid_cards.len() - 1)].clone();
    
    CardType::Permanent(selected_card)
}

fn generate_random_temporal_card(stage: u32) -> TemporalCard {
    let cards = vec![
        TemporalCard::CardDropRateIncrease { duration: 60.0 },
        TemporalCard::IncreasedMagnetPower { duration: 45.0 },
        TemporalCard::ExtraATP { duration: 30.0 },
        TemporalCard::MaximumFireRate { duration: 20.0 },
        TemporalCard::RandomPowerUp,
    ];

    let index = ((stage as f32 * 567.890).sin().abs() * cards.len() as f32) as usize;
    cards[index.min(cards.len() - 1)].clone()
}

// ===== CARD PICKUP SYSTEM =====

pub fn card_pickup_system(
    mut commands: Commands,
    card_query: Query<(Entity, &Transform, &Collider, &CardPickup), Without<PendingDespawn>>,
    player_query: Query<(&Transform, &Collider), With<Player>>,
    mut card_collection: ResMut<CardCollection>,
    mut spawn_events: EventWriter<SpawnParticles>,
) {
    if let Ok((player_transform, player_collider)) = player_query.single() {
        for (card_entity, card_transform, card_collider, card_pickup) in card_query.iter() {
            let distance = player_transform.translation.distance(card_transform.translation);
            
            if distance < player_collider.radius + card_pickup.collection_range {
                // Collect card
                match &card_pickup.card_type {
                    CardType::Permanent(perm_card) => {
                        if !card_collection.permanent_cards.contains(perm_card) {
                            card_collection.permanent_cards.insert(perm_card.clone());
                            apply_permanent_card_effect(perm_card, &mut commands);
                        }
                    }
                    CardType::Temporal(temp_card) => {
                        apply_temporal_card_effect(temp_card.clone(), &mut commands);
                        card_collection.active_temporal_cards.push((temp_card.clone(), TemporalCard::get_default_duration()));                    }
                }
                
                card_collection.cards_found_this_run += 1;
                card_collection.total_cards_found += 1;

                // Spawn collection particles
                spawn_events.write(SpawnParticles {
                    position: card_transform.translation,
                    count: 20,
                    config: ParticleConfig {
                        color_start: Color::srgb(0.8, 1.0, 0.3),
                        color_end: Color::srgba(0.3, 1.0, 0.8, 0.0),
                        velocity_range: (Vec2::new(-80.0, -40.0), Vec2::new(80.0, 120.0)),
                        lifetime_range: (1.0, 2.5),
                        size_range: (0.4, 1.2),
                        gravity: Vec2::new(0.0, -15.0),
                        organic_motion: true,
                        bioluminescence: 1.0,
                    },
                });

                commands.entity(card_entity)
                    .safe_despawn();
            }
        }
    }
}

fn apply_permanent_card_effect(card: &PermanentCard, commands: &mut Commands) {
    // Permanent card effects are applied immediately and persist
    match card {
        PermanentCard::CellDivision => {
            // Extra life - handled in player setup system
        }
        PermanentCard::PowerfulDrones => {
            // Support drone boost - handled in drone systems
        }
        PermanentCard::CardDropRateIncrease => {
            // Better drop rates - handled in card generation
        }
        PermanentCard::LuckyShield => {
            // Emergency shield - handled in damage system
        }
        PermanentCard::SupportDroneRight | PermanentCard::SupportDroneLeft => {
            // Spawn support drones - handled in drone setup system
        }
        PermanentCard::ReducedMissileReload => {
            // Faster missiles - handled in weapon systems
        }
        PermanentCard::InitialFireRateUp => {
            // Start with metabolic upgrade - handled in player setup
        }
        PermanentCard::IncreasedMissileDamage => {
            // Missile damage boost - handled in weapon systems
        }
        PermanentCard::UpgradePricesReduced => {
            // ATP discount - handled in upgrade systems
        }
        PermanentCard::HealthRegeneration => {
            // Health regen - handled in health system
        }
        PermanentCard::EmergencySporeToBegin => {
            // Extra spore - handled in player setup
        }
        PermanentCard::IncreasedShipSpeed => {
            // Speed boost - handled in movement system
        }
        PermanentCard::ScoreBonus => {
            // Score bonus - handled in scoring system
        }
    }
}

fn apply_temporal_card_effect(card: TemporalCard, commands: &mut Commands) {
    match card {
        TemporalCard::RandomPowerUp => {
            // Immediately spawn a random power-up near player
            // This will be handled by existing power-up systems
        }
        _ => {
            // Other temporal effects are duration-based and handled by update systems
        }
    }
}

// ===== CARD VISUAL SYSTEMS =====

pub fn spawn_card_system(
    mut commands: Commands,
    mut spawn_events: EventReader<SpawnCardEvent>,
    assets: Option<Res<GameAssets>>,
) {
    let Some(assets) = assets else { return };
    
    for event in spawn_events.read() {
        let (texture, color) = match &event.card_type {
            CardType::Permanent(_) => {
                (assets.permanent_card_texture.clone(), Color::srgb(1.0, 0.8, 0.3)) // Gold
            }
            CardType::Temporal(_) => {
                (assets.temporal_card_texture.clone(), Color::srgb(0.3, 0.8, 1.0)) // Blue
            }
        };

        commands.spawn((
            Sprite {
                image: texture,
                color,
                custom_size: Some(Vec2::splat(24.0)),
                ..default()
            },
            Transform::from_translation(event.position),
            CardPickup {
                card_type: event.card_type.clone(),
                bob_timer: 0.0,
                collection_range: 25.0,
            },
            Collider { radius: 12.0 },
            BioluminescentParticle {
                base_color: color,
                pulse_frequency: 2.0,
                pulse_intensity: 0.8,
                organic_motion: OrganicMotion {
                    undulation_speed: 1.5,
                    response_to_current: 0.4,
                },
            },
        ));
    }
}

pub fn spawn_green_box_system(
    mut commands: Commands,
    mut spawn_events: EventReader<SpawnGreenBoxEvent>,
    assets: Option<Res<GameAssets>>,
) {
    let Some(assets) = assets else { return };
    
    for event in spawn_events.read() {
        commands.spawn((
            Sprite {
                image: assets.green_box_texture.clone(),
                color: Color::srgb(0.3, 1.0, 0.3),
                custom_size: Some(Vec2::splat(20.0)),
                ..default()
            },
            Transform::from_translation(event.position),
            GreenCardBox {
                stage_number: event.stage_number,
                guaranteed_card: true,
            },
            Collider { radius: 10.0 },
            BioluminescentParticle {
                base_color: Color::srgb(0.3, 1.0, 0.3),
                pulse_frequency: 3.0,
                pulse_intensity: 0.9,
                organic_motion: OrganicMotion {
                    undulation_speed: 2.0,
                    response_to_current: 0.3,
                },
            },
        ));
    }
}

pub fn move_cards_and_boxes(
    mut card_query: Query<(&mut Transform, &mut CardPickup), (With<CardPickup>, Without<GreenCardBox>)>,
    mut box_query: Query<(&mut Transform, &mut GreenCardBox), (With<GreenCardBox>, Without<CardPickup>)>,
    time: Res<Time>,
) {
    // Move cards with organic floating motion
    for (mut transform, mut card_pickup) in card_query.iter_mut() {
        transform.translation.y -= 60.0 * time.delta_secs();
        
        card_pickup.bob_timer += time.delta_secs() * 2.0;
        let bob_amplitude = 12.0;
        transform.translation.y += card_pickup.bob_timer.sin() * bob_amplitude * time.delta_secs();
        
        // Gentle horizontal drift
        let drift = (time.elapsed_secs() * 1.2 + transform.translation.x * 0.003).sin() * 25.0;
        transform.translation.x += drift * time.delta_secs();
        
        // Organic rotation
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 0.8);
    }

    // Move green boxes similarly but slower
    for (mut transform, mut green_box) in box_query.iter_mut() {
        transform.translation.y -= 40.0 * time.delta_secs();
        
        let bob_phase = time.elapsed_secs() * 1.5 + transform.translation.x * 0.005;
        let bob_amplitude = 8.0;
        transform.translation.y += bob_phase.sin() * bob_amplitude * time.delta_secs();
        
        // Slower rotation for boxes
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 0.5);
    }
}

pub fn green_box_collection_system(
    mut commands: Commands,
    box_query: Query<(Entity, &Transform, &Collider, &GreenCardBox), Without<PendingDespawn>>,
    player_query: Query<(&Transform, &Collider), With<Player>>,
    mut card_collection: ResMut<CardCollection>,
    mut spawn_card_events: EventWriter<SpawnCardEvent>,
) {
    if let Ok((player_transform, player_collider)) = player_query.single() {
        for (box_entity, box_transform, box_collider, green_box) in box_query.iter() {
            let distance = player_transform.translation.distance(box_transform.translation);
            
            if distance < player_collider.radius + box_collider.radius {
                // Spawn guaranteed card
                let card_type = if (green_box.stage_number as f32 * 789.012).sin().abs() < 0.3 {
                    generate_random_permanent_card(&card_collection, green_box.stage_number)
                } else {
                    CardType::Temporal(generate_random_temporal_card(green_box.stage_number))
                };

                spawn_card_events.write(SpawnCardEvent {
                    position: box_transform.translation + Vec3::new(0.0, 30.0, 0.0),
                    card_type,
                });

                commands.entity(box_entity)
                    .safe_despawn();
            }
        }
    }
}

// ===== TEMPORAL CARD UPDATE SYSTEM =====

pub fn update_temporal_cards(
    mut card_collection: ResMut<CardCollection>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    // Update active temporal cards
    card_collection.active_temporal_cards.retain_mut(|(card, remaining_time)| {
        *remaining_time -= dt;
        
        if *remaining_time <= 0.0 {
            // Card expired, remove effects if needed
            remove_temporal_card_effect(card, &mut commands);
            false
        } else {
            true
        }
    });
}

fn remove_temporal_card_effect(card: &TemporalCard, commands: &mut Commands) {
    match card {
        TemporalCard::CardDropRateIncrease { .. } => {
            // Drop rate returns to normal
        }
        TemporalCard::IncreasedMagnetPower { .. } => {
            // Magnet returns to normal strength
        }
        TemporalCard::ExtraATP { .. } => {
            // ATP multiplier returns to 1x
        }
        TemporalCard::MaximumFireRate { .. } => {
            // Fire rate returns to normal
        }
        TemporalCard::RandomPowerUp => {
            // One-time effect, nothing to remove
        }
    }
}



// ===== CARD EFFECT QUERIES =====

pub fn has_permanent_card(collection: &CardCollection, card: &PermanentCard) -> bool {
    collection.permanent_cards.contains(card)
}

pub fn has_temporal_card_active(collection: &CardCollection, card_check: impl Fn(&TemporalCard) -> bool) -> Option<f32> {
    collection.active_temporal_cards
        .iter()
        .find(|(card, _)| card_check(card))
        .map(|(_, remaining_time)| *remaining_time)
}

pub fn get_card_drop_rate_multiplier(collection: &CardCollection) -> f32 {
    let mut multiplier = 1.0;
    
    if has_permanent_card(collection, &PermanentCard::CardDropRateIncrease) {
        multiplier *= 1.5;
    }
    
    if has_temporal_card_active(collection, |card| matches!(card, TemporalCard::CardDropRateIncrease { .. })).is_some() {
        multiplier *= 2.0;
    }
    
    multiplier
}

pub fn get_atp_multiplier(collection: &CardCollection) -> f32 {
    if has_temporal_card_active(collection, |card| matches!(card, TemporalCard::ExtraATP { .. })).is_some() {
        2.0
    } else {
        1.0
    }
}

pub fn get_upgrade_discount(collection: &CardCollection) -> f32 {
    if has_permanent_card(collection, &PermanentCard::UpgradePricesReduced) {
        0.9 // 10% discount
    } else {
        1.0
    }
}

// ===== INTEGRATION WITH EXISTING SYSTEMS =====

// Add these to your existing player setup system
pub fn apply_permanent_card_effects_to_player(
    mut player_query: Query<(&mut Player, &mut CellularUpgrades, &mut EvolutionSystem), Added<Player>>,
    card_collection: Res<CardCollection>,
) {
    for (mut player, mut upgrades, mut evolution_system) in player_query.iter_mut() {
        if has_permanent_card(&card_collection, &PermanentCard::CellDivision) {
            player.lives += 1;
        }
        
        if has_permanent_card(&card_collection, &PermanentCard::IncreasedShipSpeed) {
            player.speed *= 1.2;
        }
        
        if has_permanent_card(&card_collection, &PermanentCard::InitialFireRateUp) {
            upgrades.metabolic_rate *= 1.15;
        }
        
        if has_permanent_card(&card_collection, &PermanentCard::EmergencySporeToBegin) {
            evolution_system.emergency_spores += 1;
        }
    }
}