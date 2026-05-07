//! Bevy UI adapter for the pure gameplay cores.

use bevy::{app::AppExit, prelude::*};

use crate::{
    app_state::AppMode,
    core::{
        battle::{
            BattleDefinition, BattleLeaderEffect, BattleRunPhase, BattleRunState, BattleRunStep,
            BattleState, PurchaseOutcome, TeamSide, resolver::front_chimera_id,
        },
        work::{CombatState, StageDefinition},
    },
};

#[derive(Resource)]
struct GameplayResource {
    mode: AppMode,
    work_definition: StageDefinition,
    battle_definition: BattleDefinition,
    work: CombatState,
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
        work: CombatState::from_stage(stage.clone()),
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
                "Tab: switch mode    Space: advance/start battle    1-3: buy    R: refresh shop    Q/W/E: swap lineup    B/V: bench/deploy    N: reset    Up/Down/Page: scroll log    End: latest    Esc: quit",
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
    }

    if keys.just_pressed(KeyCode::Space) {
        match gameplay.mode {
            AppMode::WorkAssignment => {
                let outcome = gameplay.work.step_round();
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
}

fn reset_active_mode(gameplay: &mut GameplayResource, logs: &mut UiLogs) {
    match gameplay.mode {
        AppMode::WorkAssignment => {
            gameplay.work = CombatState::from_stage(gameplay.work_definition.clone());
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
            let state = &gameplay.work;
            format!(
                "Work Assignment - Round {}/{}{}",
                state.round,
                state.max_round,
                if state.is_finished { " - Finished" } else { "" }
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
            let state = &gameplay.work;
            format!(
                "Awoo Cookies: {} / {}    Completed Tasks: {} / {}",
                state.cookie_score,
                state.target_cookie_score,
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
                    run.defenders.len()
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
        AppMode::WorkAssignment => format_work_chimeras(&gameplay.work),
        AppMode::ChimeraBattle => format_run_chimeras(&gameplay.battle),
    }
}

fn format_details(gameplay: &GameplayResource) -> String {
    match gameplay.mode {
        AppMode::WorkAssignment => format_tasks(&gameplay.work),
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
            format!(
                "  {}  slot {}\n    HP {}/{}  ATK {}  Lv{}  XP {}",
                chimera.name,
                chimera.slot,
                chimera.stats.hp,
                chimera.stats.max_hp,
                chimera.stats.attack,
                chimera.level,
                chimera.experience
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

    format!(
        "Draft Team - {} ({}/{})\n{active_entries}\n\nBench\n{bench_entries}",
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
                    .map(|(index, offer)| {
                        format!(
                            "{}. {}  ATK {}  HP {}  {:?}",
                            index + 1,
                            offer.name,
                            offer.attack,
                            offer.hp,
                            offer.rarity
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let opponent = run
                .defenders
                .get(run.battle_index)
                .map(|team| team.name.as_str())
                .unwrap_or("None");
            format!(
                "Draft\n  Leader: {}\n  Health: {}/{}\n  Gold: {}\n  Active lineup: {}/{}\n  Bench: {}\n  Next opponent: {opponent}\n  Press Space to start battle.\n\nLeader Effects\n{}\n\nShop\n{shop}",
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
            "Run complete.\nWins: {}\nLosses: {}\nHealth: {}/{}\nGold: {}",
            run.wins, run.losses, run.health, run.max_health, run.draft.gold
        ),
        (BattleRunPhase::Battle, None) => "Battle is preparing.".to_string(),
    }
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
        return "none".to_string();
    }

    leader
        .effects
        .iter()
        .map(|effect| match effect {
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
        })
        .collect::<Vec<_>>()
        .join("\n")
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
