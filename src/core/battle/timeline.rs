//! UI-independent battle playback timeline.

use crate::core::battle::{
    BattleAbilityDatabase, BattleChimeraId, BattleEvent, BattleRunEvent, BattleRunStep,
    BattleState, TeamSide,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleViewSnapshot {
    pub state: BattleState,
}

impl BattleViewSnapshot {
    pub fn from_state(state: &BattleState) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleFrameKind {
    Run,
    Turn,
    Attack,
    Ability,
    Chance,
    Damage,
    Heal,
    Buff,
    Position,
    Summon,
    Knockdown,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleTimelineFrame {
    pub line: String,
    pub snapshot: Option<BattleViewSnapshot>,
    pub kind: BattleFrameKind,
    pub focus: Vec<BattleChimeraId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BattleTimeline {
    pub frames: Vec<BattleTimelineFrame>,
}

impl BattleTimeline {
    pub fn from_step(
        step: BattleRunStep,
        battle_before: Option<&BattleState>,
        battle_after: Option<&BattleState>,
        battle_events: &[BattleEvent],
        run_events: &[BattleRunEvent],
        abilities: &BattleAbilityDatabase,
    ) -> Self {
        let mut frames = Vec::new();
        let mut visual = battle_before.cloned();

        match step {
            BattleRunStep::StartedBattle => {
                visual = battle_after.cloned();
                frames.push(frame(
                    "Battle run: started battle.",
                    visual.as_ref(),
                    BattleFrameKind::Run,
                    Vec::new(),
                ));
            }
            BattleRunStep::AdvancedBattle => {}
            BattleRunStep::BattleResolved { .. } => {}
        }

        for event in battle_events {
            apply_event(&mut visual, event);
            frames.push(frame(
                format_battle_event(event, visual.as_ref(), abilities),
                visual.as_ref(),
                frame_kind(event),
                event_focus(event),
            ));
        }

        if let BattleRunStep::BattleResolved { winner } = step {
            let result = winner
                .map(|side| format!("{} wins", side_label(side)))
                .unwrap_or_else(|| "draw".to_string());
            frames.push(frame(
                format!("Battle run: resolved battle, {result}."),
                visual.as_ref().or(battle_after),
                BattleFrameKind::Run,
                Vec::new(),
            ));
        }

        for event in run_events {
            frames.push(frame(
                format_run_event(event),
                visual.as_ref(),
                BattleFrameKind::Run,
                Vec::new(),
            ));
        }

        Self { frames }
    }

    pub fn lines(&self) -> Vec<String> {
        self.frames.iter().map(|frame| frame.line.clone()).collect()
    }
}

fn frame(
    line: impl Into<String>,
    state: Option<&BattleState>,
    kind: BattleFrameKind,
    focus: Vec<BattleChimeraId>,
) -> BattleTimelineFrame {
    BattleTimelineFrame {
        line: line.into(),
        snapshot: state.map(BattleViewSnapshot::from_state),
        kind,
        focus,
    }
}

fn apply_event(visual: &mut Option<BattleState>, event: &BattleEvent) {
    let Some(state) = visual else {
        return;
    };

    match event {
        BattleEvent::BattleStarted => state.has_started = true,
        BattleEvent::TurnStarted { turn } => state.turn = *turn,
        BattleEvent::DamageApplied {
            target, hp_after, ..
        } => {
            if let Some(chimera) = state.chimera_mut(*target) {
                chimera.stats.hp = *hp_after;
            }
        }
        BattleEvent::HpRestored {
            target, hp_after, ..
        } => {
            if let Some(chimera) = state.chimera_mut(*target) {
                chimera.stats.hp = *hp_after;
            }
        }
        BattleEvent::AttackChanged {
            target,
            attack_after,
            ..
        } => {
            if let Some(chimera) = state.chimera_mut(*target) {
                chimera.stats.attack = *attack_after;
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
        BattleEvent::ChimeraQueued { side, chimera } => {
            state.team_mut(*side).summon_queue.push(chimera.clone());
        }
        BattleEvent::ChimeraSummoned {
            chimera,
            state: summoned,
        } => {
            let team = state.team_mut(chimera.side);
            if chimera.index >= team.chimeras.len() {
                team.chimeras.push(summoned.clone());
            }
            if !team.summon_queue.is_empty() {
                team.summon_queue.remove(0);
            }
        }
        BattleEvent::BattleEnded { winner } => {
            state.is_finished = true;
            state.winner = *winner;
        }
        BattleEvent::BasicAttack { .. }
        | BattleEvent::DamageReduced { .. }
        | BattleEvent::AbilityTriggered { .. }
        | BattleEvent::ChanceRolled { .. }
        | BattleEvent::ChimeraKnockedDown { .. } => {}
    }
}

fn format_battle_event(
    event: &BattleEvent,
    state: Option<&BattleState>,
    abilities: &BattleAbilityDatabase,
) -> String {
    match event {
        BattleEvent::BattleStarted => "Battle started.".to_string(),
        BattleEvent::TurnStarted { turn } => turn_line(*turn, "Turn started."),
        BattleEvent::BasicAttack {
            attacker,
            target,
            damage,
        } => turn_line(
            state.map(|state| state.turn).unwrap_or_default(),
            format!(
                "{} attacks {} for {damage}.",
                chimera_name(state, *attacker),
                chimera_name(state, *target)
            ),
        ),
        BattleEvent::DamageApplied {
            target,
            amount,
            hp_before,
            hp_after,
        } => turn_line(
            state.map(|state| state.turn).unwrap_or_default(),
            format!(
                "{} took {amount} damage. HP: {hp_before} -> {hp_after}.",
                chimera_name(state, *target)
            ),
        ),
        BattleEvent::DamageReduced {
            target,
            amount,
            damage_before,
            damage_after,
        } => turn_line(
            state.map(|state| state.turn).unwrap_or_default(),
            format!(
                "{} reduced incoming damage by {amount}. Damage: {damage_before} -> {damage_after}.",
                chimera_name(state, *target)
            ),
        ),
        BattleEvent::HpRestored {
            target,
            amount,
            hp_before,
            hp_after,
        } => turn_line(
            state.map(|state| state.turn).unwrap_or_default(),
            format!(
                "{} restored {amount} HP. HP: {hp_before} -> {hp_after}.",
                chimera_name(state, *target)
            ),
        ),
        BattleEvent::AttackChanged {
            target,
            amount,
            attack_before,
            attack_after,
        } => turn_line(
            state.map(|state| state.turn).unwrap_or_default(),
            format!(
                "{} attack changed by {amount}. ATK: {attack_before} -> {attack_after}.",
                chimera_name(state, *target)
            ),
        ),
        BattleEvent::PositionSwapped { first, second } => turn_line(
            state.map(|state| state.turn).unwrap_or_default(),
            format!(
                "{} swapped positions with {}.",
                chimera_name(state, *first),
                chimera_name(state, *second)
            ),
        ),
        BattleEvent::ChimeraQueued { side, chimera } => turn_line(
            state.map(|state| state.turn).unwrap_or_default(),
            format!(
                "{} was added to {}'s summon queue.",
                chimera.name,
                side_label(*side)
            ),
        ),
        BattleEvent::ChimeraSummoned {
            chimera,
            state: summoned,
        } => turn_line(
            state.map(|state| state.turn).unwrap_or_default(),
            format!(
                "{} joined {}'s lineup at slot {}.",
                summoned.name,
                side_label(chimera.side),
                summoned.slot
            ),
        ),
        BattleEvent::AbilityTriggered { source, ability } => {
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
            turn_line(
                state.map(|state| state.turn).unwrap_or_default(),
                format!(
                    "Trigger: {} {} -> {}",
                    side_label(source.side),
                    chimera_name(state, *source),
                    ability_text
                ),
            )
        }
        BattleEvent::ChanceRolled {
            percent,
            roll,
            success,
        } => turn_line(
            state.map(|state| state.turn).unwrap_or_default(),
            format!(
                "Trigger roll: {percent}% rolled {roll} => {}",
                if *success { "success" } else { "miss" }
            ),
        ),
        BattleEvent::ChimeraKnockedDown { chimera } => turn_line(
            state.map(|state| state.turn).unwrap_or_default(),
            format!("{} was knocked down.", chimera_name(state, *chimera)),
        ),
        BattleEvent::BattleEnded { winner } => match winner {
            Some(TeamSide::Challenger) => "Battle ended. Winner: Challenger.".to_string(),
            Some(TeamSide::Defender) => "Battle ended. Winner: Defender.".to_string(),
            None => "Battle ended. Result: Draw.".to_string(),
        },
    }
}

fn format_run_event(event: &BattleRunEvent) -> String {
    match event {
        BattleRunEvent::RunHealthLost {
            amount,
            health_after,
            max_health,
        } => {
            format!("Run damage: lost {amount} health. Health: {health_after}/{max_health}.")
        }
        BattleRunEvent::GoldRewarded { amount, gold_after } => {
            format!("Run reward: gained {amount} gold. Gold: {gold_after}.")
        }
        BattleRunEvent::RunHealed {
            amount,
            health_after,
            max_health,
        } => format!("Run reward: healed {amount}. Health: {health_after}/{max_health}."),
        BattleRunEvent::ShopItemRewarded { item_name } => {
            format!("Run reward: added {item_name} to the shop.")
        }
    }
}

fn frame_kind(event: &BattleEvent) -> BattleFrameKind {
    match event {
        BattleEvent::BattleStarted => BattleFrameKind::Run,
        BattleEvent::TurnStarted { .. } => BattleFrameKind::Turn,
        BattleEvent::BasicAttack { .. } => BattleFrameKind::Attack,
        BattleEvent::DamageApplied { .. } | BattleEvent::DamageReduced { .. } => {
            BattleFrameKind::Damage
        }
        BattleEvent::HpRestored { .. } => BattleFrameKind::Heal,
        BattleEvent::AttackChanged { .. } => BattleFrameKind::Buff,
        BattleEvent::PositionSwapped { .. } => BattleFrameKind::Position,
        BattleEvent::ChimeraQueued { .. } | BattleEvent::ChimeraSummoned { .. } => {
            BattleFrameKind::Summon
        }
        BattleEvent::AbilityTriggered { .. } => BattleFrameKind::Ability,
        BattleEvent::ChanceRolled { .. } => BattleFrameKind::Chance,
        BattleEvent::ChimeraKnockedDown { .. } => BattleFrameKind::Knockdown,
        BattleEvent::BattleEnded { .. } => BattleFrameKind::End,
    }
}

fn event_focus(event: &BattleEvent) -> Vec<BattleChimeraId> {
    match event {
        BattleEvent::BasicAttack {
            attacker, target, ..
        } => vec![*attacker, *target],
        BattleEvent::DamageApplied { target, .. }
        | BattleEvent::DamageReduced { target, .. }
        | BattleEvent::HpRestored { target, .. }
        | BattleEvent::AttackChanged { target, .. } => vec![*target],
        BattleEvent::PositionSwapped { first, second } => vec![*first, *second],
        BattleEvent::AbilityTriggered { source, .. }
        | BattleEvent::ChimeraSummoned {
            chimera: source, ..
        }
        | BattleEvent::ChimeraKnockedDown { chimera: source } => vec![*source],
        BattleEvent::BattleStarted
        | BattleEvent::TurnStarted { .. }
        | BattleEvent::ChimeraQueued { .. }
        | BattleEvent::ChanceRolled { .. }
        | BattleEvent::BattleEnded { .. } => Vec::new(),
    }
}

fn chimera_name(state: Option<&BattleState>, id: BattleChimeraId) -> String {
    state
        .and_then(|state| state.chimera(id))
        .map(|chimera| chimera.name.clone())
        .unwrap_or_else(|| format!("{:?} Chimera {}", id.side, id.index))
}

pub fn side_label(side: TeamSide) -> &'static str {
    match side {
        TeamSide::Challenger => "Challenger",
        TeamSide::Defender => "Defender",
    }
}

fn battle_trigger_label(trigger: crate::core::battle::BattleTrigger) -> &'static str {
    match trigger {
        crate::core::battle::BattleTrigger::BattleStart => "battle start",
        crate::core::battle::BattleTrigger::TurnStart => "turn start",
        crate::core::battle::BattleTrigger::BeforeDamageTaken => "before hit",
        crate::core::battle::BattleTrigger::AfterDamageTaken => "after hit",
        crate::core::battle::BattleTrigger::OnAllyAttack => "ally attack",
        crate::core::battle::BattleTrigger::AfterAttack => "after attack",
        crate::core::battle::BattleTrigger::OnAllyAheadDamaged => "ally ahead hit",
        crate::core::battle::BattleTrigger::OnSummon => "on summon",
        crate::core::battle::BattleTrigger::OnKnockdown => "knockdown",
    }
}

fn turn_line(turn: u32, message: impl AsRef<str>) -> String {
    format!("[Turn {turn}] {}", message.as_ref())
}
