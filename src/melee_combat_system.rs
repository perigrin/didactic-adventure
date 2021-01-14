use super::{
    gamelog::GameLog, ArmorBonus, CombatStats, Equipped, MeleePowerBonus, Name, SufferDamage,
    WantsToMelee,
};
use specs::prelude::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, ArmorBonus>,
        ReadStorage<'a, Equipped>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            mut wants_melee,
            names,
            combat_stats,
            mut inflict_damage,
            melee_power_bonuses,
            defense_bonuses,
            equipped,
            mut rng,
        ) = data;

        for (entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            let target_stats = combat_stats.get(wants_melee.target).unwrap();
            let mut offensive_bonus = 0;
            for (_item_entity, melee_bonus, equipped_by) in
                (&entities, &melee_power_bonuses, &equipped).join()
            {
                if equipped_by.owner == entity {
                    offensive_bonus += melee_bonus.bonus;
                }
            }
            if stats.hp > 0 && target_stats.hp > 0 {
                let target_name = names.get(wants_melee.target).unwrap();
                let dice_roll = rng.roll_dice(2, 6) + stats.str;
                let mut defensive_bonus = 0;
                for (_item_entity, defense_bonus, equipped_by) in
                    (&entities, &defense_bonuses, &equipped).join()
                {
                    if equipped_by.owner == wants_melee.target {
                        defensive_bonus += defense_bonus.bonus;
                    }
                }

                match dice_roll {
                    12 => {
                        // critical success
                        let damage = rng.roll_dice(1, 6) + offensive_bonus - defensive_bonus;
                        let damage = damage + 2; // TODO figure out a better boon
                        log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                    10 | 11 => {
                        // normal success
                        let damage = rng.roll_dice(1, 6) + offensive_bonus - defensive_bonus;
                        log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                    7..=9 => {
                        // partial failure ... enemy gets a hit too
                        let damage = rng.roll_dice(1, 6) + offensive_bonus - defensive_bonus;
                        log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);

                        // enemy's hit - reverse the calculations
                        let mut offensive_bonus = 0;
                        for (_item_entity, melee_bonus, equipped_by) in
                            (&entities, &melee_power_bonuses, &equipped).join()
                        {
                            if equipped_by.owner == wants_melee.target {
                                offensive_bonus += melee_bonus.bonus;
                            }
                        }

                        let mut defensive_bonus = 0;
                        for (_item_entity, defense_bonus, equipped_by) in
                            (&entities, &defense_bonuses, &equipped).join()
                        {
                            if equipped_by.owner == entity {
                                defensive_bonus += defense_bonus.bonus;
                            }
                        }

                        let damage = rng.roll_dice(1, 6) + offensive_bonus - defensive_bonus;
                        log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &target_name.name, &name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, entity, damage);
                    }
                    _ => {
                        // botch, enemy get a free hit
                        log.entries.push(format!(
                            "{} is unable to hurt {}",
                            &name.name, &target_name.name
                        ));

                        // enemy's hit - reverse the calculations
                        let mut offensive_bonus = 0;
                        for (_item_entity, melee_bonus, equipped_by) in
                            (&entities, &melee_power_bonuses, &equipped).join()
                        {
                            if equipped_by.owner == wants_melee.target {
                                offensive_bonus += melee_bonus.bonus;
                            }
                        }

                        let mut defensive_bonus = 0;
                        for (_item_entity, defense_bonus, equipped_by) in
                            (&entities, &defense_bonuses, &equipped).join()
                        {
                            if equipped_by.owner == entity {
                                defensive_bonus += defense_bonus.bonus;
                            }
                        }

                        let damage = rng.roll_dice(1, 6) + offensive_bonus - defensive_bonus;
                        log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &target_name.name, &name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, entity, damage);
                    }
                }
            }
        }

        wants_melee.clear();
    }
}
