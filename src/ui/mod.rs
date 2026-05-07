//! Bevy UI adapter for the pure work-battle core.

use bevy::{app::AppExit, prelude::*};

use crate::core::work::{CombatState, StageDefinition};

#[derive(Resource)]
struct CombatResource(CombatState);

#[derive(Resource, Default)]
struct UiLogs(Vec<String>);

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

pub fn build_app(stage: StageDefinition) -> App {
    let initial_logs = stage.initial_logs.clone();
    let combat = CombatState::from_stage(stage);
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Howlcore Combat Debugger".to_string(),
            resolution: (1180.0, 760.0).into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(CombatResource(combat))
    .insert_resource(UiLogs(initial_logs))
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
                spawn_panel(columns, "Tasks", 0.72, 0.9, 0.76, TaskText);
                spawn_panel(columns, "Log", 0.9, 0.82, 0.72, LogText);
            });
            root.spawn(TextBundle::from_section(
                "Space: next round    Esc: quit",
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
    mut combat: ResMut<CombatResource>,
    mut logs: ResMut<UiLogs>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }

    if keys.just_pressed(KeyCode::Space) {
        let outcome = combat.0.step_round();
        logs.0.extend(outcome.logs);
    }
}

fn update_round_text_system(
    combat: Res<CombatResource>,
    mut round_text: Query<&mut Text, With<RoundText>>,
) {
    let state = &combat.0;
    set_text(
        &mut round_text,
        format!(
            "Round {}/{}{}",
            state.round,
            state.max_round,
            if state.is_finished { " - Finished" } else { "" }
        ),
    );
}

fn update_score_text_system(
    combat: Res<CombatResource>,
    mut score_text: Query<&mut Text, With<ScoreText>>,
) {
    let state = &combat.0;
    set_text(
        &mut score_text,
        format!(
            "Awoo Cookies: {} / {}    Completed Tasks: {} / {}",
            state.cookie_score,
            state.target_cookie_score,
            state.completed_tasks,
            state.tasks.len()
        ),
    );
}

fn update_chimera_text_system(
    combat: Res<CombatResource>,
    mut chimera_text: Query<&mut Text, With<ChimeraText>>,
) {
    let state = &combat.0;
    set_text(&mut chimera_text, format_chimeras(state));
}

fn update_task_text_system(
    combat: Res<CombatResource>,
    mut task_text: Query<&mut Text, With<TaskText>>,
) {
    let state = &combat.0;
    set_text(&mut task_text, format_tasks(state));
}

fn update_log_text_system(logs: Res<UiLogs>, mut log_text: Query<&mut Text, With<LogText>>) {
    set_text(&mut log_text, format_logs(&logs.0));
}

fn set_text<T: Component>(query: &mut Query<&mut Text, With<T>>, value: String) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = value;
    }
}

fn format_chimeras(state: &CombatState) -> String {
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

fn format_logs(logs: &[String]) -> String {
    logs.iter()
        .rev()
        .take(18)
        .rev()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}
