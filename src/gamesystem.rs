use super::{Skill, Skills};

pub fn roll_stat() -> i32 {
    let roll = crate::rng::roll_dice(1, 6);
    let val = match roll {
        12 => 3,
        10 | 11 => 2,
        7..=9 => 1,
        _ => 0,
    };

    val
}

pub fn attr_bonus(value: i32) -> i32 {
    0
}

pub fn player_hp_per_level(fitness: i32) -> i32 {
    15 + attr_bonus(fitness)
}

pub fn player_hp_at_level(fitness: i32, level: i32) -> i32 {
    15 + (player_hp_per_level(fitness) * level)
}

pub fn npc_hp(fitness: i32, level: i32) -> i32 {
    let mut total = 1;
    for _i in 0..level {
        total += i32::max(1, 8 + attr_bonus(fitness));
    }
    total
}

pub fn mana_per_level(intelligence: i32) -> i32 {
    i32::max(1, 4 + attr_bonus(intelligence))
}

pub fn mana_at_level(intelligence: i32, level: i32) -> i32 {
    mana_per_level(intelligence) * level
}

pub fn skill_bonus(skill: Skill, skills: &Skills) -> i32 {
    if skills.skills.contains_key(&skill) {
        skills.skills[&skill]
    } else {
        -4
    }
}
