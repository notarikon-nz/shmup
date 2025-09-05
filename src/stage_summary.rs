// src/stage_summary.rs
use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::card_system::*;
use crate::wave_systems::*;
use crate::enemy_types::*;
use crate::events::*;
use crate::achievements::*;
use crate::input::*;
use crate::despawn::*;
use std::collections::HashSet;

// ===== CONSTANTS =====
const PERFECT_ATP_BONUS: u32 = 5000;
const NO_DAMAGE_BONUS: u32 = 5000;
const OBJECTIVE_MULTIPLIERS: [f32; 4] = [2.0, 4.0, 8.0, 16.0]; // Up to 16x multiplier
const FULL_DECK_BONUS_PERCENT: f32 = 0.1; // 10%

// ===== STAGE SUMMARY COMPONENTS =====

#[derive(Component)]
pub struct StageSummaryUI;

#[derive(Component)]
pub struct SummaryScoreText;

#[derive(Component)]
pub struct SummaryButton;

#[derive(Resource, Default)]
pub struct StageSummaryData {
    pub stage_number: u32,
    pub base_score: u32,
    pub atp_collected_percent: f32,
    pub enemies_destroyed_percent: f32,
    pub all_infrastructure_destroyed: bool,
    pub no_damage_taken: bool,
    pub score_multiplier: f32,
    pub full_deck_bonus_applicable: bool,
    pub final_score: u32,
    pub show_summary: bool,
}

// ===== STAGE OBJECTIVES =====

#[derive(Clone, Debug)]
pub enum StageObjective {
    SeventyPercentEnemies,     // 70% of enemies destroyed
    HundredPercentEnemies,     // 100% of enemies destroyed  
    AllInfrastructureDestroyed, // All infrastructure targets destroyed
    Untouched,                 // No damage taken
}

impl StageObjective {
    pub fn get_display_text(&self) -> &'static str {
        match self {
            StageObjective::SeventyPercentEnemies => "70% of Enemies Destroyed",
            StageObjective::HundredPercentEnemies => "100% of Enemies Destroyed",
            StageObjective::AllInfrastructureDestroyed => "All Infrastructure Targets Destroyed",
            StageObjective::Untouched => "Untouched (No Damage)",
        }
    }

    pub fn is_completed(&self, summary_data: &StageSummaryData) -> bool {
        match self {
            StageObjective::SeventyPercentEnemies => summary_data.enemies_destroyed_percent >= 0.70,
            StageObjective::HundredPercentEnemies => summary_data.enemies_destroyed_percent >= 1.00,
            StageObjective::AllInfrastructureDestroyed => summary_data.all_infrastructure_destroyed,
            StageObjective::Untouched => summary_data.no_damage_taken,
        }
    }
}

// ===== SUMMARY CALCULATION =====

pub fn calculate_stage_summary(
    stage_progress: &StageProgress,
    game_score: &GameScore,
    card_collection: &CardCollection,
) -> StageSummaryData {
    let base_score = game_score.stage_score;
    
    // Calculate ATP collection percentage
    let atp_collected_percent = if game_score.stage_atp_total > 0 {
        game_score.stage_atp_collected as f32 / game_score.stage_atp_total as f32
    } else { 1.0 };
    
    // Calculate enemy destruction percentage
    let enemies_destroyed_percent = if stage_progress.total_enemies_this_stage > 0 {
        stage_progress.enemies_destroyed_this_stage as f32 / stage_progress.total_enemies_this_stage as f32
    } else { 1.0 };
    
    // Check objectives completion
    let objectives = vec![
        StageObjective::SeventyPercentEnemies,
        StageObjective::HundredPercentEnemies,
        StageObjective::AllInfrastructureDestroyed,
        StageObjective::Untouched,
    ];
    
    let temp_summary = StageSummaryData {
        stage_number: stage_progress.current_stage,
        base_score,
        atp_collected_percent,
        enemies_destroyed_percent,
        all_infrastructure_destroyed: stage_progress.infrastructure_destroyed >= stage_progress.infrastructure_total,
        no_damage_taken: !stage_progress.damage_taken_this_stage,
        score_multiplier: 1.0,
        full_deck_bonus_applicable: card_collection.permanent_cards.len() >= 13,
        final_score: 0,
        show_summary: true,
    };
    
    // Calculate score multiplier based on completed objectives
    let completed_objectives = objectives.iter()
        .filter(|obj| obj.is_completed(&temp_summary))
        .count();
    
    let score_multiplier = if completed_objectives > 0 {
        OBJECTIVE_MULTIPLIERS[completed_objectives - 1]
    } else { 1.0 };
    
    // Calculate bonuses
    let mut total_bonus = 0;
    
    // ATP bonus (5k if none missed)
    if atp_collected_percent >= 1.0 {
        total_bonus += PERFECT_ATP_BONUS;
    }
    
    // No damage bonus
    if temp_summary.no_damage_taken {
        total_bonus += NO_DAMAGE_BONUS;
    }
    
    // Calculate final score: ((Base Score + Bonuses) x Multipliers) + Full Deck Bonus
    let pre_deck_bonus = ((base_score + total_bonus) as f32 * score_multiplier) as u32;
    let full_deck_bonus = if temp_summary.full_deck_bonus_applicable {
        (pre_deck_bonus as f32 * FULL_DECK_BONUS_PERCENT) as u32
    } else { 0 };
    
    let final_score = pre_deck_bonus + full_deck_bonus;
    
    StageSummaryData {
        score_multiplier,
        final_score,
        ..temp_summary
    }
}

// ===== STAGE SUMMARY SYSTEMS =====

pub fn stage_summary_trigger_system(
    mut commands: Commands,
    stage_progress: Res<StageProgress>,
    mut game_score: ResMut<GameScore>,
    card_collection: Res<CardCollection>,
    wave_manager: Res<WaveManager>,
    enemy_query: Query<&Enemy>,
    mut summary_data: ResMut<StageSummaryData>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Trigger summary when stage is complete (every 5 waves)
    if wave_manager.current_wave > 0 && 
       wave_manager.current_wave % 5 == 0 && 
       !wave_manager.wave_active && 
       enemy_query.iter().count() == 0 &&
       !summary_data.show_summary {
        
        // Calculate summary data
        *summary_data = calculate_stage_summary(&stage_progress, &game_score, &card_collection);
        
        // Add final score to total
        game_score.current += summary_data.final_score;
        
        // Show summary
        next_state.set(GameState::StageSummary);
    }
}

pub fn setup_stage_summary_ui(
    mut commands: Commands,
    fonts: Res<GameFonts>,
    summary_data: Res<StageSummaryData>,
) {
    let font = fonts.default_font.clone();
    
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.1, 0.05, 0.95)),
        StageSummaryUI,
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(700.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(30.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.2, 0.15, 0.98)),
            BorderColor(Color::srgb(0.4, 0.8, 0.6)),
        )).with_children(|summary| {
            // Title
            summary.spawn((
                Text::new(&format!("STAGE {:02} COMPLETE", summary_data.stage_number)),
                TextFont { font: font.clone(), font_size: 36.0, ..default() },
                TextColor(Color::srgb(0.3, 1.0, 0.7)),
                Node { margin: UiRect::bottom(Val::Px(20.0)), align_self: AlignSelf::Center, ..default() },
            ));
            
            // Base Score
            summary.spawn((
                Text::new(&format!("Base Score: {}", summary_data.base_score)),
                TextFont { font: font.clone(), font_size: 22.0, ..default() },
                TextColor(Color::WHITE),
                Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
            ));
            
            // ATP Collection
            let atp_color = if summary_data.atp_collected_percent >= 1.0 {
                Color::srgb(0.3, 1.0, 0.3) // Perfect - Green
            } else if summary_data.atp_collected_percent >= 0.8 {
                Color::srgb(0.8, 0.8, 0.3) // Good - Yellow
            } else {
                Color::srgb(0.8, 0.3, 0.3) // Poor - Red
            };
            
            summary.spawn((
                Text::new(&format!("ATP Collected: {:.1}%{}", 
                    summary_data.atp_collected_percent * 100.0,
                    if summary_data.atp_collected_percent >= 1.0 { " (+5,000 bonus)" } else { "" }
                )),
                TextFont { font: font.clone(), font_size: 18.0, ..default() },
                TextColor(atp_color),
                Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
            ));
            
            // No Damage Bonus
            if summary_data.no_damage_taken {
                summary.spawn((
                    Text::new("No Damage: +5,000 bonus"),
                    TextFont { font: font.clone(), font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.3, 1.0, 0.3)),
                    Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
                ));
            }
            
            // Objectives Section
            summary.spawn((
                Text::new("OBJECTIVES"),
                TextFont { font: font.clone(), font_size: 24.0, ..default() },
                TextColor(Color::srgb(0.6, 1.0, 0.8)),
                Node { margin: UiRect::vertical(Val::Px(15.0)), align_self: AlignSelf::Center, ..default() },
            ));
            
            let objectives = vec![
                StageObjective::SeventyPercentEnemies,
                StageObjective::HundredPercentEnemies,
                StageObjective::AllInfrastructureDestroyed,
                StageObjective::Untouched,
            ];
            
            for objective in objectives {
                let completed = objective.is_completed(&summary_data);
                let color = if completed { 
                    Color::srgb(0.3, 1.0, 0.3) 
                } else { 
                    Color::srgb(0.5, 0.5, 0.5) 
                };
                let symbol = if completed { "✓" } else { "✗" };
                
                summary.spawn((
                    Text::new(&format!("{} {}", symbol, objective.get_display_text())),
                    TextFont { font: font.clone(), font_size: 16.0, ..default() },
                    TextColor(color),
                    Node { margin: UiRect::bottom(Val::Px(5.0)), ..default() },
                ));
            }
            
            // Score Multiplier
            let multiplier_color = if summary_data.score_multiplier >= 8.0 {
                Color::srgb(1.0, 0.8, 0.2) // Gold for high multiplier
            } else if summary_data.score_multiplier >= 4.0 {
                Color::srgb(0.8, 0.8, 0.3) // Yellow for medium
            } else {
                Color::WHITE
            };
            
            summary.spawn((
                Text::new(&format!("Score Multiplier: {:.0}x", summary_data.score_multiplier)),
                TextFont { font: font.clone(), font_size: 20.0, ..default() },
                TextColor(multiplier_color),
                Node { margin: UiRect::vertical(Val::Px(15.0)), ..default() },
            ));
            
            // Full Deck Bonus
            if summary_data.full_deck_bonus_applicable {
                summary.spawn((
                    Text::new("Full Genome Bonus: 10%"),
                    TextFont { font: font.clone(), font_size: 18.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.8, 0.2)),
                    Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
                ));
            }
            
            // Final Score (Highlighted)
            summary.spawn((
                Text::new(&format!("FINAL SCORE: {}", summary_data.final_score)),
                TextFont { font: font.clone(), font_size: 32.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.3)),
                Node { 
                    margin: UiRect::vertical(Val::Px(20.0)), 
                    align_self: AlignSelf::Center, 
                    ..default() 
                },
            ));
            
            // Continue Button
            summary.spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.7, 0.4)),
                SummaryButton,
            )).with_children(|button| {
                button.spawn((
                    Text::new("CONTINUE EVOLUTION"),
                    TextFont { font: font.clone(), font_size: 18.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
            
            // Stage calculation breakdown (small text)
            let breakdown_text = format!(
                "Calculation: (({} + bonuses) × {:.0}) + deck bonus = {}",
                summary_data.base_score,
                summary_data.score_multiplier,
                summary_data.final_score
            );
            
            summary.spawn((
                Text::new(breakdown_text),
                TextFont { font: font.clone(), font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.6, 0.8, 0.6)),
                Node { 
                    margin: UiRect::top(Val::Px(15.0)), 
                    align_self: AlignSelf::Center, 
                    ..default() 
                },
            ));
        });
    });
}

pub fn stage_summary_button_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<SummaryButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut summary_data: ResMut<StageSummaryData>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                summary_data.show_summary = false;
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.25, 0.8, 0.5));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.7, 0.4));
            }
        }
    }
}

pub fn cleanup_stage_summary_ui(
    mut commands: Commands,
    summary_query: Query<Entity, With<StageSummaryUI>>,
) {
    for entity in summary_query.iter() {
        commands.entity(entity).safe_despawn();
    }
}

// ===== INTEGRATION WITH GAME SCORE =====

// Add these fields to GameScore in resources.rs
impl GameScore {
    pub fn add_stage_atp(&mut self, collected: u32, total: u32) {
        self.stage_atp_collected += collected;
        self.stage_atp_total += total;
    }

    pub fn add_score(&mut self, points: u32, apply_multiplier: bool) {
        let final_points = if apply_multiplier {
            (points as f32 * self.score_multiplier) as u32
        } else {
            points
        };
        
        self.current = self.current.saturating_add(final_points);
        self.stage_score = self.stage_score.saturating_add(final_points);
    }
    
    pub fn start_new_stage(&mut self) {
        self.stage_score = 0;
        self.stage_atp_collected = 0;
        self.stage_atp_total = 0;
    }
    
    pub fn complete_stage(&mut self, perfect: bool) {
        self.stages_completed += 1;
        if perfect {
            self.perfect_stages += 1;
        }
    }    
}

// ===== STAGE TRACKING SYSTEM =====

pub fn stage_tracking_system(
    mut stage_progress: ResMut<StageProgress>,
    mut game_score: ResMut<GameScore>,
    wave_manager: Res<WaveManager>,
    enemy_query: Query<&Enemy>,
    mut enemy_death_events: EventReader<EnemyDeathEvent>,
    mut player_damage_events: EventReader<PlayerHit>,
) {
    // Track wave progression within stage
    let current_wave_in_stage = (wave_manager.current_wave - 1) % 5 + 1;
    stage_progress.waves_in_current_stage = current_wave_in_stage;
    
    // Track enemy deaths this stage
    for event in enemy_death_events.read() {
        stage_progress.enemies_destroyed_this_stage += 1;
        game_score.stage_score += event.points_awarded;
    }
    
    // Track damage taken this stage
    for _event in player_damage_events.read() {
        stage_progress.damage_taken_this_stage = true;
    }
    
    // Update total enemies count when new enemies spawn
    let current_enemy_count = enemy_query.iter().count();
    if current_enemy_count > stage_progress.total_enemies_this_stage as usize {
        stage_progress.total_enemies_this_stage = current_enemy_count as u32;
    }
}

// ===== INFRASTRUCTURE SYSTEM =====

#[derive(Component)]
pub struct InfrastructureTarget {
    pub target_type: InfrastructureType,
    pub points_value: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum InfrastructureType {
    ToxicWastePipe,    // Polluting infrastructure
    ChemicalVat,       // Contamination source
    RadiationEmitter,  // Radiation source
    PlasticFactory,    // Microplastic producer
}

pub fn spawn_infrastructure_targets(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    stage_progress: Res<StageProgress>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
) {
    let Some(assets) = assets else { return };
    
    *spawn_timer += time.delta_secs();
    
    // Spawn infrastructure targets every 30 seconds
    if *spawn_timer >= 30.0 {
        *spawn_timer = 0.0;
        
        let infrastructure_types = [
            InfrastructureType::ToxicWastePipe,
            InfrastructureType::ChemicalVat,
            InfrastructureType::RadiationEmitter,
            InfrastructureType::PlasticFactory,
        ];
        
        let index = (stage_progress.current_stage as usize) % infrastructure_types.len();
        let infra_type = infrastructure_types[index];
        
        let (texture, color, points) = match infra_type {
            InfrastructureType::ToxicWastePipe => {
                (assets.infrastructure_pipe_texture.clone(), Color::srgb(0.6, 0.4, 0.2), 500)
            }
            InfrastructureType::ChemicalVat => {
                (assets.infrastructure_vat_texture.clone(), Color::srgb(0.8, 0.6, 0.2), 750)
            }
            InfrastructureType::RadiationEmitter => {
                (assets.infrastructure_emitter_texture.clone(), Color::srgb(0.9, 0.9, 0.3), 1000)
            }
            InfrastructureType::PlasticFactory => {
                (assets.infrastructure_factory_texture.clone(), Color::srgb(0.7, 0.7, 0.7), 1250)
            }
        };
        
        let x_pos = (time.elapsed_secs() * 150.0).sin() * 300.0;
        
        commands.spawn((
            Sprite {
                image: texture,
                color,
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            Transform::from_xyz(x_pos, 400.0, 0.0),
            InfrastructureTarget {
                target_type: infra_type,
                points_value: points,
            },
            Health(100),
            Collider { radius: 20.0 },
            // Make infrastructure move slowly downward
            Projectile {
                velocity: Vec2::new(0.0, -50.0),
                damage: 0,
                friendly: false,
                organic_trail: false,
            },
        ));
    }
}

pub fn infrastructure_destruction_system(
    mut stage_progress: ResMut<StageProgress>,
    mut game_score: ResMut<GameScore>,
    infrastructure_query: Query<&InfrastructureTarget>,
    mut infrastructure_death_events: EventReader<InfrastructureDestroyedEvent>,
) {
    // Count total infrastructure
    let total_infrastructure = infrastructure_query.iter().count() as u32;
    stage_progress.infrastructure_total = total_infrastructure;
    
    // Track destroyed infrastructure
    for event in infrastructure_death_events.read() {
        stage_progress.infrastructure_destroyed += 1;
        game_score.stage_score += event.points_awarded;
    }
}

// ===== NEW EVENTS =====

#[derive(Event)]
pub struct EnemyDeathEvent {
    pub enemy_type: String,
    pub points_awarded: u32,
    pub position: Vec3,
}

#[derive(Event)]
pub struct InfrastructureDestroyedEvent {
    pub infrastructure_type: InfrastructureType,
    pub points_awarded: u32,
    pub position: Vec3,
}



// ===== STAGE RESET SYSTEM =====

pub fn stage_reset_system(
    mut stage_progress: ResMut<StageProgress>,
    mut game_score: ResMut<GameScore>,
    mut summary_data: ResMut<StageSummaryData>,
    current_state: Res<State<GameState>>,
) {
    // Reset stage data when returning to playing from summary
    if current_state.get() == &GameState::Playing && summary_data.show_summary {
        stage_progress.waves_in_current_stage = 0;
        stage_progress.enemies_destroyed_this_stage = 0;
        stage_progress.total_enemies_this_stage = 0;
        stage_progress.damage_taken_this_stage = false;
        stage_progress.infrastructure_destroyed = 0;
        stage_progress.infrastructure_total = 0;
        
        game_score.start_new_stage();
        summary_data.show_summary = false;
    }
}

// ===== ACHIEVEMENTS INTEGRATION =====

pub fn stage_achievement_system(
    summary_data: Res<StageSummaryData>,
    mut achievement_events: EventWriter<AchievementEvent>,
    mut processed_stages: Local<HashSet<u32>>,
) {
    if summary_data.show_summary && !processed_stages.contains(&summary_data.stage_number) {
        processed_stages.insert(summary_data.stage_number);
        
        // Perfect stage achievements
        if summary_data.no_damage_taken && summary_data.atp_collected_percent >= 1.0 {
            achievement_events.write(AchievementEvent::PerfectStage {
                stage_number: summary_data.stage_number,
            });
        }
        
        // High multiplier achievements
        if summary_data.score_multiplier >= 16.0 {
            achievement_events.write(AchievementEvent::MaxMultiplier {
                stage_number: summary_data.stage_number,
            });
        }
        
        // Milestone achievements
        match summary_data.stage_number {
            5 => { achievement_events.write(AchievementEvent::FirstFiveStages); },
            10 => { achievement_events.write(AchievementEvent::FirstTenStages); },
            20 => { achievement_events.write(AchievementEvent::TwentyStages); },
            50 => { achievement_events.write(AchievementEvent::FiftyStages); },
            _ => {}
        }
        
        // Full deck achievement
        if summary_data.full_deck_bonus_applicable {
            achievement_events.write(AchievementEvent::FullDeck);
        }
    }
}

// ===== VISUAL POLISH SYSTEMS =====

pub fn stage_summary_animations(
    mut summary_query: Query<&mut Transform, With<StageSummaryUI>>,
    time: Res<Time>,
) {
    for mut transform in summary_query.iter_mut() {
        // Gentle breathing animation for the summary panel
        let scale_factor = 1.0 + (time.elapsed_secs() * 0.5).sin() * 0.02;
        transform.scale = Vec3::splat(scale_factor);
    }
}

pub fn stage_summary_input_system(
    input_manager: Res<InputManager>,
    mut summary_data: ResMut<StageSummaryData>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<IsPaused>>,
) {
    if current_state.get() != &GameState::StageSummary { return; }
    
    // Allow any input to continue
    if input_manager.just_pressed(InputAction::Shoot) || 
       input_manager.just_pressed(InputAction::EmergencySpore) ||
       input_manager.just_pressed(InputAction::Pause) {
        summary_data.show_summary = false;
        next_state.set(IsPaused::Running);

    }
}