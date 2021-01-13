use super::{
    gamelog::GameLog, CombatStats, DefenseBonus, Equipped, MeleePowerBonus, Name, SufferDamage,
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
        ReadStorage<'a, DefenseBonus>,
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
            _melee_power_bonuses,
            _defense_bonuses,
            _equipped,
            mut rng,
        ) = data;

        for (entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            let target_stats = combat_stats.get(wants_melee.target).unwrap();
            if stats.hp > 0 && target_stats.hp > 0 {
                let target_name = names.get(wants_melee.target).unwrap();
                let dice_roll = rng.roll_dice(2, 6) + stats.str;
                match dice_roll {
                    12 => {
                        // critical success
                        let damage = rng.roll_dice(1, 4); // TODO look at equipment
                        let damage = damage * 2; // TODO figure out a better boon
                        log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                    10 | 11 => {
                        // normal success
                        let damage = rng.roll_dice(1, 6); // TODO look at equipment
                        log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                    7..=9 => {
                        // partial failure ... enemy gets a hit too
                        let damage = rng.roll_dice(1, 6); // TODO look at equipment
                        log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);

                        // enemy's hit
                        let damage = rng.roll_dice(1, 6); // TODO look at equipment
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
                        let damage = rng.roll_dice(1, 6); // TODO look at equipment
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
