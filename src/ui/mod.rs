//! Bevy UI adapter for the pure gameplay cores.

use bevy::{app::AppExit, prelude::*};

use crate::{
    app_state::AppMode,
    core::{
        battle::{
            BattleDefinition, BattleLeaderEffect, BattleRunPhase, BattleRunReward, BattleRunState,
            BattleRunStep, BattleShopItem, BattleState, PurchaseOutcome, TeamSide,
            resolver::front_chimera_id,
        },
        work::{CombatState, StageDefinition, WorkRunPhase, WorkRunState},
    },
};

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
    work_offset: usize,
    battle_offset: usize,
}

#[derive(Component)]
struct RoundText;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct ChimeraText;

#[derive(Component)]
struct TaskText;

#[derive(Component)]
struct LogText;

const LOG_WINDOW_LINES: usize = 18;

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
            resolution: (1180.0, 760.0).into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(gameplay)
    .insert_resource(UiLogs {
        work: work_logs,
        battle: battle_logs,
        work_offset: 0,
        battle_offset: 0,
    })
    .add_systems(Startup, setup_ui_system)
    .add_systems(
        Update,
        (
            handle_input_system,
            update_round_text_system,
            update_score_text_system,
            update_chimera_text_system,
            update_task_text_system,
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
                padding: UiRect::all(Val::Px(18.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            background_color: Color::srgb(0.08, 0.09, 0.1).into(),
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 28.0,
                        color: Color::srgb(0.92, 0.94, 0.96),
                        ..default()
                    },
                ),
                RoundText,
            ));
            root.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 22.0,
                        color: Color::srgb(0.78, 0.86, 0.94),
                        ..default()
                    },
                ),
                ScoreText,
            ));
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    column_gap: Val::Px(14.0),
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|columns| {
                spawn_panel(columns, "Chimeras", 0.92, 0.86, 0.68, ChimeraText);
                spawn_panel(columns, "Details", 0.72, 0.9, 0.76, TaskText);
                spawn_panel(columns, "Log", 0.9, 0.82, 0.72, LogText);
            });
            root.spawn(TextBundle::from_section(
                "Tab: switch mode    Space: advance/start battle    1-3: buy    R: refresh shop    Q/W/E: swap lineup    B/V: bench/deploy    Z/X: equip/unequip    N: reset    Up/Down/Page: scroll log    End: latest    Esc: quit",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.58, 0.62, 0.66),
                    ..default()
                },
            ));
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
                width: Val::Percent(33.33),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            background_color: Color::srgb(0.13, 0.15, 0.17).into(),
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
                        font_size: 17.0,
                        color: Color::srgb(0.86, 0.89, 0.91),
                        ..default()
                    },
                )
                .with_style(Style {
                    width: Val::Percent(100.0),
                    ..default()
                }),
                marker,
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
                let step_result = gameplay.battle.step();
                let was_following = logs.battle_offset == 0;
                match step_result {
                    Ok((step, outcome)) => {
                        logs.battle.extend(format_battle_run_step(step));
                        logs.battle.extend(outcome.logs);
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

fn handle_work_prep_input(
    keys: &ButtonInput<KeyCode>,
    gameplay: &mut GameplayResource,
    logs: &mut UiLogs,
) {
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
            logs.battle.push("Chimera Battle run reset.".to_string());
            logs.battle_offset = 0;
        }
    }
}

fn update_round_text_system(
    gameplay: Res<GameplayResource>,
    mut round_text: Query<&mut Text, With<RoundText>>,
) {
    set_text(&mut round_text, format_header(&gameplay));
}

fn update_score_text_system(
    gameplay: Res<GameplayResource>,
    mut score_text: Query<&mut Text, With<ScoreText>>,
) {
    set_text(&mut score_text, format_score_line(&gameplay));
}

fn update_chimera_text_system(
    gameplay: Res<GameplayResource>,
    mut chimera_text: Query<&mut Text, With<ChimeraText>>,
) {
    set_text(&mut chimera_text, format_chimeras(&gameplay));
}

fn update_task_text_system(
    gameplay: Res<GameplayResource>,
    mut task_text: Query<&mut Text, With<TaskText>>,
) {
    set_text(&mut task_text, format_details(&gameplay));
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

fn format_header(gameplay: &GameplayResource) -> String {
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
            match (&run.phase, &run.battle) {
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

fn format_score_line(gameplay: &GameplayResource) -> String {
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
            match &run.battle {
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

fn format_chimeras(gameplay: &GameplayResource) -> String {
    match gameplay.mode {
        AppMode::WorkAssignment => format_work_chimeras(&gameplay.work.assignment),
        AppMode::ChimeraBattle => format_run_chimeras(&gameplay.battle),
    }
}

fn format_details(gameplay: &GameplayResource) -> String {
    match gameplay.mode {
        AppMode::WorkAssignment => format_work_run_details(&gameplay.work),
        AppMode::ChimeraBattle => format_run_details(&gameplay.battle),
    }
}

fn format_work_chimeras(state: &CombatState) -> String {
    let mut chimeras = state.chimeras.iter().collect::<Vec<_>>();
    chimeras.sort_by_key(|chimera| chimera.slot);

    chimeras
        .into_iter()
        .map(|chimera| {
            let status = if chimera.is_active { "active" } else { "out" };
            format!(
                "{}  slot {} ({status})\n  stamina {}/{}  efficiency {}\n",
                chimera.name,
                chimera.slot,
                chimera.stats.stamina,
                chimera.stats.max_stamina,
                chimera.stats.efficiency
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_run_chimeras(run: &BattleRunState) -> String {
    match &run.battle {
        Some(state) => format_battle_chimeras(state),
        None => format_draft_chimeras(run),
    }
}

fn format_draft_chimeras(run: &BattleRunState) -> String {
    let team = &run.draft.team;
    let mut chimeras = team.chimeras.iter().collect::<Vec<_>>();
    chimeras.sort_by_key(|chimera| chimera.slot);
    let active_entries = chimeras
        .into_iter()
        .map(|chimera| {
            let equipment = if chimera.equipment.is_empty() {
                "none".to_string()
            } else {
                chimera
                    .equipment
                    .iter()
                    .map(|equipment| equipment.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            format!(
                "  {}  slot {}\n    HP {}/{}  ATK {}  Lv{}  XP {}  Eq: {}",
                chimera.name,
                chimera.slot,
                chimera.stats.hp,
                chimera.stats.max_hp,
                chimera.stats.attack,
                chimera.level,
                chimera.experience,
                equipment
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let bench_entries = if run.draft.bench.is_empty() {
        "  empty".to_string()
    } else {
        run.draft
            .bench
            .iter()
            .enumerate()
            .map(|(index, chimera)| {
                format!(
                    "  {}. {}\n    HP {}/{}  ATK {}  Lv{}  XP {}",
                    index + 1,
                    chimera.name,
                    chimera.stats.hp,
                    chimera.stats.max_hp,
                    chimera.stats.attack,
                    chimera.level,
                    chimera.experience
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let equipment_entries = if run.draft.equipment_inventory.is_empty() {
        "  empty".to_string()
    } else {
        run.draft
            .equipment_inventory
            .iter()
            .enumerate()
            .map(|(index, equipment)| {
                format!(
                    "  {}. {}  ATK +{}  HP +{}  {:?}",
                    index + 1,
                    equipment.name,
                    equipment.attack,
                    equipment.hp,
                    equipment.rarity
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "Draft Team - {} ({}/{})\n{active_entries}\n\nBench\n{bench_entries}\n\nEquipment Inventory\n{equipment_entries}",
        team.name,
        team.chimeras.len(),
        run.draft.active_team_limit
    )
}

fn format_battle_chimeras(state: &BattleState) -> String {
    [TeamSide::Challenger, TeamSide::Defender]
        .into_iter()
        .map(|side| {
            let team = state.team(side);
            let mut chimeras = team.chimeras.iter().collect::<Vec<_>>();
            chimeras.sort_by_key(|chimera| chimera.slot);
            let entries = chimeras
                .into_iter()
                .map(|chimera| {
                    let status = if chimera.is_alive() { "alive" } else { "down" };
                    format!(
                        "  {}  slot {} ({status})\n    HP {}/{}  ATK {}  Lv{}",
                        chimera.name,
                        chimera.slot,
                        chimera.stats.hp,
                        chimera.stats.max_hp,
                        chimera.stats.attack,
                        chimera.level
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("{} - {}\n{entries}", side_label(side), team.name)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn format_work_run_details(run: &WorkRunState) -> String {
    let assignment = &run.assignment;
    let period = run
        .current_review_period()
        .map(|period| {
            format!(
                "{} -> rank {} at {} cookies",
                period.name, period.target_rank, period.required_cookie_score
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let mode_detail = match run.phase {
        WorkRunPhase::Review => format!(
            "Review\n  Current target: {period}\n  Weeks elapsed: {}\n  Total cookies: {}",
            run.weeks_elapsed, run.total_cookies
        ),
        WorkRunPhase::Overtime => format!(
            "Overtime\n  Cycle: {}\n  Overtime cookies: {}\n  Task growth is active after every clear.",
            run.overtime_cycle, run.overtime_cookies
        ),
        WorkRunPhase::OvertimePrep => format!(
            "Overtime Prep\n  Completed cycle: {}\n  Overtime cookies: {}\n  Q/W/E: swap adjacent chimeras\n  1-5: toggle active workers\n  Space: start next cycle",
            run.overtime_cycle, run.overtime_cookies
        ),
        WorkRunPhase::Complete => format!(
            "Complete\n  Final rank: {}\n  Total cookies: {}\n  Overtime cookies: {}",
            run.current_rank, run.total_cookies, run.overtime_cookies
        ),
    };

    format!("{mode_detail}\n\nTasks\n{}", format_tasks(assignment))
}

fn format_tasks(state: &CombatState) -> String {
    let mut tasks = state.tasks.iter().collect::<Vec<_>>();
    tasks.sort_by_key(|task| task.order);

    tasks
        .into_iter()
        .map(|task| {
            let status = if task.progress.completed {
                "done"
            } else {
                "open"
            };
            format!(
                "{}  ({status})\n  progress {}/{}  cost {}  reward {}\n",
                task.name,
                task.progress.current,
                task.progress.required,
                task.progress.stamina_cost,
                task.progress.cookie_reward
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
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
        return String::new();
    }

    let max_offset = logs.len().saturating_sub(LOG_WINDOW_LINES);
    let offset = offset_from_latest.min(max_offset);
    let end = logs.len() - offset;
    let start = end.saturating_sub(LOG_WINDOW_LINES);
    let mut lines = logs[start..end].to_vec();

    if offset > 0 {
        lines.push(format!(
            "[{} newer log line(s) below - End jumps to latest]",
            offset
        ));
    }

    lines.join("\n")
}
