//! Test battle content loaded from RON configuration.

use crate::{
    content::battle_config::load_test_battle,
    core::battle::{BattleAbilityId, BattleDefinition},
};

pub const WORKAHOLIC: BattleAbilityId = BattleAbilityId("workaholic");
pub const TOUGH_COOKIE: BattleAbilityId = BattleAbilityId("tough_cookie");
pub const SOOTHING_CARE: BattleAbilityId = BattleAbilityId("soothing_care");
pub const ABSENTEE_FREAK: BattleAbilityId = BattleAbilityId("absentee_freak");
pub const RUTHLESS_DEMON: BattleAbilityId = BattleAbilityId("ruthless_demon");
pub const LITTLE_VILLAIN: BattleAbilityId = BattleAbilityId("little_villain");
pub const KIND_PRAISER: BattleAbilityId = BattleAbilityId("kind_praiser");
pub const SUMMON_TRAINER: BattleAbilityId = BattleAbilityId("summon_trainer");

pub fn test_battle() -> BattleDefinition {
    load_test_battle()
}
