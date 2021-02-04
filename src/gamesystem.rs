use super::{Attribute, Skill, Skills};
use itertools::Itertools;
use rltk::prelude::DiceType;

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

pub fn roll_plus_stat(stat: Attribute) -> i32 {
    let roll = crate::rng::roll(DiceType {
        n_dice: 2,
        die_type: 6,
        bonus: stat.base,
    });

    roll
}
