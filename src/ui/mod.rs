//! Bevy UI adapter for the pure gameplay cores.

use bevy::{app::AppExit, prelude::*};

use crate::{
    app_state::AppMode,
    core::{
        battle::{BattleDefinition, BattleState, TeamSide, resolver::front_chimera_id},
        work::{CombatState, StageDefinition},
    },
};

#[derive(Resource)]
struct GameplayResource {
    mode: AppMode,
    work: CombatState,
    battle: BattleState,
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
        work: CombatState::from_stage(stage),
        battle: BattleState::from_definition(battle),
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
                "Tab: switch mode    Space: advance    Up/Down/Page: scroll log    End: latest    Esc: quit",
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
                let outcome = gameplay.battle.step_turn();
                let was_following = logs.battle_offset == 0;
                logs.battle.extend(outcome.logs);
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
            let state = &gameplay.battle;
            format!(
                "Chimera Battle - Turn {}/{}{}",
                state.turn,
                state.max_turn,
                if state.is_finished { " - Finished" } else { "" }
            )
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
            let state = &gameplay.battle;
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
                "{}: {}/{} alive    {}: {}/{} alive{}",
                state.challenger.name,
                challenger_alive,
                state.challenger.chimeras.len(),
                state.defender.name,
                defender_alive,
                state.defender.chimeras.len(),
                winner
            )
        }
    }
}

fn format_chimeras(gameplay: &GameplayResource) -> String {
    match gameplay.mode {
        AppMode::WorkAssignment => format_work_chimeras(&gameplay.work),
        AppMode::ChimeraBattle => format_battle_chimeras(&gameplay.battle),
    }
}

fn format_details(gameplay: &GameplayResource) -> String {
    match gameplay.mode {
        AppMode::WorkAssignment => format_tasks(&gameplay.work),
        AppMode::ChimeraBattle => format_battle_details(&gameplay.battle),
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
