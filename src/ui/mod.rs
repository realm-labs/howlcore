//! Bevy UI adapter for the pure gameplay cores.

use bevy::{app::AppExit, prelude::*};

use crate::{
    app_state::AppMode,
    core::{
        battle::{
            BattleAbilityDatabase, BattleChimera, BattleDefinition, BattleEffect, BattleEvent,
            BattleLeaderEffect, BattleOutcome, BattleRunPhase, BattleRunReward, BattleRunState,
            BattleRunStep, BattleShopItem, BattleState, BattleTrigger, PurchaseOutcome, TeamSide,
            resolver::front_chimera_id,
        },
        work::{Chimera, Effect, StageDefinition, TraitDef, Trigger, WorkRunPhase, WorkRunState},
    },
};

const ROOT_BG: Color = Color::srgb(0.07, 0.075, 0.08);
const SURFACE: Color = Color::srgb(0.12, 0.135, 0.145);
const SURFACE_RAISED: Color = Color::srgb(0.155, 0.17, 0.18);
const PANEL: Color = Color::srgb(0.09, 0.1, 0.11);
const CARD: Color = Color::srgb(0.19, 0.21, 0.22);
const CARD_HOVERED: Color = Color::srgb(0.25, 0.28, 0.29);
const CARD_ACTIVE: Color = Color::srgb(0.18, 0.32, 0.27);
const CARD_ACTIVE_HOVERED: Color = Color::srgb(0.23, 0.4, 0.34);
const CARD_SELECTED: Color = Color::srgb(0.34, 0.27, 0.15);
const CARD_SELECTED_HOVERED: Color = Color::srgb(0.43, 0.34, 0.18);
const CARD_DISABLED: Color = Color::srgb(0.105, 0.115, 0.12);
const TEXT: Color = Color::srgb(0.91, 0.93, 0.92);
const MUTED: Color = Color::srgb(0.6, 0.64, 0.65);
const ACCENT: Color = Color::srgb(0.9, 0.66, 0.28);
const GOOD: Color = Color::srgb(0.31, 0.74, 0.58);
const DANGER: Color = Color::srgb(0.72, 0.28, 0.27);

type ButtonInteractionQuery<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static UiAction), (Changed<Interaction>, With<Button>)>;
type ButtonVisualQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static ButtonColors,
        &'static mut BackgroundColor,
    ),
    (Changed<Interaction>, With<Button>),
>;

#[derive(Resource)]
struct GameplayResource {
    mode: AppMode,
    work_definition: StageDefinition,
    battle_definition: BattleDefinition,
    work: WorkRunState,
    battle: BattleRunState,
}

#[derive(Resource, Default)]
struct UiLogs {
    work: Vec<String>,
    battle: Vec<String>,
    battle_pending: Vec<BattlePlaybackFrame>,
    battle_display: Option<BattleState>,
    work_offset: usize,
    battle_offset: usize,
}

#[derive(Clone)]
struct BattlePlaybackFrame {
    line: String,
    battle: Option<BattleState>,
}

#[derive(Component)]
struct RoundText;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct BoardRoot;

#[derive(Component)]
struct LogText;

#[derive(Component, Clone, Copy)]
enum UiAction {
    SwitchMode,
    Advance,
    Reset,
    SelectAlpha(usize),
    WorkSwap(usize),
    WorkToggle(usize),
    BuyShop(usize),
    RefreshShop,
    BattleSwap(usize),
    BenchLast,
    DeployFirst,
    EquipFirst,
    UnequipFirst,
}

#[derive(Component, Clone, Copy)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
}

const LOG_WINDOW_LINES: usize = 24;

pub fn build_app(stage: StageDefinition, battle: BattleDefinition) -> App {
    let work_logs = stage.initial_logs.clone();
    let battle_logs = battle.initial_logs.clone();
    let gameplay = GameplayResource {
        mode: AppMode::WorkAssignment,
        work: WorkRunState::from_stage(stage.clone()),
        battle: BattleRunState::from_definition(battle.clone()),
        work_definition: stage,
        battle_definition: battle,
    };
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Howlcore Gameplay Debugger".to_string(),
            resolution: (1280.0, 820.0).into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(gameplay)
    .insert_resource(UiLogs {
        work: work_logs,
        battle: battle_logs,
        battle_pending: Vec::new(),
        battle_display: None,
        work_offset: 0,
        battle_offset: 0,
    })
    .add_systems(Startup, setup_ui_system)
    .add_systems(
        Update,
        (
            handle_input_system,
            update_button_visual_system,
            handle_ui_action_system,
            update_round_text_system,
            update_score_text_system,
            rebuild_board_system,
            update_log_text_system,
        )
            .chain(),
    );

    app
}

fn setup_ui_system(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            background_color: ROOT_BG.into(),
            ..default()
        })
        .with_children(|root| {
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(82.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                background_color: SURFACE.into(),
                ..default()
            })
            .with_children(|header| {
                header.spawn((
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font_size: 25.0,
                            color: TEXT,
                            ..default()
                        },
                    ),
                    RoundText,
                ));
                header.spawn((
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font_size: 16.0,
                            color: MUTED,
                            ..default()
                        },
                    ),
                    ScoreText,
                ));
            });
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    min_height: Val::Px(0.0),
                    column_gap: Val::Px(12.0),
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|columns| {
                columns.spawn((
                    NodeBundle {
                        style: Style {
                            height: Val::Percent(100.0),
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        background_color: Color::NONE.into(),
                        ..default()
                    },
                    BoardRoot,
                ));
                spawn_panel(columns, "Event Log", 0.9, 0.66, 0.28, LogText);
            });
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(46.0),
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    ..default()
                },
                background_color: SURFACE.into(),
                ..default()
            })
            .with_children(|footer| {
                spawn_button(footer, "Switch Mode", UiAction::SwitchMode, Val::Px(126.0));
                spawn_button(footer, "Advance", UiAction::Advance, Val::Px(108.0));
                spawn_button(footer, "Reset", UiAction::Reset, Val::Px(88.0));
                footer.spawn(TextBundle::from_section(
                    "Tab mode   Space advance   1-3 choose/buy   R refresh   Q/W/E swap   B/V bench/deploy   Z/X equip   Arrows/Page log   Esc quit",
                    TextStyle {
                        font_size: 13.0,
                        color: MUTED,
                        ..default()
                    },
                ));
            });
        });
}

fn spawn_panel<T: Component>(
    parent: &mut ChildBuilder,
    title: &'static str,
    red: f32,
    green: f32,
    blue: f32,
    marker: T,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(360.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                flex_shrink: 0.0,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            background_color: SURFACE.into(),
            ..default()
        })
        .with_children(|panel| {
            panel.spawn(TextBundle::from_section(
                title,
                TextStyle {
                    font_size: 21.0,
                    color: Color::srgb(red, green, blue),
                    ..default()
                },
            ));
            panel.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 14.0,
                        color: TEXT,
                        ..default()
                    },
                )
                .with_style(Style {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    ..default()
                }),
                marker,
            ));
        });
}

fn spawn_button(parent: &mut ChildBuilder, label: &str, action: UiAction, width: Val) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width,
                    height: Val::Px(36.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    ..default()
                },
                background_color: SURFACE_RAISED.into(),
                ..default()
            },
            action,
            ButtonColors {
                normal: SURFACE_RAISED,
                hovered: CARD_HOVERED,
            },
        ))
        .with_children(|button| {
            button.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 14.0,
                    color: TEXT,
                    ..default()
                },
            ));
        });
}

fn handle_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut gameplay: ResMut<GameplayResource>,
    mut logs: ResMut<UiLogs>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }

    if keys.just_pressed(KeyCode::Tab) {
        gameplay.mode = match gameplay.mode {
            AppMode::WorkAssignment => AppMode::ChimeraBattle,
            AppMode::ChimeraBattle => AppMode::WorkAssignment,
        };
    }

    if keys.just_pressed(KeyCode::KeyN) {
        reset_active_mode(&mut gameplay, &mut logs);
    }

    if gameplay.mode == AppMode::ChimeraBattle {
        handle_battle_draft_input(&keys, &mut gameplay, &mut logs);
    } else {
        handle_work_prep_input(&keys, &mut gameplay, &mut logs);
    }

    if keys.just_pressed(KeyCode::Space) {
        advance_active_mode(&mut gameplay, &mut logs);
    }

    let scroll_delta = log_scroll_delta(&keys);
    if scroll_delta != 0 {
        scroll_active_log(&gameplay, &mut logs, scroll_delta);
    }

    if keys.just_pressed(KeyCode::End) {
        match gameplay.mode {
            AppMode::WorkAssignment => logs.work_offset = 0,
            AppMode::ChimeraBattle => logs.battle_offset = 0,
        }
    }
}

fn handle_ui_action_system(
    mut interactions: ButtonInteractionQuery,
    mut gameplay: ResMut<GameplayResource>,
    mut logs: ResMut<UiLogs>,
) {
    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *action {
            UiAction::SwitchMode => {
                gameplay.mode = match gameplay.mode {
                    AppMode::WorkAssignment => AppMode::ChimeraBattle,
                    AppMode::ChimeraBattle => AppMode::WorkAssignment,
                };
            }
            UiAction::Advance => advance_active_mode(&mut gameplay, &mut logs),
            UiAction::Reset => reset_active_mode(&mut gameplay, &mut logs),
            UiAction::SelectAlpha(index) => {
                match gameplay.work.select_alpha(index) {
                    Ok(alpha_name) => logs.work.push(format!("Work run: selected {alpha_name}.")),
                    Err(error) => logs.work.push(format!("Alpha selection error: {error:?}.")),
                }
                logs.work_offset = 0;
            }
            UiAction::WorkSwap(left_position) => {
                match gameplay
                    .work
                    .swap_overtime_positions(left_position, left_position + 1)
                {
                    Ok(()) => logs.work.push(format!(
                        "Overtime prep: swapped positions {} and {}.",
                        left_position + 1,
                        left_position + 2
                    )),
                    Err(error) => logs
                        .work
                        .push(format!("Overtime prep swap error: {error:?}.")),
                }
                logs.work_offset = 0;
            }
            UiAction::WorkToggle(position) => {
                match gameplay.work.toggle_overtime_chimera(position) {
                    Ok(chimera_name) => logs
                        .work
                        .push(format!("Overtime prep: toggled {chimera_name}.")),
                    Err(error) => logs
                        .work
                        .push(format!("Overtime prep toggle error: {error:?}.")),
                }
                logs.work_offset = 0;
            }
            UiAction::BuyShop(index) => {
                match gameplay.battle.draft.purchase(index) {
                    Ok(outcome) => logs.battle.push(format_purchase_outcome(outcome)),
                    Err(error) => logs
                        .battle
                        .push(format!("Draft purchase error: {error:?}.")),
                }
                logs.battle_offset = 0;
            }
            UiAction::RefreshShop => {
                match gameplay.battle.refresh_shop() {
                    Ok(()) => logs.battle.push("Draft: refreshed shop.".to_string()),
                    Err(error) => logs.battle.push(format!("Draft refresh error: {error:?}.")),
                }
                logs.battle_offset = 0;
            }
            UiAction::BattleSwap(left_position) => {
                match gameplay
                    .battle
                    .draft
                    .swap_active_positions(left_position, left_position + 1)
                {
                    Ok(()) => logs.battle.push(format!(
                        "Draft: swapped active positions {} and {}.",
                        left_position + 1,
                        left_position + 2
                    )),
                    Err(error) => logs.battle.push(format!("Draft swap error: {error:?}.")),
                }
                logs.battle_offset = 0;
            }
            UiAction::BenchLast => {
                let last_position = gameplay.battle.draft.team.chimeras.len().saturating_sub(1);
                match gameplay.battle.draft.send_active_to_bench(last_position) {
                    Ok(chimera_name) => logs
                        .battle
                        .push(format!("Draft: moved {chimera_name} to bench.")),
                    Err(error) => logs.battle.push(format!("Draft bench error: {error:?}.")),
                }
                logs.battle_offset = 0;
            }
            UiAction::DeployFirst => {
                match gameplay.battle.draft.deploy_from_bench(0) {
                    Ok(chimera_name) => logs
                        .battle
                        .push(format!("Draft: deployed {chimera_name} from bench.")),
                    Err(error) => logs.battle.push(format!("Draft deploy error: {error:?}.")),
                }
                logs.battle_offset = 0;
            }
            UiAction::EquipFirst => {
                match gameplay.battle.draft.equip_inventory_item(0, 0) {
                    Ok(outcome) => logs.battle.push(format!(
                        "Draft: equipped {} on {}.",
                        outcome.equipment_name, outcome.chimera_name
                    )),
                    Err(error) => logs.battle.push(format!("Draft equip error: {error:?}.")),
                }
                logs.battle_offset = 0;
            }
            UiAction::UnequipFirst => {
                match gameplay.battle.draft.unequip_active_item(0, 0) {
                    Ok(outcome) => logs.battle.push(format!(
                        "Draft: unequipped {} from {}.",
                        outcome.equipment_name, outcome.chimera_name
                    )),
                    Err(error) => logs.battle.push(format!("Draft unequip error: {error:?}.")),
                }
                logs.battle_offset = 0;
            }
        }
    }
}

fn update_button_visual_system(mut interactions: ButtonVisualQuery) {
    for (interaction, colors, mut background) in &mut interactions {
        background.0 = match *interaction {
            Interaction::Hovered | Interaction::Pressed => colors.hovered,
            Interaction::None => colors.normal,
        };
    }
}

fn advance_active_mode(gameplay: &mut GameplayResource, logs: &mut UiLogs) {
    match gameplay.mode {
        AppMode::WorkAssignment => {
            let outcome = gameplay.work.step();
            let was_following = logs.work_offset == 0;
            logs.work.extend(outcome.logs);
            if was_following {
                logs.work_offset = 0;
            }
        }
        AppMode::ChimeraBattle => {
            if reveal_next_battle_playback_line(logs) {
                return;
            }

            let previous_battle = gameplay.battle.battle.clone();
            let step_result = gameplay.battle.step();
            let was_following = logs.battle_offset == 0;
            match step_result {
                Ok((step, outcome)) => {
                    let playback_frames = format_battle_playback_frames(
                        step,
                        &outcome,
                        previous_battle.as_ref(),
                        gameplay.battle.battle.as_ref(),
                        &gameplay.battle.ability_database,
                    );
                    if playback_frames.is_empty() {
                        logs.battle
                            .push("Battle run: no visible event.".to_string());
                    } else {
                        logs.battle_pending = playback_frames;
                        reveal_next_battle_playback_line(logs);
                    }
                }
                Err(error) => {
                    logs.battle.push(format!("Battle run error: {error:?}."));
                }
            }
            if was_following {
                logs.battle_offset = 0;
            }
        }
    }
}

fn reveal_next_battle_playback_line(logs: &mut UiLogs) -> bool {
    if logs.battle_pending.is_empty() {
        return false;
    }

    let frame = logs.battle_pending.remove(0);
    logs.battle.push(frame.line);
    if let Some(battle) = frame.battle {
        logs.battle_display = Some(battle);
    }
    logs.battle_offset = 0;
    true
}

fn handle_work_prep_input(
    keys: &ButtonInput<KeyCode>,
    gameplay: &mut GameplayResource,
    logs: &mut UiLogs,
) {
    if gameplay.work.phase == WorkRunPhase::Review {
        for (key, index) in [
            (KeyCode::Digit1, 0),
            (KeyCode::Digit2, 1),
            (KeyCode::Digit3, 2),
        ] {
            if keys.just_pressed(key) {
                match gameplay.work.select_alpha(index) {
                    Ok(alpha_name) => logs.work.push(format!("Work run: selected {alpha_name}.")),
                    Err(error) => logs.work.push(format!("Alpha selection error: {error:?}.")),
                }
                logs.work_offset = 0;
            }
        }
    }

    if gameplay.work.phase != WorkRunPhase::OvertimePrep {
        return;
    }

    for (key, left_position) in [(KeyCode::KeyQ, 0), (KeyCode::KeyW, 1), (KeyCode::KeyE, 2)] {
        if keys.just_pressed(key) {
            match gameplay
                .work
                .swap_overtime_positions(left_position, left_position + 1)
            {
                Ok(()) => logs.work.push(format!(
                    "Overtime prep: swapped positions {} and {}.",
                    left_position + 1,
                    left_position + 2
                )),
                Err(error) => logs
                    .work
                    .push(format!("Overtime prep swap error: {error:?}.")),
            }
            logs.work_offset = 0;
        }
    }

    for (key, position) in [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
    ] {
        if keys.just_pressed(key) {
            match gameplay.work.toggle_overtime_chimera(position) {
                Ok(chimera_name) => logs
                    .work
                    .push(format!("Overtime prep: toggled {chimera_name}.")),
                Err(error) => logs
                    .work
                    .push(format!("Overtime prep toggle error: {error:?}.")),
            }
            logs.work_offset = 0;
        }
    }
}

fn handle_battle_draft_input(
    keys: &ButtonInput<KeyCode>,
    gameplay: &mut GameplayResource,
    logs: &mut UiLogs,
) {
    if gameplay.battle.phase != BattleRunPhase::Draft {
        return;
    }

    if keys.just_pressed(KeyCode::KeyR) {
        match gameplay.battle.refresh_shop() {
            Ok(()) => logs.battle.push("Draft: refreshed shop.".to_string()),
            Err(error) => logs.battle.push(format!("Draft refresh error: {error:?}.")),
        }
    }

    for (key, index) in [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
    ] {
        if keys.just_pressed(key) {
            match gameplay.battle.draft.purchase(index) {
                Ok(outcome) => logs.battle.push(format_purchase_outcome(outcome)),
                Err(error) => logs
                    .battle
                    .push(format!("Draft purchase error: {error:?}.")),
            }
            logs.battle_offset = 0;
        }
    }

    for (key, left_position) in [(KeyCode::KeyQ, 0), (KeyCode::KeyW, 1), (KeyCode::KeyE, 2)] {
        if keys.just_pressed(key) {
            match gameplay
                .battle
                .draft
                .swap_active_positions(left_position, left_position + 1)
            {
                Ok(()) => logs.battle.push(format!(
                    "Draft: swapped active positions {} and {}.",
                    left_position + 1,
                    left_position + 2
                )),
                Err(error) => logs.battle.push(format!("Draft swap error: {error:?}.")),
            }
            logs.battle_offset = 0;
        }
    }

    if keys.just_pressed(KeyCode::KeyB) {
        let last_position = gameplay.battle.draft.team.chimeras.len().saturating_sub(1);
        match gameplay.battle.draft.send_active_to_bench(last_position) {
            Ok(chimera_name) => logs
                .battle
                .push(format!("Draft: moved {chimera_name} to bench.")),
            Err(error) => logs.battle.push(format!("Draft bench error: {error:?}.")),
        }
        logs.battle_offset = 0;
    }

    if keys.just_pressed(KeyCode::KeyV) {
        match gameplay.battle.draft.deploy_from_bench(0) {
            Ok(chimera_name) => logs
                .battle
                .push(format!("Draft: deployed {chimera_name} from bench.")),
            Err(error) => logs.battle.push(format!("Draft deploy error: {error:?}.")),
        }
        logs.battle_offset = 0;
    }

    if keys.just_pressed(KeyCode::KeyZ) {
        match gameplay.battle.draft.equip_inventory_item(0, 0) {
            Ok(outcome) => logs.battle.push(format!(
                "Draft: equipped {} on {}.",
                outcome.equipment_name, outcome.chimera_name
            )),
            Err(error) => logs.battle.push(format!("Draft equip error: {error:?}.")),
        }
        logs.battle_offset = 0;
    }

    if keys.just_pressed(KeyCode::KeyX) {
        match gameplay.battle.draft.unequip_active_item(0, 0) {
            Ok(outcome) => logs.battle.push(format!(
                "Draft: unequipped {} from {}.",
                outcome.equipment_name, outcome.chimera_name
            )),
            Err(error) => logs.battle.push(format!("Draft unequip error: {error:?}.")),
        }
        logs.battle_offset = 0;
    }
}

fn reset_active_mode(gameplay: &mut GameplayResource, logs: &mut UiLogs) {
    match gameplay.mode {
        AppMode::WorkAssignment => {
            gameplay.work = WorkRunState::from_stage(gameplay.work_definition.clone());
            logs.work = gameplay.work_definition.initial_logs.clone();
            logs.work.push("Work Assignment reset.".to_string());
            logs.work_offset = 0;
        }
        AppMode::ChimeraBattle => {
            gameplay.battle = BattleRunState::from_definition(gameplay.battle_definition.clone());
            logs.battle = gameplay.battle_definition.initial_logs.clone();
            logs.battle_pending.clear();
            logs.battle_display = None;
            logs.battle.push("Chimera Battle run reset.".to_string());
            logs.battle_offset = 0;
        }
    }
}

fn update_round_text_system(
    gameplay: Res<GameplayResource>,
    logs: Res<UiLogs>,
    mut round_text: Query<&mut Text, With<RoundText>>,
) {
    set_text(&mut round_text, format_header(&gameplay, &logs));
}

fn update_score_text_system(
    gameplay: Res<GameplayResource>,
    logs: Res<UiLogs>,
    mut score_text: Query<&mut Text, With<ScoreText>>,
) {
    set_text(&mut score_text, format_score_line(&gameplay, &logs));
}

fn rebuild_board_system(
    mut commands: Commands,
    gameplay: Res<GameplayResource>,
    logs: Res<UiLogs>,
    board_root: Query<Entity, With<BoardRoot>>,
) {
    if !gameplay.is_changed() && !logs.is_changed() {
        return;
    }

    let Ok(root) = board_root.get_single() else {
        return;
    };

    commands.entity(root).despawn_descendants();
    commands
        .entity(root)
        .with_children(|root| match gameplay.mode {
            AppMode::WorkAssignment => spawn_work_board(root, &gameplay.work),
            AppMode::ChimeraBattle => spawn_battle_board(root, &gameplay.battle, &logs),
        });
}

fn update_log_text_system(
    gameplay: Res<GameplayResource>,
    logs: Res<UiLogs>,
    mut log_text: Query<&mut Text, With<LogText>>,
) {
    let active_logs = match gameplay.mode {
        AppMode::WorkAssignment => &logs.work,
        AppMode::ChimeraBattle => &logs.battle,
    };
    let offset = match gameplay.mode {
        AppMode::WorkAssignment => logs.work_offset,
        AppMode::ChimeraBattle => logs.battle_offset,
    };
    set_text(&mut log_text, format_logs(active_logs, offset));
}

fn set_text<T: Component>(query: &mut Query<&mut Text, With<T>>, value: String) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = value;
    }
}

fn spawn_work_board(parent: &mut ChildBuilder, run: &WorkRunState) {
    spawn_board_section(parent, "Work Assignment", |section| {
        section
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|row| {
                spawn_metric_card(
                    row,
                    "Mode",
                    &format!("{:?}", run.phase),
                    CARD,
                    Val::Px(140.0),
                );
                spawn_metric_card(
                    row,
                    "Rank",
                    &run.current_rank.to_string(),
                    CARD_SELECTED,
                    Val::Px(92.0),
                );
                spawn_metric_card(
                    row,
                    "Week",
                    &(run.weeks_elapsed + 1).to_string(),
                    CARD,
                    Val::Px(92.0),
                );
                spawn_metric_card(
                    row,
                    "Cookies",
                    &format!(
                        "{}/{}",
                        run.assignment.cookie_score, run.assignment.target_cookie_score
                    ),
                    CARD_ACTIVE,
                    Val::Px(132.0),
                );
                spawn_metric_card(
                    row,
                    "Alpha",
                    selected_work_alpha_name(run),
                    CARD,
                    Val::Px(170.0),
                );
            });
    });

    if run.phase == WorkRunPhase::Review && !run.alpha_options.is_empty() {
        spawn_board_section(parent, "Alpha Chimera", |section| {
            section
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        column_gap: Val::Px(8.0),
                        row_gap: Val::Px(8.0),
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|row| {
                    for (index, alpha) in run.alpha_options.iter().enumerate() {
                        let label = format!("{}\n{}", alpha.name, alpha.chimera_name);
                        let selected = run.selected_alpha == Some(index);
                        let color = if selected { CARD_SELECTED } else { CARD };
                        spawn_card_button(
                            row,
                            &label,
                            UiAction::SelectAlpha(index),
                            color,
                            hovered_card_color(color),
                            Val::Px(170.0),
                            Val::Px(72.0),
                        );
                    }
                });
        });
    }

    spawn_board_section(parent, "Tasks", |section| {
        section
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|row| {
                let mut tasks = run.assignment.tasks.iter().collect::<Vec<_>>();
                tasks.sort_by_key(|task| task.order);
                for task in tasks {
                    let progress = progress_fraction(task.progress.current, task.progress.required);
                    spawn_progress_card(
                        row,
                        &task.name,
                        &format!(
                            "{}/{}  Cost {}  Reward {}",
                            task.progress.current,
                            task.progress.required,
                            task.progress.stamina_cost,
                            task.progress.cookie_reward
                        ),
                        progress,
                        task.progress.completed,
                        if task.progress.completed {
                            CARD_DISABLED
                        } else {
                            CARD
                        },
                        Val::Px(190.0),
                    );
                }
            });
    });

    spawn_board_section(parent, "Chimera Queue", |section| {
        section
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|row| {
                let mut chimeras = run
                    .assignment
                    .chimeras
                    .iter()
                    .enumerate()
                    .collect::<Vec<_>>();
                chimeras.sort_by_key(|(_, chimera)| chimera.slot);
                for (position, (_, chimera)) in chimeras.into_iter().enumerate() {
                    let color = if !chimera.is_active {
                        CARD_DISABLED
                    } else if run.phase == WorkRunPhase::OvertimePrep {
                        CARD_SELECTED
                    } else {
                        CARD_ACTIVE
                    };
                    let label = format_work_chimera_card(run, chimera);
                    if run.phase == WorkRunPhase::OvertimePrep {
                        spawn_card_button(
                            row,
                            &label,
                            UiAction::WorkToggle(position),
                            color,
                            hovered_card_color(color),
                            Val::Px(154.0),
                            Val::Px(124.0),
                        );
                    } else {
                        spawn_card(row, &label, color, Val::Px(154.0), Val::Px(124.0));
                    }
                }
            });
    });

    if run.phase == WorkRunPhase::OvertimePrep {
        spawn_board_section(parent, "Prep Controls", |section| {
            section
                .spawn(NodeBundle {
                    style: Style {
                        column_gap: Val::Px(8.0),
                        row_gap: Val::Px(8.0),
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|row| {
                    spawn_button(row, "Swap 1-2", UiAction::WorkSwap(0), Val::Px(110.0));
                    spawn_button(row, "Swap 2-3", UiAction::WorkSwap(1), Val::Px(110.0));
                    spawn_button(row, "Swap 3-4", UiAction::WorkSwap(2), Val::Px(110.0));
                });
        });
    }
}

fn spawn_battle_board(parent: &mut ChildBuilder, run: &BattleRunState, logs: &UiLogs) {
    let display_battle = logs.battle_display.as_ref().or(run.battle.as_ref());

    match (&run.phase, display_battle) {
        (BattleRunPhase::Draft, _) => spawn_battle_draft_board(parent, run),
        (BattleRunPhase::Battle, Some(state)) => spawn_battle_combat_board(parent, state),
        (BattleRunPhase::Complete, _) => spawn_board_section(parent, "Run Complete", |section| {
            section.spawn(TextBundle::from_section(
                format_run_details(run),
                TextStyle {
                    font_size: 17.0,
                    color: TEXT,
                    ..default()
                },
            ));
        }),
        (BattleRunPhase::Battle, None) => {}
    }
}

fn spawn_battle_draft_board(parent: &mut ChildBuilder, run: &BattleRunState) {
    spawn_board_section(parent, "Draft Status", |section| {
        let opponent = run.current_opponent();
        section
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|row| {
                spawn_metric_card(row, "Leader", leader_name(run), CARD, Val::Px(150.0));
                spawn_metric_card(
                    row,
                    "Health",
                    &format!("{}/{}", run.health, run.max_health),
                    health_status_color(run.health, run.max_health),
                    Val::Px(105.0),
                );
                spawn_metric_card(
                    row,
                    "Gold",
                    &run.draft.gold.to_string(),
                    CARD_SELECTED,
                    Val::Px(90.0),
                );
                spawn_metric_card(
                    row,
                    "Next",
                    opponent
                        .map(|opponent| opponent.name.as_str())
                        .unwrap_or("None"),
                    CARD,
                    Val::Px(170.0),
                );
                spawn_metric_card(
                    row,
                    "Reward",
                    &opponent
                        .map(|opponent| format_win_rewards(&opponent.win_rewards))
                        .unwrap_or_else(|| "none".to_string()),
                    CARD,
                    Val::Px(190.0),
                );
            });
    });

    spawn_board_section(parent, "Active Lineup", |section| {
        section
            .spawn(NodeBundle {
                style: Style {
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|row| {
                let mut chimeras = run.draft.team.chimeras.iter().collect::<Vec<_>>();
                chimeras.sort_by_key(|chimera| chimera.slot);
                for chimera in chimeras {
                    let equipment = if chimera.equipment.is_empty() {
                        "Eq none".to_string()
                    } else {
                        format!("Eq {}", chimera.equipment[0].name)
                    };
                    spawn_card(
                        row,
                        &format_battle_chimera_card(
                            chimera,
                            &run.ability_database,
                            Some(&equipment),
                        ),
                        CARD_ACTIVE,
                        Val::Px(172.0),
                        Val::Px(162.0),
                    );
                }
            });
    });

    spawn_board_section(parent, "Shop", |section| {
        section
            .spawn(NodeBundle {
                style: Style {
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|row| {
                for (index, item) in run.draft.shop.iter().enumerate() {
                    spawn_card_button(
                        row,
                        &format_shop_card_item(item, &run.ability_database),
                        UiAction::BuyShop(index),
                        CARD,
                        CARD_HOVERED,
                        Val::Px(196.0),
                        Val::Px(148.0),
                    );
                }
                spawn_button(row, "Refresh", UiAction::RefreshShop, Val::Px(105.0));
            });
    });

    spawn_board_section(parent, "Bench and Equipment", |section| {
        section
            .spawn(NodeBundle {
                style: Style {
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|row| {
                spawn_button(row, "Swap 1-2", UiAction::BattleSwap(0), Val::Px(110.0));
                spawn_button(row, "Swap 2-3", UiAction::BattleSwap(1), Val::Px(110.0));
                spawn_button(row, "Bench Last", UiAction::BenchLast, Val::Px(110.0));
                spawn_button(row, "Deploy", UiAction::DeployFirst, Val::Px(92.0));
                spawn_button(row, "Equip", UiAction::EquipFirst, Val::Px(84.0));
                spawn_button(row, "Unequip", UiAction::UnequipFirst, Val::Px(100.0));
            });

        spawn_inventory_summary(section, run);
    });
}

fn spawn_battle_combat_board(parent: &mut ChildBuilder, state: &BattleState) {
    spawn_battle_team_row(parent, state, TeamSide::Defender, "Defender");
    spawn_board_section(parent, "Front Exchange", |section| {
        let challenger_front = front_chimera_id(state, TeamSide::Challenger)
            .and_then(|id| state.chimera(id))
            .map(|chimera| chimera.name.as_str())
            .unwrap_or("None");
        let defender_front = front_chimera_id(state, TeamSide::Defender)
            .and_then(|id| state.chimera(id))
            .map(|chimera| chimera.name.as_str())
            .unwrap_or("None");
        section
            .spawn(NodeBundle {
                style: Style {
                    column_gap: Val::Px(12.0),
                    row_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|row| {
                spawn_metric_card(
                    row,
                    "Defender Front",
                    defender_front,
                    CARD_SELECTED,
                    Val::Px(190.0),
                );
                row.spawn(TextBundle::from_section(
                    "vs",
                    TextStyle {
                        font_size: 26.0,
                        color: ACCENT,
                        ..default()
                    },
                ));
                spawn_metric_card(
                    row,
                    "Challenger Front",
                    challenger_front,
                    CARD_SELECTED,
                    Val::Px(190.0),
                );
            });
    });
    spawn_battle_team_row(parent, state, TeamSide::Challenger, "Challenger");
}

fn spawn_battle_team_row(
    parent: &mut ChildBuilder,
    state: &BattleState,
    side: TeamSide,
    title: &str,
) {
    let front = front_chimera_id(state, side);
    spawn_board_section(parent, title, |section| {
        section
            .spawn(NodeBundle {
                style: Style {
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|row| {
                let team = state.team(side);
                let mut chimeras = team.chimeras.iter().enumerate().collect::<Vec<_>>();
                chimeras.sort_by_key(|(_, chimera)| chimera.slot);
                for (index, chimera) in chimeras {
                    let is_front = front.is_some_and(|id| id.index == index);
                    let color = if !chimera.is_alive() {
                        CARD_DISABLED
                    } else if is_front {
                        CARD_SELECTED
                    } else {
                        CARD
                    };
                    spawn_progress_card(
                        row,
                        &chimera.name,
                        &format_battle_combat_subtitle(chimera, &state.ability_database),
                        progress_fraction(chimera.stats.hp, chimera.stats.max_hp),
                        !chimera.is_alive(),
                        color,
                        Val::Px(172.0),
                    );
                }
            });
    });
}

fn format_work_chimera_card(run: &WorkRunState, chimera: &Chimera) -> String {
    let mut lines = vec![
        chimera.name.clone(),
        format!(
            "STA {}/{}  EFF {}",
            chimera.stats.stamina, chimera.stats.max_stamina, chimera.stats.efficiency
        ),
    ];

    let mut trait_lines = chimera
        .traits
        .iter()
        .filter_map(|trait_id| run.assignment.trait_database.traits.get(trait_id))
        .map(format_work_trait_summary)
        .collect::<Vec<_>>();

    trait_lines.extend(
        chimera
            .active_effects
            .temporary_traits
            .iter()
            .filter_map(|timed_trait| {
                run.assignment
                    .trait_database
                    .traits
                    .get(&timed_trait.trait_id)
                    .map(|trait_def| {
                        format!(
                            "{} ({}r)",
                            format_work_trait_summary(trait_def),
                            timed_trait.remaining_rounds
                        )
                    })
            }),
    );

    if trait_lines.is_empty() {
        lines.push("Trait: none".to_string());
    } else {
        lines.extend(trait_lines.into_iter().take(1));
    }

    lines.join("\n")
}

fn format_work_trait_summary(trait_def: &TraitDef) -> String {
    format!(
        "{} [{}]\n{}",
        trait_def.name,
        work_trigger_label(trait_def.trigger),
        format_work_effects(&trait_def.effects)
    )
}

fn format_work_effects(effects: &[Effect]) -> String {
    effects
        .iter()
        .map(|effect| match effect {
            Effect::AdvanceTask { amount } => format!("task +{amount}"),
            Effect::AdvanceTaskByEfficiency { bonus } => format!("eff +{bonus}"),
            Effect::GainCookie { amount } => format!("cookie +{amount}"),
            Effect::AddEfficiency { amount, duration } => format!("EFF {amount:+}/{duration}r"),
            Effect::AddStamina { amount } => format!("STA +{amount}"),
            Effect::ConsumeStamina { amount } => format!("STA -{amount}"),
            Effect::RestoreStamina { amount } => format!("STA +{amount}"),
            Effect::AddTemporaryTrait { duration, .. } => format!("trait {duration}r"),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn work_trigger_label(trigger: Trigger) -> &'static str {
    match trigger {
        Trigger::WorkStart => "work start",
        Trigger::RoundStart => "round",
        Trigger::OnWork => "on work",
        Trigger::AfterWork => "after",
        Trigger::RoundEnd => "round end",
        Trigger::TaskCompleted => "done",
    }
}

fn format_battle_chimera_card(
    chimera: &BattleChimera,
    abilities: &BattleAbilityDatabase,
    note: Option<&str>,
) -> String {
    let mut lines = vec![
        format!("{}  #{}", chimera.name, chimera.slot + 1),
        format!(
            "HP {}/{}  ATK {}  Lv{}",
            chimera.stats.hp, chimera.stats.max_hp, chimera.stats.attack, chimera.level
        ),
    ];

    if let Some(note) = note {
        lines.push(note.to_string());
    }

    let ability_lines = format_battle_ability_lines(&chimera.abilities, abilities);
    if ability_lines.is_empty() {
        lines.push("Ability: none".to_string());
    } else {
        lines.extend(ability_lines.into_iter().take(2));
    }

    lines.join("\n")
}

fn format_battle_combat_subtitle(
    chimera: &BattleChimera,
    abilities: &BattleAbilityDatabase,
) -> String {
    let ability_lines = format_battle_ability_lines(&chimera.abilities, abilities);
    let ability_text = if ability_lines.is_empty() {
        "Ability: none".to_string()
    } else {
        ability_lines
            .into_iter()
            .take(2)
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "HP {}/{}  ATK {}  Lv{}\n{}",
        chimera.stats.hp, chimera.stats.max_hp, chimera.stats.attack, chimera.level, ability_text
    )
}

fn format_battle_ability_lines(
    ability_ids: &[crate::core::battle::BattleAbilityId],
    abilities: &BattleAbilityDatabase,
) -> Vec<String> {
    ability_ids
        .iter()
        .filter_map(|ability_id| abilities.abilities.get(ability_id))
        .map(|ability| {
            format!(
                "{} [{}] {}",
                ability.name,
                battle_trigger_label(ability.trigger),
                format_battle_effects(&ability.effects)
            )
        })
        .collect()
}

fn format_battle_effects(effects: &[BattleEffect]) -> String {
    effects
        .iter()
        .map(|effect| match effect {
            BattleEffect::Chance { percent, effects } => {
                format!("{percent}% {}", format_battle_effects(effects))
            }
            BattleEffect::DealDamage { amount } => format!("dmg {amount}"),
            BattleEffect::DealAttackDamagePercent { percent, minimum } => {
                format!("{percent}% ATK dmg min {minimum}")
            }
            BattleEffect::Heal { amount } => format!("heal {amount}"),
            BattleEffect::AddAttack { amount } => format!("ATK +{amount}"),
            BattleEffect::ReduceIncomingDamage { amount, minimum } => {
                format!("dmg -{amount} min {minimum}")
            }
            BattleEffect::SwapWithTarget => "swap".to_string(),
            BattleEffect::QueueSummon { name, .. } => format!("summon {name}"),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn battle_trigger_label(trigger: BattleTrigger) -> &'static str {
    match trigger {
        BattleTrigger::BattleStart => "battle start",
        BattleTrigger::TurnStart => "turn start",
        BattleTrigger::BeforeDamageTaken => "before hit",
        BattleTrigger::AfterDamageTaken => "after hit",
        BattleTrigger::OnAllyAttack => "ally attack",
        BattleTrigger::AfterAttack => "after attack",
        BattleTrigger::OnAllyAheadDamaged => "ally ahead hit",
        BattleTrigger::OnSummon => "on summon",
        BattleTrigger::OnKnockdown => "knockdown",
    }
}

fn spawn_board_section(
    parent: &mut ChildBuilder,
    title: &str,
    children: impl FnOnce(&mut ChildBuilder),
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(11.0)),
                row_gap: Val::Px(9.0),
                ..default()
            },
            background_color: SURFACE_RAISED.into(),
            ..default()
        })
        .with_children(|section| {
            section.spawn(TextBundle::from_section(
                title,
                TextStyle {
                    font_size: 18.0,
                    color: ACCENT,
                    ..default()
                },
            ));
            children(section);
        });
}

fn spawn_card(parent: &mut ChildBuilder, label: &str, color: Color, width: Val, height: Val) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width,
                height,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: color.into(),
            ..default()
        })
        .with_children(|card| {
            card.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 13.5,
                    color: TEXT,
                    ..default()
                },
            ));
        });
}

fn spawn_metric_card(
    parent: &mut ChildBuilder,
    label: &str,
    value: &str,
    color: Color,
    width: Val,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width,
                min_height: Val::Px(58.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                padding: UiRect::horizontal(Val::Px(10.0)),
                row_gap: Val::Px(3.0),
                ..default()
            },
            background_color: color.into(),
            ..default()
        })
        .with_children(|card| {
            card.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 12.0,
                    color: MUTED,
                    ..default()
                },
            ));
            card.spawn(TextBundle::from_section(
                value,
                TextStyle {
                    font_size: 16.0,
                    color: TEXT,
                    ..default()
                },
            ));
        });
}

fn spawn_inventory_summary(parent: &mut ChildBuilder, run: &BattleRunState) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                column_gap: Val::Px(8.0),
                row_gap: Val::Px(8.0),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|row| {
            for chimera in &run.draft.bench {
                spawn_card(
                    row,
                    &format_battle_chimera_card(chimera, &run.ability_database, Some("Bench")),
                    CARD,
                    Val::Px(190.0),
                    Val::Px(142.0),
                );
            }
            for equipment in &run.draft.equipment_inventory {
                spawn_card(
                    row,
                    &format!(
                        "{}\nEquipment\nATK +{}  HP +{}",
                        equipment.name, equipment.attack, equipment.hp
                    ),
                    CARD_SELECTED,
                    Val::Px(145.0),
                    Val::Px(82.0),
                );
            }
            if run.draft.bench.is_empty() && run.draft.equipment_inventory.is_empty() {
                spawn_card(row, "Empty", CARD_DISABLED, Val::Px(120.0), Val::Px(60.0));
            }
        });
}

fn spawn_card_button(
    parent: &mut ChildBuilder,
    label: &str,
    action: UiAction,
    color: Color,
    hovered: Color,
    width: Val,
    height: Val,
) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width,
                    height,
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                background_color: color.into(),
                ..default()
            },
            action,
            ButtonColors {
                normal: color,
                hovered,
            },
        ))
        .with_children(|card| {
            card.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 13.5,
                    color: TEXT,
                    ..default()
                },
            ));
        });
}

fn spawn_progress_card(
    parent: &mut ChildBuilder,
    title: &str,
    subtitle: &str,
    progress: f32,
    completed: bool,
    color: Color,
    width: Val,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width,
                height: Val::Px(118.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(6.0),
                ..default()
            },
            background_color: color.into(),
            ..default()
        })
        .with_children(|card| {
            card.spawn(TextBundle::from_section(
                title,
                TextStyle {
                    font_size: 15.0,
                    color: TEXT,
                    ..default()
                },
            ));
            card.spawn(TextBundle::from_section(
                subtitle,
                TextStyle {
                    font_size: 12.0,
                    color: MUTED,
                    ..default()
                },
            ));
            card.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                background_color: PANEL.into(),
                ..default()
            })
            .with_children(|bar| {
                bar.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent((progress * 100.0).clamp(0.0, 100.0)),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: if completed { GOOD } else { ACCENT }.into(),
                    ..default()
                });
            });
        });
}

fn progress_fraction(current: i32, required: i32) -> f32 {
    if required <= 0 {
        1.0
    } else {
        current.max(0) as f32 / required as f32
    }
}

fn hovered_card_color(color: Color) -> Color {
    if color == CARD_ACTIVE {
        CARD_ACTIVE_HOVERED
    } else if color == CARD_SELECTED {
        CARD_SELECTED_HOVERED
    } else {
        CARD_HOVERED
    }
}

fn health_status_color(current: i32, max: i32) -> Color {
    if max <= 0 {
        return CARD;
    }

    let fraction = current as f32 / max as f32;
    if fraction <= 0.34 {
        DANGER
    } else if fraction <= 0.67 {
        CARD_SELECTED
    } else {
        CARD_ACTIVE
    }
}

fn log_scroll_delta(keys: &ButtonInput<KeyCode>) -> isize {
    let mut delta = 0;
    if keys.just_pressed(KeyCode::ArrowUp) {
        delta += 1;
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        delta -= 1;
    }
    if keys.just_pressed(KeyCode::PageUp) {
        delta += LOG_WINDOW_LINES as isize;
    }
    if keys.just_pressed(KeyCode::PageDown) {
        delta -= LOG_WINDOW_LINES as isize;
    }
    delta
}

fn scroll_active_log(gameplay: &GameplayResource, logs: &mut UiLogs, delta: isize) {
    let (entries, offset) = match gameplay.mode {
        AppMode::WorkAssignment => (&logs.work, &mut logs.work_offset),
        AppMode::ChimeraBattle => (&logs.battle, &mut logs.battle_offset),
    };
    let max_offset = entries.len().saturating_sub(LOG_WINDOW_LINES);

    if delta.is_positive() {
        *offset = (*offset + delta as usize).min(max_offset);
    } else {
        *offset = offset.saturating_sub(delta.unsigned_abs());
    }
}

fn format_battle_run_step(step: BattleRunStep) -> Vec<String> {
    match step {
        BattleRunStep::StartedBattle => vec!["Battle run: started battle.".to_string()],
        BattleRunStep::AdvancedBattle => Vec::new(),
        BattleRunStep::BattleResolved { winner } => {
            let result = winner
                .map(|side| format!("{} wins", side_label(side)))
                .unwrap_or_else(|| "draw".to_string());
            vec![format!("Battle run: resolved battle, {result}.")]
        }
    }
}

fn format_battle_playback_frames(
    step: BattleRunStep,
    outcome: &BattleOutcome,
    previous_state: Option<&BattleState>,
    current_state: Option<&BattleState>,
    abilities: &BattleAbilityDatabase,
) -> Vec<BattlePlaybackFrame> {
    let mut trigger_groups = Vec::<Vec<String>>::new();

    for event in &outcome.events {
        match event {
            BattleEvent::AbilityTriggered { source, ability } => {
                let source_name = previous_state
                    .and_then(|state| state.chimera(*source))
                    .or_else(|| current_state.and_then(|state| state.chimera(*source)))
                    .map(|chimera| chimera.name.as_str())
                    .unwrap_or("Unknown");
                let ability_text = abilities
                    .abilities
                    .get(ability)
                    .map(|ability| {
                        format!(
                            "{} [{}]",
                            ability.name,
                            battle_trigger_label(ability.trigger)
                        )
                    })
                    .unwrap_or_else(|| ability.0.to_string());

                trigger_groups.push(vec![format!(
                    "Trigger: {} {} -> {}",
                    side_label(source.side),
                    source_name,
                    ability_text
                )]);
            }
            BattleEvent::ChanceRolled {
                percent,
                roll,
                success,
            } => {
                if let Some(group) = trigger_groups.last_mut() {
                    group.push(format!(
                        "Trigger roll: {percent}% rolled {roll} => {}",
                        if *success { "success" } else { "miss" }
                    ));
                }
            }
            _ => {}
        }
    }

    let mut frames = format_battle_run_step(step)
        .into_iter()
        .map(|line| BattlePlaybackFrame {
            line,
            battle: current_state.cloned().or_else(|| previous_state.cloned()),
        })
        .collect::<Vec<_>>();

    let mut visual = previous_state.cloned().or_else(|| current_state.cloned());
    let mut event_index = 0;
    for line in &outcome.logs {
        if line.contains(" used ") && !trigger_groups.is_empty() {
            frames.extend(
                trigger_groups
                    .remove(0)
                    .into_iter()
                    .map(|line| BattlePlaybackFrame {
                        line,
                        battle: visual.clone(),
                    }),
            );
        } else {
            advance_visual_battle_for_log(
                &mut visual,
                &outcome.events,
                &mut event_index,
                line,
                current_state,
            );
            frames.push(BattlePlaybackFrame {
                line: line.clone(),
                battle: visual.clone(),
            });
        }
    }

    for group in trigger_groups {
        frames.extend(group.into_iter().map(|line| BattlePlaybackFrame {
            line,
            battle: visual.clone(),
        }));
    }

    frames
}

fn advance_visual_battle_for_log(
    visual: &mut Option<BattleState>,
    events: &[BattleEvent],
    event_index: &mut usize,
    line: &str,
    current_state: Option<&BattleState>,
) {
    let wanted = if line.contains("Turn started") {
        Some("turn")
    } else if line.contains(" took ") && line.contains(" damage") {
        Some("damage")
    } else if line.contains("restored") {
        Some("heal")
    } else if line.contains("attack changed") {
        Some("attack")
    } else if line.contains("swapped positions") {
        Some("swap")
    } else if line.contains("joined") {
        Some("summon")
    } else if line.contains("Battle ended") {
        Some("end")
    } else {
        None
    };

    let Some(wanted) = wanted else {
        return;
    };

    while *event_index < events.len() {
        let event = &events[*event_index];
        *event_index += 1;
        if visual_event_kind(event) == Some(wanted) {
            apply_visual_battle_event(visual, event, current_state);
            break;
        }
    }
}

fn visual_event_kind(event: &BattleEvent) -> Option<&'static str> {
    match event {
        BattleEvent::TurnStarted { .. } => Some("turn"),
        BattleEvent::DamageDealt { .. } => Some("damage"),
        BattleEvent::HpRestored { .. } => Some("heal"),
        BattleEvent::AttackChanged { .. } => Some("attack"),
        BattleEvent::PositionSwapped { .. } => Some("swap"),
        BattleEvent::ChimeraSummoned { .. } => Some("summon"),
        BattleEvent::BattleEnded { .. } => Some("end"),
        _ => None,
    }
}

fn apply_visual_battle_event(
    visual: &mut Option<BattleState>,
    event: &BattleEvent,
    current_state: Option<&BattleState>,
) {
    let Some(state) = visual else {
        return;
    };

    match event {
        BattleEvent::TurnStarted { turn } => state.turn = *turn,
        BattleEvent::DamageDealt { target, amount } => {
            if let Some(chimera) = state.chimera_mut(*target) {
                chimera.stats.hp = (chimera.stats.hp - amount).max(0);
            }
        }
        BattleEvent::HpRestored { target, amount } => {
            if let Some(chimera) = state.chimera_mut(*target) {
                chimera.stats.hp = (chimera.stats.hp + amount).min(chimera.stats.max_hp);
            }
        }
        BattleEvent::AttackChanged { target, amount } => {
            if let Some(chimera) = state.chimera_mut(*target) {
                chimera.stats.attack = (chimera.stats.attack + amount).max(0);
            }
        }
        BattleEvent::PositionSwapped { first, second } => {
            if first.side == second.side {
                let team = state.team_mut(first.side);
                if first.index < team.chimeras.len() && second.index < team.chimeras.len() {
                    let first_slot = team.chimeras[first.index].slot;
                    team.chimeras[first.index].slot = team.chimeras[second.index].slot;
                    team.chimeras[second.index].slot = first_slot;
                }
            }
        }
        BattleEvent::ChimeraSummoned { chimera } => {
            if state.chimera(*chimera).is_none()
                && let Some(source) = current_state.and_then(|current| current.chimera(*chimera))
            {
                state.team_mut(chimera.side).chimeras.push(source.clone());
            }
        }
        BattleEvent::BattleEnded { winner } => {
            state.is_finished = true;
            state.winner = *winner;
        }
        _ => {}
    }
}

fn format_purchase_outcome(outcome: PurchaseOutcome) -> String {
    match outcome {
        PurchaseOutcome::Added { chimera_name } => format!("Draft: bought {chimera_name}."),
        PurchaseOutcome::AddedToBench { chimera_name } => {
            format!("Draft: bought {chimera_name} to bench.")
        }
        PurchaseOutcome::EquipmentStored { equipment_name } => {
            format!("Draft: bought equipment {equipment_name}.")
        }
        PurchaseOutcome::Merged {
            chimera_name,
            level_before,
            level_after,
        } => {
            if level_before == level_after {
                format!("Draft: merged {chimera_name}.")
            } else {
                format!("Draft: merged {chimera_name}, level {level_before} -> {level_after}.")
            }
        }
    }
}

fn format_header(gameplay: &GameplayResource, logs: &UiLogs) -> String {
    match gameplay.mode {
        AppMode::WorkAssignment => {
            let run = &gameplay.work;
            let state = &run.assignment;
            format!(
                "Work Assignment - {:?} - Round {}/{}{}",
                run.phase,
                state.round,
                state.max_round,
                if run.phase == WorkRunPhase::Complete {
                    " - Complete"
                } else if run.phase == WorkRunPhase::OvertimePrep {
                    " - Prep"
                } else if state.is_finished {
                    " - Review"
                } else {
                    ""
                }
            )
        }
        AppMode::ChimeraBattle => {
            let run = &gameplay.battle;
            let display_battle = logs.battle_display.as_ref().or(run.battle.as_ref());
            match (&run.phase, display_battle) {
                (BattleRunPhase::Battle, Some(state)) => format!(
                    "Chimera Battle - Turn {}/{}{}",
                    state.turn,
                    state.max_turn,
                    if state.is_finished { " - Finished" } else { "" }
                ),
                (BattleRunPhase::Draft, _) => {
                    format!(
                        "Chimera Battle - Draft before battle {}",
                        run.battle_index + 1
                    )
                }
                (BattleRunPhase::Complete, _) => "Chimera Battle - Run Complete".to_string(),
                (BattleRunPhase::Battle, None) => "Chimera Battle - Battle".to_string(),
            }
        }
    }
}

fn format_score_line(gameplay: &GameplayResource, logs: &UiLogs) -> String {
    match gameplay.mode {
        AppMode::WorkAssignment => {
            let run = &gameplay.work;
            let state = &run.assignment;
            format!(
                "Rank: {}    Week: {}    Awoo Cookies: {} / {}    Total: {}    Overtime: {}    Completed Tasks: {} / {}",
                run.current_rank,
                run.weeks_elapsed + 1,
                state.cookie_score,
                state.target_cookie_score,
                run.total_cookies,
                run.overtime_cookies,
                state.completed_tasks,
                state.tasks.len()
            )
        }
        AppMode::ChimeraBattle => {
            let run = &gameplay.battle;
            match logs.battle_display.as_ref().or(run.battle.as_ref()) {
                Some(state) => format_battle_score(state, run),
                None => format!(
                    "Phase: {:?}    Leader: {}    Health: {}/{}    Gold: {}    Wins: {}    Losses: {}    Battle: {}/{}",
                    run.phase,
                    leader_name(run),
                    run.health,
                    run.max_health,
                    run.draft.gold,
                    run.wins,
                    run.losses,
                    run.battle_index + usize::from(run.phase != BattleRunPhase::Complete),
                    run.opponents.len()
                ),
            }
        }
    }
}

fn format_battle_score(state: &BattleState, run: &BattleRunState) -> String {
    let challenger_alive = state
        .challenger
        .chimeras
        .iter()
        .filter(|chimera| chimera.is_alive())
        .count();
    let defender_alive = state
        .defender
        .chimeras
        .iter()
        .filter(|chimera| chimera.is_alive())
        .count();
    let winner = state
        .winner
        .map(|side| format!("    Winner: {}", side_label(side)))
        .unwrap_or_default();
    format!(
        "{}: {}/{} alive    {}: {}/{} alive    Leader: {}    Health: {}/{}    Gold: {}    Wins: {}{}",
        state.challenger.name,
        challenger_alive,
        state.challenger.chimeras.len(),
        state.defender.name,
        defender_alive,
        state.defender.chimeras.len(),
        leader_name(run),
        run.health,
        run.max_health,
        run.draft.gold,
        run.wins,
        winner
    )
}

fn selected_work_alpha_name(run: &WorkRunState) -> &str {
    run.selected_alpha()
        .map(|alpha| alpha.name.as_str())
        .unwrap_or("None")
}

fn format_battle_details(state: &BattleState) -> String {
    let challenger_front = front_chimera_id(state, TeamSide::Challenger)
        .and_then(|id| state.chimera(id))
        .map(|chimera| chimera.name.as_str())
        .unwrap_or("None");
    let defender_front = front_chimera_id(state, TeamSide::Defender)
        .and_then(|id| state.chimera(id))
        .map(|chimera| chimera.name.as_str())
        .unwrap_or("None");
    let challenger_queue = format_summon_queue(state, TeamSide::Challenger);
    let defender_queue = format_summon_queue(state, TeamSide::Defender);

    format!(
        "Front Line\n  Challenger: {challenger_front}\n  Defender: {defender_front}\n\nSummon Queue\n  Challenger: {challenger_queue}\n  Defender: {defender_queue}\n\nBattle state uses deterministic RNG for replayable tests."
    )
}

fn format_run_details(run: &BattleRunState) -> String {
    match (&run.phase, &run.battle) {
        (BattleRunPhase::Battle, Some(state)) => format_battle_details(state),
        (BattleRunPhase::Draft, _) => {
            let shop = if run.draft.shop.is_empty() {
                "empty".to_string()
            } else {
                run.draft
                    .shop
                    .iter()
                    .enumerate()
                    .map(|(index, item)| format!("{}. {}", index + 1, format_shop_item(item)))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let opponent = run.current_opponent();
            let opponent_name = opponent
                .map(|opponent| opponent.name.as_str())
                .unwrap_or("None");
            let opponent_kind = opponent
                .map(|opponent| if opponent.is_boss { "boss" } else { "normal" })
                .unwrap_or("none");
            let win_rewards = opponent
                .map(|opponent| format_win_rewards(&opponent.win_rewards))
                .unwrap_or_else(|| "none".to_string());
            let loss_damage = opponent
                .map(|opponent| opponent.loss_health_damage)
                .unwrap_or(run.loss_health_damage);
            format!(
                "Draft\n  Leader: {}\n  Health: {}/{}\n  Gold: {}\n  Active lineup: {}/{}\n  Bench: {}\n  Next opponent: {opponent_name} ({opponent_kind})\n  Win rewards: {win_rewards}\n  Loss damage: {loss_damage} health\n  Press Space to start battle.\n\nLeader Effects\n{}\n\nShop\n{shop}",
                leader_name(run),
                run.health,
                run.max_health,
                run.draft.gold,
                run.draft.team.chimeras.len(),
                run.draft.active_team_limit,
                run.draft.bench.len(),
                format_leader_effects(run)
            )
        }
        (BattleRunPhase::Complete, _) => format!(
            "Run complete.\nResult: {}\nWins: {}\nLosses: {}\nHealth: {}/{}\nGold: {}",
            run.result
                .map(|result| format!("{result:?}"))
                .unwrap_or_else(|| "Unknown".to_string()),
            run.wins,
            run.losses,
            run.health,
            run.max_health,
            run.draft.gold
        ),
        (BattleRunPhase::Battle, None) => "Battle is preparing.".to_string(),
    }
}

fn format_shop_item(item: &BattleShopItem) -> String {
    let tags = if item.tags().is_empty() {
        "none".to_string()
    } else {
        item.tags().join(", ")
    };
    match item {
        BattleShopItem::Chimera(offer) => format!(
            "{}  Chimera  ATK {}  HP {}  {:?}  Tags: {}",
            offer.name, offer.attack, offer.hp, offer.rarity, tags
        ),
        BattleShopItem::Equipment(offer) => format!(
            "{}  Equipment  ATK +{}  HP +{}  {:?}  Tags: {}",
            offer.name, offer.attack, offer.hp, offer.rarity, tags
        ),
    }
}

fn format_shop_card_item(item: &BattleShopItem, abilities: &BattleAbilityDatabase) -> String {
    match item {
        BattleShopItem::Chimera(offer) => {
            let ability_text = format_battle_ability_lines(&offer.abilities, abilities)
                .into_iter()
                .take(1)
                .next()
                .unwrap_or_else(|| "Ability: none".to_string());
            format!(
                "{}\nChimera  Cost {}\nATK {}  HP {}\n{:?}\n{}",
                offer.name,
                item.cost(),
                offer.attack,
                offer.hp,
                offer.rarity,
                ability_text
            )
        }
        BattleShopItem::Equipment(offer) => {
            format!(
                "{}\nEquipment  Cost {}\nATK +{}  HP +{}\n{:?}",
                offer.name,
                item.cost(),
                offer.attack,
                offer.hp,
                offer.rarity
            )
        }
    }
}

fn format_win_rewards(rewards: &[BattleRunReward]) -> String {
    if rewards.is_empty() {
        return "none".to_string();
    }

    rewards
        .iter()
        .map(|reward| match reward {
            BattleRunReward::AddGold { amount } => format!("{amount} gold"),
            BattleRunReward::HealRun { amount } => format!("heal {amount} health"),
            BattleRunReward::AddShopItem { item } => format!("shop item {}", item.name()),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn leader_name(run: &BattleRunState) -> &str {
    run.leader
        .as_ref()
        .map(|leader| leader.name.as_str())
        .unwrap_or("None")
}

fn format_leader_effects(run: &BattleRunState) -> String {
    let Some(leader) = &run.leader else {
        return "none".to_string();
    };

    if leader.effects.is_empty() {
        let shop_bias = format_leader_shop_bias(leader);
        return if shop_bias.is_empty() {
            "none".to_string()
        } else {
            shop_bias
        };
    }

    let mut lines = Vec::new();
    let shop_bias = format_leader_shop_bias(leader);
    if !shop_bias.is_empty() {
        lines.push(shop_bias);
    }
    lines.extend(leader.effects.iter().map(|effect| match effect {
        BattleLeaderEffect::AddStartingGold { amount } => {
            format!("  Starting gold {amount:+}")
        }
        BattleLeaderEffect::AddRunHealth { amount } => {
            format!("  Run health {amount:+}")
        }
        BattleLeaderEffect::AddWinGoldReward { amount } => {
            format!("  Win reward {amount:+} gold")
        }
        BattleLeaderEffect::AddTeamStats { attack, hp } => {
            format!("  Team ATK {attack:+}, HP {hp:+}")
        }
        BattleLeaderEffect::AddShopOfferStats { attack, hp } => {
            format!("  Shop offers ATK {attack:+}, HP {hp:+}")
        }
    }));
    lines.join("\n")
}

fn format_leader_shop_bias(leader: &crate::core::battle::BattleLeader) -> String {
    if leader.preferred_shop_tags.is_empty() || leader.shop_bias_every == 0 {
        return String::new();
    }

    format!(
        "  Shop bias every {} item(s): {}",
        leader.shop_bias_every,
        leader.preferred_shop_tags.join(", ")
    )
}

fn format_summon_queue(state: &BattleState, side: TeamSide) -> String {
    let team = state.team(side);
    if team.summon_queue.is_empty() {
        "empty".to_string()
    } else {
        team.summon_queue
            .iter()
            .map(|chimera| chimera.name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn side_label(side: TeamSide) -> &'static str {
    match side {
        TeamSide::Challenger => "Challenger",
        TeamSide::Defender => "Defender",
    }
}

fn format_logs(logs: &[String], offset_from_latest: usize) -> String {
    if logs.is_empty() {
        return "No events yet.".to_string();
    }

    let max_offset = logs.len().saturating_sub(LOG_WINDOW_LINES);
    let offset = offset_from_latest.min(max_offset);
    let end = logs.len() - offset;
    let start = end.saturating_sub(LOG_WINDOW_LINES);
    let mut lines = logs[start..end]
        .iter()
        .enumerate()
        .map(|(index, line)| format_log_line(start + index + 1, line))
        .collect::<Vec<_>>();

    if offset > 0 {
        lines.push(format!(
            "[{} newer line(s) below - End jumps to latest]",
            offset
        ));
    }

    lines.join("\n")
}

fn format_log_line(index: usize, line: &str) -> String {
    let marker = if line.starts_with("Trigger:") {
        ">>"
    } else if line.starts_with("Trigger roll:") {
        "??"
    } else if line.contains(" used ") {
        "**"
    } else {
        "  "
    };

    format!("{index:>3} {marker} {line}")
}
