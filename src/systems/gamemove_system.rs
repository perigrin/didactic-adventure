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
    name: Option<&'a Name>,
    attributes: Option<&'a Attributes>,
    skills: Option<&'a Skills>,
    pools: Option<&'a Pools>,
    weapon: Box<dyn AttackDice>,
}

impl<'a> Character<'a> {
    fn is_alive(&self) -> bool {
        if self.pools.unwrap().hit_points.current < 1 {
            return false; // c dead
        };
        true
    }
    fn deal_damage(&self, to: &Character, modifier: f32, logger: Logger) {
        let weapon = &self.weapon;
        let damage = weapon.damage();
        add_effect(
            Some(self.entity),
            EffectType::Damage {
                amount: ((damage as f32) * modifier) as i32,
            },
            Targets::Single { target: to.entity },
        );
        logger
            .npc_name(&self.name.unwrap().name)
            .append("hits")
            .npc_name(&to.name.unwrap().name)
            .append("for")
            .damage(damage)
            .append("hp.")
            .log();
    }
}

fn defend(gamemove: &WantsToGameMove, pc: Character, npc: Character) {
    if !pc.is_alive() || !npc.is_alive() {
        return; // can't defend if you're dead
    };

    let success = crate::roll_plus_stat(pc.attributes.unwrap().CON); // roll+CON
    match success {
        Success::Critical => {
            let log = Logger::new()
                .npc_name(&pc.name.unwrap().name)
                .append("dodges")
                .npc_name(&npc.name.unwrap().name)
                .append("and");

            pc.deal_damage(&npc, 1.0, log);
        }
        Success::Full => {
            // PC avoids NPC
            Logger::new()
                .npc_name(&pc.name.unwrap().name)
                .append("dodges")
                .npc_name(&npc.name.unwrap().name)
                .log();
        }
        Success::Partial => {
            // PC "mostly" avoids NPC, takes half damage
            let log = Logger::new()
                .npc_name(&pc.name.unwrap().name)
                .append("almost dodges")
                .npc_name(&npc.name.unwrap().name)
                .append("and");

            pc.deal_damage(&npc, 0.5, log)
        }
        Success::Miss => npc.deal_damage(&pc, 1.0, Logger::new()), // PC is hit
    }
}

fn hack_and_slash(gamemove: &WantsToGameMove, pc: Character, npc: Character) {
    if !pc.is_alive() || !npc.is_alive() {
        return; // can't hack and slash if you're dead
    }

    let success = crate::roll_plus_stat(pc.attributes.unwrap().STR); // roll+STR
    match success {
        Success::Critical => pc.deal_damage(&npc, 2.0, Logger::new()), // PC hits NPC for 2x
        Success::Full => pc.deal_damage(&npc, 1.0, Logger::new()), // PC hits NPC for regular damage

        Success::Partial => {
            // PC hits NPC for damage but gets hit in return
            pc.deal_damage(&npc, 1.0, Logger::new());

            let log = Logger::new()
                .append("But")
                .npc_name(&npc.name.unwrap().name)
                .append("manages a return strike; ");
            npc.deal_damage(&pc, 1.0, log);
        }
        Success::Miss => {
            // PC misses entirely and NPC gets a hit
            let log = Logger::new()
                .npc_name(&pc.name.unwrap().name)
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
            skills,
            hunger_clock,
            pools,
            equipped_items,
            weapon,
            wearables,
            natural,
        ) = data;

        let default_weapon = |character: &Entity| -> Box<dyn AttackDice> {
            Box::new(Weapon {
                range: None,
                attribute: WeaponAttribute::Might,
                hit_bonus: 0,
                damage_n_dice: 1,
                damage_die_type: 4,
                damage_bonus: 0,
                proc_chance: None,
                proc_target: None,
            })
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

        let find_combat_character = |entity: Entity, slot: EquipmentSlot| -> Character {
            Character {
                entity: entity,
                name: names.get(entity),
                attributes: attributes.get(entity),
                skills: skills.get(entity),
                pools: pools.get(entity),
                weapon: find_weapon(&entity, slot),
            }
        };

        for (entity, gamemove, pc_name, pc_attributes, pc_skills, pc_pools) in
            (&entities, &wants_move, &names, &attributes, &skills, &pools).join()
        {
            match gamemove.kind {
                GameMove::Defend => {
                    let pc = find_combat_character(entity, EquipmentSlot::Melee);
                    let npc = find_combat_character(gamemove.npc, EquipmentSlot::Melee);
                    defend(gamemove, pc, npc);
                }
                GameMove::DefyDanger => {}
                GameMove::DiscernReality => {}
                GameMove::HackAndSlash => {
                    let pc = find_combat_character(entity, EquipmentSlot::Melee);
                    let npc = find_combat_character(gamemove.npc, EquipmentSlot::Melee);
                    hack_and_slash(gamemove, pc, npc);
                }
                GameMove::LastBreath => {}
                GameMove::MakeCamp => {}
                GameMove::Parley => {}
                GameMove::SpoutLore => {}
                GameMove::Supply => {}
                GameMove::UndertakePerilousJourney => {}
                GameMove::Volly => {}
                _ => {}
            }
        }
    }
}
