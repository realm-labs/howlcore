//! Data definitions for battle abilities, selectors, and effects.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BattleAbilityId(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleTrigger {
    BattleStart,
    TurnStart,
    BeforeDamageTaken,
    AfterDamageTaken,
    OnAllyAttack,
    AfterAttack,
    OnAllyAheadDamaged,
    OnSummon,
    OnKnockdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleTargetSelector {
    SelfChimera,
    AttackTarget,
    DamageTarget,
    SummonedChimera,
    KnockedDownChimera,
    FrontEnemy,
    FirstLivingEnemy,
    AllyAhead,
    AllyBehind,
    HighestHpEnemy,
    HighestAttackEnemy,
    AllEnemies,
    AllAllies,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleEffect {
    Chance {
        percent: u32,
        effects: Vec<BattleEffect>,
    },
    DealDamage {
        amount: i32,
    },
    DealAttackDamagePercent {
        percent: u32,
        minimum: i32,
    },
    Heal {
        amount: i32,
    },
    AddAttack {
        amount: i32,
    },
    ReduceIncomingDamage {
        amount: i32,
        minimum: i32,
    },
    SwapWithTarget,
    QueueSummon {
        name: &'static str,
        attack: i32,
        hp: i32,
        abilities: Vec<BattleAbilityId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleAbilityDef {
    pub id: BattleAbilityId,
    pub name: &'static str,
    pub trigger: BattleTrigger,
    pub selector: BattleTargetSelector,
    pub effects: Vec<BattleEffect>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BattleAbilityDatabase {
    pub abilities: HashMap<BattleAbilityId, BattleAbilityDef>,
}
