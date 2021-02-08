use crate::{
    effects::*, gamelog::Logger, Attributes, EquipmentSlot, Equipped, GameMove, HungerClock, Name,
    NaturalAttack, NaturalAttackDefense, Pools, Skills, Success, WantsToGameMove, Weapon,
    WeaponAttribute, Wearable,
};
use rltk::DiceType;
use specs::prelude::*;

pub struct GameMoveSystem {}

pub trait AttackDice {
    fn attack_dice(&self) -> DiceType;

    fn damage(&self) -> i32 {
        crate::rng::roll(self.attack_dice())
    }
}

impl AttackDice for Weapon {
    fn attack_dice(&self) -> DiceType {
        DiceType {
            n_dice: self.damage_n_dice,
            die_type: self.damage_die_type,
            bonus: self.damage_bonus,
        }
    }
}

impl AttackDice for NaturalAttack {
    fn attack_dice(&self) -> DiceType {
        DiceType {
            n_dice: self.damage_n_dice,
            die_type: self.damage_die_type,
            bonus: self.damage_bonus,
        }
    }
}

struct Character<'a> {
    entity: Entity,
    name: String,
    attributes: Attributes,
    //skills: Option<&'a Skills>,
    pools: &'a Pools,
    weapon: Box<dyn AttackDice>,
}

impl<'a> Character<'a> {
    fn is_alive(&self) -> bool {
        if self.pools.hit_points.current < 1 {
            return false; // we dead jim
        };
        true
    }

    fn deal_damage(&self, to: &Character, modifier: f32, logger: Logger) {
        let weapon = &self.weapon;
        let damage = weapon.damage();

        rltk::console::log(format!(
            "So {}({:?}) is dealing damage {} to {}({:?})",
            self.name, self.entity, damage, to.name, to.entity
        ));

        add_effect(
            Some(self.entity),
            EffectType::Damage {
                amount: ((damage as f32) * modifier) as i32,
            },
            Targets::Single { target: to.entity },
        );

        logger
            .npc_name(&self.name)
            .append("hits")
            .npc_name(&to.name)
            .append("for")
            .damage(damage)
            .append("hp.")
            .log();
    }
}

fn defend(pc: Character, npc: Character) {
    rltk::console::log(format!(
        "{}({:?}) defends against {}({:?})",
        pc.name, pc.entity, npc.name, npc.entity
    ));

    if !pc.is_alive() || !npc.is_alive() {
        return; // can't defend if you're dead
    };

    // roll+CON
    match crate::roll_plus_stat(pc.attributes.con) {
        Success::Critical => {
            let log = Logger::new()
                .npc_name(&pc.name)
                .append("dodges")
                .npc_name(&npc.name)
                .append("and");

            pc.deal_damage(&npc, 1.0, log);
        }
        Success::Full => {
            // PC avoids NPC
            Logger::new()
                .npc_name(&pc.name)
                .append("dodges")
                .npc_name(&npc.name)
                .log();
        }
        Success::Partial => {
            // PC "mostly" avoids NPC, takes half damage
            let log = Logger::new()
                .npc_name(&pc.name)
                .append("almost dodges")
                .npc_name(&npc.name)
                .append("and");

            pc.deal_damage(&npc, 0.5, log)
        }
        Success::Miss => npc.deal_damage(&pc, 1.0, Logger::new()), // PC is hit
    }
}

fn hack_and_slash(pc: Character, npc: Character) {
    rltk::console::log(format!(
        "{}({:?}) hack_and_slash vs {}({:?})",
        pc.name, pc.entity, npc.name, npc.entity
    ));
    if !pc.is_alive() || !npc.is_alive() {
        return; // can't hack and slash if you're dead
    }

    let success = crate::roll_plus_stat(pc.attributes.str); // roll+STR
    match success {
        Success::Critical => pc.deal_damage(&npc, 2.0, Logger::new()), // PC hits NPC for 2x
        Success::Full => pc.deal_damage(&npc, 1.0, Logger::new()), // PC hits NPC for regular damage
        Success::Partial => {
            // PC hits NPC for damage but gets hit in return
            pc.deal_damage(&npc, 1.0, Logger::new());

            let log = Logger::new()
                .append("But")
                .npc_name(&npc.name)
                .append("manages a return strike; ");
            npc.deal_damage(&pc, 1.0, log);
        }
        Success::Miss => {
            // PC misses entirely and NPC gets a hit
            let log = Logger::new()
                .npc_name(&pc.name)
                .append("misses entirely and");

            npc.deal_damage(&pc, 1.0, log);
        }
    }
}

// TODO replace effects with non-melee effects
fn volly(pc: Character, npc: Character) {
    rltk::console::log(format!(
        "{}({:?}) volly vs {}({:?})",
        pc.name, pc.entity, npc.name, npc.entity
    ));
    if !pc.is_alive() || !npc.is_alive() {
        return; // can't hack and slash if you're dead
    }

    let success = crate::roll_plus_stat(pc.attributes.dex); // roll+DEX
    match success {
        Success::Critical => pc.deal_damage(&npc, 2.0, Logger::new()), // PC hits NPC for 2x
        Success::Full => pc.deal_damage(&npc, 1.0, Logger::new()), // PC hits NPC for regular damage
        Success::Partial => {
            // PC hits NPC for damage but gets hit in return
            pc.deal_damage(&npc, 1.0, Logger::new());

            let log = Logger::new()
                .append("But")
                .npc_name(&npc.name)
                .append("manages a return strike; ");
            npc.deal_damage(&pc, 1.0, log);
        }
        Success::Miss => {
            // PC misses entirely and NPC gets a hit
            let log = Logger::new()
                .npc_name(&pc.name)
                .append("misses entirely and");

            npc.deal_damage(&pc, 1.0, log);
        }
    }
}

impl<'a> System<'a> for GameMoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToGameMove>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
        ReadStorage<'a, HungerClock>,
        ReadStorage<'a, Pools>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, Weapon>,
        ReadStorage<'a, Wearable>,
        ReadStorage<'a, NaturalAttackDefense>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            wants_move,
            names,
            attributes,
            _skills,
            _hunger_clock,
            pools,
            equipped_items,
            weapon,
            _wearables,
            natural,
        ) = data;

        let default_weapon = |character: &Entity| -> Box<dyn AttackDice> {
            let mut weapon = Weapon {
                range: None,
                attribute: WeaponAttribute::Might,
                hit_bonus: 0,
                damage_n_dice: 1,
                damage_die_type: 4,
                damage_bonus: 0,
                proc_chance: None,
                proc_target: None,
            };

            if let Some(nat) = natural.get(*character) {
                if !nat.attacks.is_empty() {
                    let attack_index = if nat.attacks.len() == 1 {
                        0
                    } else {
                        crate::rng::roll_dice(1, nat.attacks.len() as i32) as usize - 1
                    };
                    weapon.hit_bonus = nat.attacks[attack_index].hit_bonus;
                    weapon.damage_n_dice = nat.attacks[attack_index].damage_n_dice;
                    weapon.damage_die_type = nat.attacks[attack_index].damage_die_type;
                    weapon.damage_bonus = nat.attacks[attack_index].damage_bonus;
                }
            }

            Box::new(weapon)
        };

        let find_weapon = |character: &Entity, slot: EquipmentSlot| -> Box<dyn AttackDice> {
            let mut found: Option<Weapon> = None;
            for (_e, wielded, melee) in (&entities, &equipped_items, &weapon).join() {
                if wielded.owner == *character && wielded.slot == slot {
                    found = Some(melee.clone());
                }
            }
            match found {
                None => default_weapon(character),
                Some(found) => Box::new(found),
            }
        };

        let new_character = |entity: Entity, slot: EquipmentSlot| -> Option<Character> {
            if let Some(pool) = pools.get(entity) {
                if pool.hit_points.current > 0 {
                    return Some(Character {
                        entity,
                        name: match names.get(entity) {
                            Some(name) => name.clone().name,
                            None => "Unknown".to_string(),
                        },
                        attributes: match attributes.get(entity) {
                            Some(attributes) => attributes.clone(),
                            None => panic!(format!("no attributes for {:?}", entity)),
                        },
                        //skills: skills.get(entity),
                        pools: pool,
                        weapon: find_weapon(&entity, slot),
                    });
                }
            }
            None
        };

        for (entity, gamemove) in (&entities, &wants_move).join() {
            match gamemove.kind {
                GameMove::Defend => {
                    if let Some(pc) = new_character(entity, EquipmentSlot::Melee) {
                        if let Some(npc) = new_character(gamemove.npc, EquipmentSlot::Melee) {
                            defend(pc, npc);
                        }
                    }
                }
                GameMove::DefyDanger => {}
                GameMove::DiscernReality => {}
                GameMove::HackAndSlash => {
                    if let Some(pc) = new_character(entity, EquipmentSlot::Melee) {
                        if let Some(npc) = new_character(gamemove.npc, EquipmentSlot::Melee) {
                            hack_and_slash(pc, npc);
                        }
                    }
                }
                GameMove::LastBreath => {}
                GameMove::MakeCamp => {}
                GameMove::Parley => {}
                GameMove::SpoutLore => {}
                GameMove::Supply => {}
                GameMove::UndertakePerilousJourney => {}
                GameMove::Volly => {
                    if let Some(pc) = new_character(entity, EquipmentSlot::Melee) {
                        if let Some(npc) = new_character(gamemove.npc, EquipmentSlot::Melee) {
                            volly(pc, npc);
                        }
                    }
                }
            }
        }
    }
}
