// cosmic_ui_derive/src/lib.rs
//! Proc macro crate for compile-time UI generation

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Meta, Lit, Type};

/// Derive macro for GameHUD - generates optimized update systems
#[proc_macro_derive(GameHUD, attributes(bind, format, position, style))]
pub fn derive_game_hud(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let bindings = extract_bindings(&fields.named);
                    generate_hud_implementation(name, bindings)
                }
                _ => panic!("GameHUD can only be derived for structs with named fields"),
            }
        }
        _ => panic!("GameHUD can only be derived for structs"),
    }
}

// ===== BINDING EXTRACTION =====

struct FieldBinding {
    field_name: syn::Ident,
    field_type: syn::Type,
    binding_source: Option<syn::Path>,
    format_string: Option<String>,
    position: Option<String>,
    style_attrs: Vec<(String, String)>,
}

fn extract_bindings(fields: &syn::punctuated::Punctuated<Field, syn::Token![,]>) -> Vec<FieldBinding> {
    fields.iter().map(|field| {
        let field_name = field.ident.clone().expect("Named field required");
        let field_type = field.ty.clone();
        
        let mut binding_source = None;
        let mut format_string = None;
        let mut position = None;
        let mut style_attrs = Vec::new();
        
        // Parse attributes
        for attr in &field.attrs {
            //if let Ok(meta) = &attr.meta {
                match &attr.meta {
                    Meta::List(meta_list) if attr.path().is_ident("bind") => {
                        if let Ok(path) = syn::parse2::<syn::Path>(meta_list.tokens.clone()) {
                            binding_source = Some(path);
                        }
                    }
                    Meta::NameValue(meta_name_value) if attr.path().is_ident("format") => {
                        if let syn::Expr::Lit(syn::ExprLit { lit: Lit::Str(lit_str), .. }) = &meta_name_value.value {
                            format_string = Some(lit_str.value());
                        }
                    }
                    Meta::List(meta_list) if attr.path().is_ident("position") => {
                        // Simple position parsing - in a real implementation this would be more robust
                        position = Some(meta_list.tokens.to_string());
                    }
                    Meta::List(meta_list) if attr.path().is_ident("style") => {
                        // Simple style parsing
                        let style_tokens = meta_list.tokens.to_string();
                        style_attrs.push(("style".to_string(), style_tokens));
                    }
                    _ => {}
                }
            //}
        }
        
        FieldBinding {
            field_name,
            field_type,
            binding_source,
            format_string,
            position,
            style_attrs,
        }
    }).collect()
}

// ===== CODE GENERATION =====

fn generate_hud_implementation(name: &syn::Ident, bindings: Vec<FieldBinding>) -> TokenStream {
    let spawn_ui_body = generate_spawn_ui_method(&bindings);
    let register_systems_body = generate_register_systems_method(name, &bindings);
    let update_systems = generate_update_systems(name, &bindings);
    
    let expanded = quote! {
        impl cosmic_ui::GameHUD for #name {
            fn spawn_ui(commands: &mut Commands, font_handle: Handle<Font>) -> bevy::prelude::Entity {
                #spawn_ui_body
            }
            
            fn register_systems(app: &mut bevy::prelude::App) {
                #register_systems_body
            }
            
            fn update_bindings(&mut self, world: &bevy::prelude::World, ui_root: bevy::prelude::Entity) {
                // Generated binding updates would go here
            }
        }
        
        #update_systems
    };
    
    TokenStream::from(expanded)
}

fn generate_spawn_ui_method(bindings: &[FieldBinding]) -> proc_macro2::TokenStream {
    let widget_creations: Vec<proc_macro2::TokenStream> = bindings.iter().map(|binding| {
        let field_name = &binding.field_name;
        let widget_type = extract_widget_type(&binding.field_type);
        let position = generate_position_code(&binding.position);
        let widget_var = syn::Ident::new(&format!("widget_{}", field_name), field_name.span());
        
        match widget_type.as_str() {
            "TextDisplay" => {
                let initial_text = binding.format_string.as_deref().unwrap_or("").to_string();
                quote! {
                    let (mut builder, #widget_var) = builder.text_display(#initial_text, #position);
                }
            }
            "Counter" => {
                let format_parts = parse_counter_format(&binding.format_string);
                let (prefix, suffix) = format_parts.unwrap_or(("".to_string(), "".to_string()));
                quote! {
                    let (mut builder, #widget_var) = builder.counter(#prefix, #suffix, #position);
                }
            }
            "ProgressBar" => {
                quote! {
                    let (mut builder, #widget_var) = builder.progress_bar(#position, cosmic_ui::builder::ProgressBarConfig::biological());
                }
            }
            "StatusIndicator" => {
                quote! {
                    let states = vec![
                        cosmic_ui::widgets::StatusState {
                            text: "Normal".to_string(),
                            color: bevy::prelude::Color::WHITE,
                            animation: cosmic_ui::widgets::StatusAnimation::Static,
                        }
                    ];
                    let (mut builder, #widget_var) = builder.status_indicator(states, #position);
                }
            }
            "NotificationQueue" => {
                quote! {
                    let (mut builder, #widget_var) = builder.notification_queue(#position, 5);
                }
            }
            _ => {
                quote! {
                    let (mut builder, #widget_var) = builder.text_display("", #position);
                }
            }
        }
    }).collect();
    
    let field_assignments: Vec<proc_macro2::TokenStream> = bindings.iter().map(|binding| {
        let field_name = &binding.field_name;
        let widget_var = syn::Ident::new(&format!("widget_{}", field_name), field_name.span());
        quote! { #field_name: #widget_var, }
    }).collect();
    
    quote! {
        let mut builder = cosmic_ui::builder::WidgetBuilder::new(commands, fonts).root();
        
        #(#widget_creations)*
        
        let hud_entity = commands.spawn(Self {
            #(#field_assignments)*
        }).id();
        
        hud_entity
    }
}

fn generate_register_systems_method(_hud_name: &syn::Ident, bindings: &[FieldBinding]) -> proc_macro2::TokenStream {
    let system_names: Vec<proc_macro2::TokenStream> = bindings.iter()
        .filter_map(|binding| binding.binding_source.as_ref().map(|_| binding))
        .map(|binding| {
            let field_name = &binding.field_name;
            let system_name = syn::Ident::new(&format!("update_{}_system", field_name), field_name.span());
            quote! { Self::#system_name }
        })
        .collect();
    
    quote! {
        app.add_systems(bevy::prelude::Update, (
            #(#system_names,)*
        ));
    }
}

fn generate_update_systems(hud_name: &syn::Ident, bindings: &[FieldBinding]) -> proc_macro2::TokenStream {
    let systems: Vec<proc_macro2::TokenStream> = bindings.iter()
        .filter_map(|binding| binding.binding_source.as_ref().map(|source| (binding, source)))
        .map(|(binding, source)| {
            let field_name = &binding.field_name;
            let system_name = syn::Ident::new(&format!("update_{}_system", field_name), field_name.span());
            let widget_type = extract_widget_type(&binding.field_type);
            
            // Generate update logic based on binding source and widget type
            let (query_params, update_logic) = generate_update_logic(binding, source, &widget_type);
            
            quote! {
                fn #system_name(
                    mut scheduler: bevy::prelude::ResMut<cosmic_ui::UIUpdateScheduler>,
                    mut hud_query: bevy::prelude::Query<&mut #hud_name>,
                    #query_params
                ) {
                    #update_logic
                }
            }
        })
        .collect();
    
    quote! {
        impl #hud_name {
            #(#systems)*
        }
    }
}

fn generate_update_logic(binding: &FieldBinding, source: &syn::Path, widget_type: &str) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let field_name = &binding.field_name;
    let source_str = quote!(#source).to_string();
    
    let query_params = match source_str.as_str() {
        "PlayerLives" => quote! {
            player_query: bevy::prelude::Query<&crate::components::Player, bevy::prelude::Changed<crate::components::Player>>,
        },
        "PlayerATP" => quote! {
            atp_query: bevy::prelude::Query<&crate::components::ATP, (bevy::prelude::With<crate::components::Player>, bevy::prelude::Changed<crate::components::ATP>)>,
        },
        "PlayerHealth" => quote! {
            health_query: bevy::prelude::Query<&crate::components::Health, (bevy::prelude::With<crate::components::Player>, bevy::prelude::Changed<crate::components::Health>)>,
        },
        "GameScore" => quote! {
            score: bevy::prelude::Res<crate::resources::GameScore>,
        },
        "CellWallTimer" => quote! {
            cell_wall_query: bevy::prelude::Query<&crate::components::CellWallReinforcement, bevy::prelude::Changed<crate::components::CellWallReinforcement>>,
        },
        "EnvironmentStatus" => quote! {
            environment: bevy::prelude::Res<crate::resources::ChemicalEnvironment>,
        },
        _ => quote! {
            // Default query for unknown binding
        },
    };
    
    let update_logic = match (source_str.as_str(), widget_type) {
        ("PlayerLives", "Counter") => quote! {
            if let Ok(player) = player_query.get_single() {
                for mut hud in hud_query.iter_mut() {
                    if hud.#field_name.last_value != player.lives {
                        let text = format!("{}{}{}",
                            hud.#field_name.prefix,
                            player.lives,
                            hud.#field_name.suffix
                        );
                        scheduler.queue_update(cosmic_ui::UIUpdateCommand::TextUpdate {
                            entity: hud.#field_name.entity,
                            text,
                        });
                        hud.#field_name.last_value = player.lives;
                    }
                }
            }
        },
        ("PlayerATP", "TextDisplay") => quote! {
            if let Ok(atp) = atp_query.get_single() {
                for mut hud in hud_query.iter_mut() {
                    let new_hash = atp.amount as u64;
                    if hud.#field_name.last_value_hash != new_hash {
                        let text = format!("ATP: {}⚡", atp.amount);
                        scheduler.queue_update(cosmic_ui::UIUpdateCommand::TextUpdate {
                            entity: hud.#field_name.entity,
                            text,
                        });
                        hud.#field_name.last_value_hash = new_hash;
                    }
                }
            }
        },
        ("PlayerHealth", "ProgressBar") => quote! {
            if let Ok(health) = health_query.get_single() {
                for mut hud in hud_query.iter_mut() {
                    let percent = health.0 as f32 / 100.0;
                    if (hud.#field_name.current_percent - percent).abs() > 0.01 {
                        // Update progress bar percentage
                        hud.#field_name.current_percent = percent;
                        
                        // The progress bar animation system will handle the visual update
                    }
                }
            }
        },
        ("GameScore", "TextDisplay") => quote! {
            if score.is_changed() {
                for mut hud in hud_query.iter_mut() {
                    let text = if score.score_multiplier > 1.0 {
                        format!("Score: {} ({}x)", score.current, score.score_multiplier)
                    } else {
                        format!("Score: {}", score.current)
                    };
                    
                    scheduler.queue_update(cosmic_ui::UIUpdateCommand::TextUpdate {
                        entity: hud.#field_name.entity,
                        text,
                    });
                }
            }
        },
        ("CellWallTimer", "TextDisplay") => quote! {
            for mut hud in hud_query.iter_mut() {
                if let Ok(cell_wall) = cell_wall_query.get_single() {
                    let remaining = cell_wall.timer.max(0.0);
                    let icon = if remaining < 3.0 { "⚠️" } else { "🛡️" };
                    let text = format!("{} Cell Wall: {:.1}s", icon, remaining);
                    
                    scheduler.queue_update(cosmic_ui::UIUpdateCommand::TextUpdate {
                        entity: hud.#field_name.entity,
                        text,
                    });
                } else {
                    scheduler.queue_update(cosmic_ui::UIUpdateCommand::TextUpdate {
                        entity: hud.#field_name.entity,
                        text: String::new(),
                    });
                }
            }
        },
        ("EnvironmentStatus", "TextDisplay") => quote! {
            if environment.is_changed() {
                for mut hud in hud_query.iter_mut() {
                    let text = format!(
                        "pH: {:.1} | O2: {:.0}%",
                        environment.base_ph,
                        environment.base_oxygen * 100.0
                    );
                    
                    scheduler.queue_update(cosmic_ui::UIUpdateCommand::TextUpdate {
                        entity: hud.#field_name.entity,
                        text,
                    });
                }
            }
        },
        _ => quote! {
            // Default update logic - no-op
        },
    };
    
    (query_params, update_logic)
}

fn generate_position_code(position: &Option<String>) -> proc_macro2::TokenStream {
    match position.as_ref().map(|s| s.as_str()) {
        Some("top_left") => quote! { cosmic_ui::builder::UIPosition::top_left() },
        Some("top_right") => quote! { cosmic_ui::builder::UIPosition::top_right() },
        Some("bottom_left") => quote! { cosmic_ui::builder::UIPosition::bottom_left() },
        Some("bottom_right") => quote! { cosmic_ui::builder::UIPosition::bottom_right() },
        Some("center") => quote! { cosmic_ui::builder::UIPosition::center() },
        Some(pos) if pos.contains("offset") => {
            // Parse offset patterns like "bottom_left, offset_y = 30"
            if pos.contains("bottom_left") {
                let offset_y = extract_offset_value(pos).unwrap_or(0.0);
                quote! { cosmic_ui::builder::UIPosition::bottom_left().with_offset(0.0, #offset_y) }
            } else if pos.contains("top_left") {
                let offset_y = extract_offset_value(pos).unwrap_or(0.0);
                quote! { cosmic_ui::builder::UIPosition::top_left().with_offset(0.0, #offset_y) }
            } else {
                quote! { cosmic_ui::builder::UIPosition::default() }
            }
        },
        _ => quote! { cosmic_ui::builder::UIPosition::default() },
    }
}

fn extract_widget_type(ty: &Type) -> String {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident.to_string();
        }
    }
    "TextDisplay".to_string()
}

fn parse_counter_format(format: &Option<String>) -> Option<(String, String)> {
    format.as_ref().map(|f| {
        if let Some(pos) = f.find("{}") {
            let prefix = f[..pos].to_string();
            let suffix = f[pos + 2..].to_string();
            (prefix, suffix)
        } else {
            (f.clone(), String::new())
        }
    })
}

fn extract_offset_value(pos: &str) -> Option<f32> {
    // Simple parsing for offset_y = 30 patterns
    if let Some(start) = pos.find("offset_y = ") {
        let value_start = start + 11; // Length of "offset_y = "
        let value_str: String = pos.chars()
            .skip(value_start)
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        value_str.parse().ok()
    } else {
        None
    }
}