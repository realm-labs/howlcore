//! Data definitions for triggers, selectors, effects, and trait records.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraitId(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    WorkStart,
    RoundStart,
    OnWork,
    AfterWork,
    RoundEnd,
    TaskCompleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetSelector {
    SelfChimera,
    FrontTask,
    AllTasks,
    PreviousAlly,
    NextAlly,
    AdjacentAllies,
    AllAllies,
    LowestStaminaAlly,
    HighestEfficiencyAlly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    AdvanceTask { amount: i32 },
    AdvanceTaskByEfficiency { bonus: i32 },
    GainCookie { amount: i32 },
    AddEfficiency { amount: i32, duration: u32 },
    AddStamina { amount: i32 },
    ConsumeStamina { amount: i32 },
    RestoreStamina { amount: i32 },
    AddTemporaryTrait { trait_id: TraitId, duration: u32 },
}

#[derive(Debug, Clone)]
pub struct TraitDef {
    pub id: TraitId,
    pub name: &'static str,
    pub trigger: Trigger,
    pub selector: TargetSelector,
    pub effects: Vec<Effect>,
}
