use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;

// Currency pickup system
pub fn currency_pickup_system(
    mut commands: Commands,
    currency_query: Query<(Entity, &Transform, &Collider), With<Currency>>,
    mut player_query: Query<(&Transform, &Collider, &mut Currency), With<Player>>,
    mut game_score: ResMut<GameScore>,
) {
    if let Ok((player_transform, player_collider, mut player_currency)) = player_query.single_mut() {
        for (currency_entity, currency_transform, currency_collider) in currency_query.iter() {
            let distance = player_transform.translation.distance(currency_transform.translation);
            if distance < player_collider.radius + currency_collider.radius {
                // Collect currency
                if let Ok(currency_component) = commands.entity(currency_entity).get::<Currency>() {
                    player_currency.amount += currency_component.amount;
                    game_score.current += currency_component.amount * 10; // Currency also gives points
                }
                commands.entity(currency_entity).despawn();
            }
        }
    }
}

// Spawn currency on enemy death
pub fn spawn_currency_on_death(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for event in explosion_events.read() {
            if let Some(enemy_type) = &event.enemy_type {
                let currency_amount = match enemy_type {
                    EnemyType::Basic => 1,
                    EnemyType::Fast => 2,
                    EnemyType::Heavy => 5,
                    EnemyType::Boss => 25,
                    EnemyType::Kamikaze => 3,
                    EnemyType::Turret => 8,
                    EnemyType::FormationFighter => 4,
                    EnemyType::Spawner => 15,
                    EnemyType::SpawnerMinion => 1,
                };
                
                // Random chance to drop currency (70% chance)
                if (event.position.x * 123.456).sin().abs() > 0.3 {
                    commands.spawn((
                        Sprite {
                            image: assets.multiplier_powerup_texture.clone(), // Reuse multiplier texture for currency
                            color: Color::srgb(1.0, 1.0, 0.2),
                            custom_size: Some(Vec2::splat(16.0)),
                            ..default()
                        },
                        Transform::from_translation(event.position),
                        Currency { amount: currency_amount },
                        Collider { radius: 8.0 },
                    ));
                }
            }
        }
    }
}

// Move currency downward
pub fn move_currency(
    mut currency_query: Query<&mut Transform, With<Currency>>,
    time: Res<Time>,
) {
    for mut transform in currency_query.iter_mut() {
        transform.translation.y -= 150.0 * time.delta_secs();
        
        // Add gentle floating animation
        transform.translation.y += (time.elapsed_secs() * 3.0 + transform.translation.x * 0.01).sin() * 20.0 * time.delta_secs();
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 2.0);
    }
}

// Upgrade station system
pub fn upgrade_station_interaction(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    upgrade_station_query: Query<&Transform, With<UpgradeStation>>,
    mut player_query: Query<(&Transform, &mut Currency, &mut WeaponSystem, &mut PermanentUpgrades), With<Player>>,
) {
    if let Ok((player_transform, mut currency, mut weapon_system, mut upgrades)) = player_query.single_mut() {
        for station_transform in upgrade_station_query.iter() {
            let distance = player_transform.translation.distance(station_transform.translation);
            if distance < 50.0 {
                // Player is near upgrade station
                if keyboard.just_pressed(KeyCode::Digit1) && currency.amount >= 10 {
                    // Upgrade damage
                    currency.amount -= 10;
                    upgrades.damage_boost *= 1.2;
                    weapon_system.weapon_upgrades.damage_multiplier *= 1.2;
                }
                
                if keyboard.just_pressed(KeyCode::Digit2) && currency.amount >= 15 {
                    // Upgrade fire rate
                    currency.amount -= 15;
                    upgrades.fire_rate_boost *= 1.3;
                    weapon_system.weapon_upgrades.fire_rate_multiplier *= 1.3;
                }
                
                if keyboard.just_pressed(KeyCode::Digit3) && currency.amount >= 20 {
                    // Upgrade max health
                    currency.amount -= 20;
                    upgrades.max_health += 25;
                }
                
                if keyboard.just_pressed(KeyCode::Digit4) && currency.amount >= 25 {
                    // Buy armor piercing
                    currency.amount -= 25;
                    weapon_system.weapon_upgrades.armor_piercing = true;
                }
                
                if keyboard.just_pressed(KeyCode::Digit5) && currency.amount >= 30 {
                    // Buy explosive rounds
                    currency.amount -= 30;
                    weapon_system.weapon_upgrades.explosive_rounds = true;
                }
                
                if keyboard.just_pressed(KeyCode::Digit6) && currency.amount >= 20 {
                    // Buy smart bomb
                    currency.amount -= 20;
                    weapon_system.smart_bombs += 1;
                }
                
                if keyboard.just_pressed(KeyCode::Digit7) && currency.amount >= 50 {
                    // Upgrade to spread shot
                    currency.amount -= 50;
                    weapon_system.primary_weapon = WeaponType::SpreadShot {
                        damage: 8,
                        fire_rate: 0.15,
                        spread_count: 5,
                        spread_angle: 0.6,
                    };
                }
                
                if keyboard.just_pressed(KeyCode::Digit8) && currency.amount >= 75 {
                    // Upgrade to missile launcher
                    currency.amount -= 75;
                    weapon_system.primary_weapon = WeaponType::Missile {
                        damage: 25,
                        fire_rate: 0.8,
                        homing_strength: 2.0,
                        blast_radius: 50.0,
                    };
                }
                
                if keyboard.just_pressed(KeyCode::Digit9) && currency.amount >= 100 {
                    // Upgrade to laser
                    currency.amount -= 100;
                    weapon_system.primary_weapon = WeaponType::Laser {
                        damage: 15,
                        charge_time: 1.0,
                        duration: 2.0,
                        width: 20.0,
                    };
                }
            }
        }
    }
}

// Weapon power-up collection system
pub fn weapon_powerup_collection(
    mut commands: Commands,
    powerup_query: Query<(Entity, &Transform, &Collider, &WeaponPowerUp)>,
    mut player_query: Query<(Entity, &Transform, &Collider, &mut WeaponSystem), With<Player>>,
) {
    if let Ok((player_entity, player_transform, player_collider, mut weapon_system)) = player_query.single_mut() {
        for (powerup_entity, powerup_transform, powerup_collider, weapon_powerup) in powerup_query.iter() {
            let distance = player_transform.translation.distance(powerup_transform.translation);
            if distance < player_collider.radius + powerup_collider.radius {
                match &weapon_powerup.upgrade_type {
                    WeaponUpgradeType::DamageBoost(multiplier) => {
                        weapon_system.weapon_upgrades.damage_multiplier *= multiplier;
                        if weapon_powerup.temporary {
                            // Add temporary effect component
                            commands.entity(player_entity).insert(TemporaryDamageBoost {
                                timer: weapon_powerup.duration.unwrap_or(10.0),
                                multiplier: *multiplier,
                            });
                        }
                    }
                    
                    WeaponUpgradeType::FireRateBoost(multiplier) => {
                        weapon_system.weapon_upgrades.fire_rate_multiplier *= multiplier;
                        if weapon_powerup.temporary {
                            commands.entity(player_entity).insert(TemporaryFireRateBoost {
                                timer: weapon_powerup.duration.unwrap_or(10.0),
                                multiplier: *multiplier,
                            });
                        }
                    }
                    
                    WeaponUpgradeType::ArmorPiercing => {
                        weapon_system.weapon_upgrades.armor_piercing = true;
                        if weapon_powerup.temporary {
                            commands.entity(player_entity).insert(TemporaryArmorPiercing {
                                timer: weapon_powerup.duration.unwrap_or(15.0),
                            });
                        }
                    }
                    
                    WeaponUpgradeType::ExplosiveRounds => {
                        weapon_system.weapon_upgrades.explosive_rounds = true;
                        if weapon_powerup.temporary {
                            commands.entity(player_entity).insert(TemporaryExplosiveRounds {
                                timer: weapon_powerup.duration.unwrap_or(15.0),
                            });
                        }
                    }
                    
                    WeaponUpgradeType::HomingUpgrade => {
                        weapon_system.weapon_upgrades.homing_enhancement = true;
                        if weapon_powerup.temporary {
                            commands.entity(player_entity).insert(TemporaryHomingUpgrade {
                                timer: weapon_powerup.duration.unwrap_or(20.0),
                            });
                        }
                    }
                    
                    WeaponUpgradeType::WeaponSwap(new_weapon) => {
                        weapon_system.secondary_weapon = Some(weapon_system.primary_weapon.clone());
                        weapon_system.primary_weapon = new_weapon.clone();
                        if weapon_powerup.temporary {
                            commands.entity(player_entity).insert(TemporaryWeaponSwap {
                                timer: weapon_powerup.duration.unwrap_or(25.0),
                                original_weapon: weapon_system.secondary_weapon.clone().unwrap(),
                            });
                        }
                    }
                }
                
                commands.entity(powerup_entity).despawn();
            }
        }
    }
}

// Temporary weapon effect components
#[derive(Component)]
pub struct TemporaryDamageBoost {
    pub timer: f32,
    pub multiplier: f32,
}

#[derive(Component)]
pub struct TemporaryFireRateBoost {
    pub timer: f32,
    pub multiplier: f32,
}

#[derive(Component)]
pub struct TemporaryArmorPiercing {
    pub timer: f32,
}

#[derive(Component)]
pub struct TemporaryExplosiveRounds {
    pub timer: f32,
}

#[derive(Component)]
pub struct TemporaryHomingUpgrade {
    pub timer: f32,
}

#[derive(Component)]
pub struct TemporaryWeaponSwap {
    pub timer: f32,
    pub original_weapon: WeaponType,
}

// Update temporary weapon effects
pub fn update_temporary_weapon_effects(
    mut commands: Commands,
    mut damage_boost_query: Query<(Entity, &mut TemporaryDamageBoost)>,
    mut fire_rate_boost_query: Query<(Entity, &mut TemporaryFireRateBoost)>,
    mut armor_piercing_query: Query<(Entity, &mut TemporaryArmorPiercing)>,
    mut explosive_rounds_query: Query<(Entity, &mut TemporaryExplosiveRounds)>,
    mut homing_upgrade_query: Query<(Entity, &mut TemporaryHomingUpgrade)>,
    mut weapon_swap_query: Query<(Entity, &mut TemporaryWeaponSwap)>,
    mut player_query: Query<&mut WeaponSystem, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut weapon_system) = player_query.single_mut() {
        // Update damage boost
        for (entity, mut boost) in damage_boost_query.iter_mut() {
            boost.timer -= time.delta_secs();
            if boost.timer <= 0.0 {
                weapon_system.weapon_upgrades.damage_multiplier /= boost.multiplier;
                commands.entity(entity).remove::<TemporaryDamageBoost>();
            }
        }
        
        // Update fire rate boost
        for (entity, mut boost) in fire_rate_boost_query.iter_mut() {
            boost.timer -= time.delta_secs();
            if boost.timer <= 0.0 {
                weapon_system.weapon_upgrades.fire_rate_multiplier /= boost.multiplier;
                commands.entity(entity).remove::<TemporaryFireRateBoost>();
            }
        }
        
        // Update armor piercing
        for (entity, mut piercing) in armor_piercing_query.iter_mut() {
            piercing.timer -= time.delta_secs();
            if piercing.timer <= 0.0 {
                weapon_system.weapon_upgrades.armor_piercing = false;
                commands.entity(entity).remove::<TemporaryArmorPiercing>();
            }
        }
        
        // Update explosive rounds
        for (entity, mut explosive) in explosive_rounds_query.iter_mut() {
            explosive.timer -= time.delta_secs();
            if explosive.timer <= 0.0 {
                weapon_system.weapon_upgrades.explosive_rounds = false;
                commands.entity(entity).remove::<TemporaryExplosiveRounds>();
            }
        }
        
        // Update homing upgrade
        for (entity, mut homing) in homing_upgrade_query.iter_mut() {
            homing.timer -= time.delta_secs();
            if homing.timer <= 0.0 {
                weapon_system.weapon_upgrades.homing_enhancement = false;
                commands.entity(entity).remove::<TemporaryHomingUpgrade>();
            }
        }
        
        // Update weapon swap
        for (entity, mut swap) in weapon_swap_query.iter_mut() {
            swap.timer -= time.delta_secs();
            if swap.timer <= 0.0 {
                weapon_system.primary_weapon = swap.original_weapon.clone();
                weapon_system.secondary_weapon = None;
                commands.entity(entity).remove::<TemporaryWeaponSwap>();
            }
        }
    }
}

// Spawn weapon power-ups occasionally
pub fn spawn_weapon_powerups(
    mut commands: Commands,
    mut enemy_spawner: ResMut<EnemySpawner>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        // Spawn weapon power-ups less frequently than regular power-ups
        if enemy_spawner.wave_timer % 25.0 < 0.1 && enemy_spawner.wave_timer > 0.1 {
            let x_position = (time.elapsed_secs() * 30.0).sin() * 250.0;
            
            let (upgrade_type, texture, temporary) = match (time.elapsed_secs() as u32 / 25) % 6 {
                0 => (WeaponUpgradeType::DamageBoost(1.5), assets.health_powerup_texture.clone(), true),
                1 => (WeaponUpgradeType::FireRateBoost(1.8), assets.rapidfire_powerup_texture.clone(), true),
                2 => (WeaponUpgradeType::ArmorPiercing, assets.shield_powerup_texture.clone(), true),
                3 => (WeaponUpgradeType::ExplosiveRounds, assets.explosion_texture.clone(), true),
                4 => (WeaponUpgradeType::WeaponSwap(WeaponType::SpreadShot {
                    damage: 12, fire_rate: 0.12, spread_count: 7, spread_angle: 0.8
                }), assets.multiplier_powerup_texture.clone(), true),
                _ => (WeaponUpgradeType::WeaponSwap(WeaponType::RapidFire {
                    damage: 6, fire_rate: 0.05, heat_buildup: 0.0
                }), assets.speed_powerup_texture.clone(), true),
            };
            
            commands.spawn((
                Sprite {
                    image: texture,
                    color: Color::srgb(0.8, 1.0, 0.8),
                    ..default()
                },
                Transform::from_xyz(x_position, 400.0, 0.0),
                WeaponPowerUp {
                    weapon_type: WeaponType::Basic { damage: 10, fire_rate: 0.1 },
                    upgrade_type,
                    temporary,
                    duration: Some(15.0),
                },
                Collider { radius: 12.0 },
            ));
        }
    }
}

// Spawn upgrade stations occasionally
pub fn spawn_upgrade_stations(
    mut commands: Commands,
    mut enemy_spawner: ResMut<EnemySpawner>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        // Spawn upgrade station every 60 seconds
        if enemy_spawner.wave_timer % 60.0 < 0.1 && enemy_spawner.wave_timer > 30.0 {
            commands.spawn((
                Sprite {
                    image: assets.enemy_texture.clone(),
                    color: Color::srgb(0.2, 0.8, 0.2),
                    custom_size: Some(Vec2::splat(40.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 350.0, 0.0),
                UpgradeStation,
            ));
        }
    }
}

// Update upgrade station UI
pub fn update_upgrade_ui(
    mut commands: Commands,
    upgrade_station_query: Query<&Transform, With<UpgradeStation>>,
    player_query: Query<(&Transform, &Currency), With<Player>>,
    existing_ui_query: Query<Entity, With<UpgradeUI>>,
) {
    if let Ok((player_transform, currency)) = player_query.single() {
        let near_station = upgrade_station_query.iter().any(|station_transform| {
            player_transform.translation.distance(station_transform.translation) < 50.0
        });
        
        if near_station {
            // Show upgrade UI if not already showing
            if existing_ui_query.is_empty() {
                spawn_upgrade_ui(&mut commands, currency.amount);
            }
        } else {
            // Hide upgrade UI if showing
            for entity in existing_ui_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

#[derive(Component)]
struct UpgradeUI;

fn spawn_upgrade_ui(commands: &mut Commands, currency_amount: u32) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(100.0),
            width: Val::Px(300.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        UpgradeUI,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("UPGRADE STATION"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(0.2, 1.0, 0.2)),
            Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
        ));
        
        parent.spawn((
            Text::new(format!("Currency: {}", currency_amount)),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
        ));
        
        let upgrades = [
            ("1 - Damage +20% (10¤)", 10),
            ("2 - Fire Rate +30% (15¤)", 15),
            ("3 - Max Health +25 (20¤)", 20),
            ("4 - Armor Piercing (25¤)", 25),
            ("5 - Explosive Rounds (30¤)", 30),
            ("6 - Smart Bomb (20¤)", 20),
            ("7 - Spread Shot (50¤)", 50),
            ("8 - Missile Launcher (75¤)", 75),
            ("9 - Laser Cannon (100¤)", 100),
        ];
        
        for (text, cost) in upgrades {
            let color = if currency_amount >= cost {
                Color::WHITE
            } else {
                Color::srgb(0.5, 0.5, 0.5)
            };
            
            parent.spawn((
                Text::new(text),
                TextFont { font_size: 14.0, ..default() },
                TextColor(color),
                Node { margin: UiRect::bottom(Val::Px(5.0)), ..default() },
            ));
        }
    });
}
