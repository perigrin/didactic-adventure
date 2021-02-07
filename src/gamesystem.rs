use super::{Attribute, Skill, Skills};
use itertools::Itertools;
use rltk::prelude::DiceType;

pub enum Success {
    Critical,
    Full,
    Partial,
    Miss,
}

pub fn roll_success(dice: DiceType) -> Success {
    let roll = crate::rng::roll(dice);
    match roll {
        12 => Success::Critical,
        10 | 11 => Success::Full,
        7..=9 => Success::Partial,
        _ => Success::Miss,
    }
}

pub fn roll_plus_stat(stat: Attribute) -> Success {
    roll_success(DiceType {
        n_dice: 2,
        die_type: 6,
        bonus: stat.base,
    })
}

pub fn roll_stat() -> i32 {
    let roll = roll_success(DiceType {
        n_dice: 1,
        die_type: 6,
        bonus: 0,
    });
    match roll {
        Success::Critical => 3,
        Success::Full => 2,
        Success::Partial => 1,
        Success::Miss => 0,
    }
}

pub fn player_hp_at_level(con: i32, level: i32) -> i32 {
    let n: i32 = 1 + con + (level / 2);
    let hp = (0..n)
        .map(|_| crate::rng::roll_dice(1, 6))
        .sorted()
        .rev()
        .take(level as usize)
        .sum();
    hp
}

pub fn npc_hp(con: i32, level: i32) -> i32 {
    let n: i32 = 1 + con + (level / 2);
    let hp = (0..n)
        .map(|_| crate::rng::roll_dice(1, 6))
        .sorted()
        .take(level as usize)
        .sum();
    hp
}

pub fn mana_at_level(intelligence: i32, level: i32) -> i32 {
    4 * level
}

pub fn skill_bonus(skill: Skill, skills: &Skills) -> i32 {
    if skills.skills.contains_key(&skill) {
        skills.skills[&skill]
    } else {
        -4
    }
}
