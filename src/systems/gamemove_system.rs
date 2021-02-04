use crate::{
    effects::*, Attributes, Equipped, GameMove, HungerClock, Name, NaturalAttackDefense, Pools,
    Skills, WantsToGameMove, Weapon, Wearable,
};
use specs::prelude::*;

pub struct GameMoveSystem {}

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

        for (entity, gamemove, pc_name, pc_attributes, pc_skills, pc_pools) in
            (&entities, &wants_move, &names, &attributes, &skills, &pools).join()
        {
            let npc_name = names.get(gamemove.npc).unwrap();
            let npc_attributes = attributes.get(gamemove.npc).unwrap();

            match gamemove.game_move {
                GameMove::Defend => {
                    let success = crate::roll_plus_stat(pc_attributes.CON); // roll+CON
                    let damage = 0;
                    match success {
                        12 => {
                            add_effect(
                                Some(entity),
                                EffectType::Damage { amount: damage },
                                Targets::Single {
                                    target: gamemove.npc,
                                },
                            );
                            crate::gamelog::Logger::new()
                                .npc_name(&pc_name.name)
                                .append("dodges")
                                .npc_name(&npc_name.name)
                                .append("and hits")
                                .npc_name(&npc_name.name)
                                .append("for")
                                .damage(damage)
                                .append("hp.")
                                .log();
                        }
                        10 | 11 => {
                            // PC avoids NPC
                            crate::gamelog::Logger::new()
                                .npc_name(&pc_name.name)
                                .append("dodges")
                                .npc_name(&npc_name.name)
                                .log();
                        }
                        7..=9 => {
                            // PC "mostly" avoids NPC, takes half damage
                            add_effect(
                                Some(gamemove.npc),
                                EffectType::Damage { amount: damage },
                                Targets::Single { target: entity },
                            );
                            crate::gamelog::Logger::new()
                                .npc_name(&pc_name.name)
                                .append("almost dodges")
                                .npc_name(&npc_name.name)
                                .append("and is hit for only")
                                .damage(damage)
                                .append("hp.")
                                .log();
                        }
                        _ => {
                            // PC is hit
                            add_effect(
                                Some(gamemove.npc),
                                EffectType::Damage { amount: damage },
                                Targets::Single { target: entity },
                            );
                            crate::gamelog::Logger::new()
                                .npc_name(&npc_name.name)
                                .append("hits")
                                .npc_name(&pc_name.name)
                                .append("for")
                                .damage(damage)
                                .append("hp.")
                                .log();
                        }
                    }
                }
                GameMove::DefyDanger => {}
                GameMove::DiscernReality => {}
                GameMove::HackAndSlash => {}
                GameMove::LastBreath => {}
                GameMove::MakeCamp => {}
                GameMove::Parley => {}
                GameMove::SpoutLore => {}
                GameMove::Supply => {}
                GameMove::UndertakePerilousJourney => {}
                GameMove::Volly => {}
            }
        }
    }
}
