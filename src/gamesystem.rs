use super::{Attribute, Skill};
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

impl From<Attribute> for i32 {
    fn from(a: Attribute) -> i32 {
        a.base + a.modifiers + a.bonus
    }
}

pub fn roll_plus_stat(stat: i32) -> Success {
    roll_success(DiceType {
        n_dice: 2,
        die_type: 6,
        bonus: stat,
    })
}

pub fn roll_stat() -> i32 {
    let roll = roll_success(DiceType {
        n_dice: 2,
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

pub fn mana_at_level(_intelligence: i32, level: i32) -> i32 {
    4 * level
}

pub fn random_skill() -> Skill {
    let roll = crate::rng::roll_dice(1, 9);
    match roll {
        1 => Skill::Athletics,
        2 => Skill::Awareness,
        3 => Skill::Deception,
        4 => Skill::Decipher,
        5 => Skill::Heal,
        6 => Skill::Leadership,
        7 => Skill::Lore,
        8 => Skill::Stealth,
        9 => Skill::Survival,
        _ => random_skill(),
    }
}
