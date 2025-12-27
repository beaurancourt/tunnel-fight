use crate::types::{Actor, AplEntry, Side, Zone};

#[derive(Debug, Clone)]
pub enum MoveAction {
    Move { direction: MoveDirection },
    None,
}

#[derive(Debug, Clone)]
pub enum AttackAction {
    Attack { target_id: usize },
    None,
}

#[derive(Debug, Clone)]
pub struct TurnActions {
    pub move_action: MoveAction,
    pub attack_action: AttackAction,
}

#[derive(Debug, Clone)]
pub enum MoveDirection {
    Toward(usize),
    ToZone(Zone),
    Forward,
    Backward,
}

pub struct AplContext<'a> {
    pub actor: &'a Actor,
    pub actors: &'a [Actor],
}

impl<'a> AplContext<'a> {
    pub fn enemies(&self) -> impl Iterator<Item = &Actor> {
        self.actors
            .iter()
            .filter(|a| a.side != self.actor.side && a.is_alive())
    }

    pub fn allies(&self) -> impl Iterator<Item = &Actor> {
        self.actors
            .iter()
            .filter(|a| a.side == self.actor.side && a.is_alive() && a.id != self.actor.id)
    }

    pub fn nearest_enemy(&self) -> Option<&Actor> {
        self.enemies()
            .min_by_key(|e| self.actor.zone.distance_to(&e.zone))
    }

    pub fn lowest_hp_enemy(&self) -> Option<&Actor> {
        self.enemies().min_by_key(|e| e.current_hp)
    }

    pub fn random_enemy(&self, rng: &mut impl rand::Rng) -> Option<&Actor> {
        let enemies: Vec<_> = self.enemies().collect();
        if enemies.is_empty() {
            None
        } else {
            Some(enemies[rng.gen_range(0..enemies.len())])
        }
    }

    pub fn enemies_in_range(&self) -> impl Iterator<Item = &Actor> {
        let actor_zone = self.actor.zone;
        let actor_range = self.actor.range;
        self.enemies()
            .filter(move |e| actor_zone.distance_to(&e.zone) <= actor_range.max_distance())
    }

    pub fn has_enemy_in_range(&self) -> bool {
        self.enemies_in_range().next().is_some()
    }
}

pub fn evaluate_condition(condition: &str, ctx: &AplContext) -> bool {
    let condition = condition.trim().to_lowercase();

    match condition.as_str() {
        "true" | "" => true,
        "false" => false,
        "enemy.in_range" | "enemy_in_range" => ctx.has_enemy_in_range(),
        "!enemy.in_range" | "!enemy_in_range" | "not enemy.in_range" => !ctx.has_enemy_in_range(),
        _ => {
            // Handle comparisons like target.health_percent < 20
            if condition.contains('<') {
                let parts: Vec<&str> = condition.split('<').collect();
                if parts.len() == 2 {
                    let lhs = parts[0].trim();
                    let rhs = parts[1].trim().parse::<f64>().unwrap_or(0.0);
                    if let Some(lhs_val) = evaluate_numeric(lhs, ctx) {
                        return lhs_val < rhs;
                    }
                }
            } else if condition.contains('>') {
                let parts: Vec<&str> = condition.split('>').collect();
                if parts.len() == 2 {
                    let lhs = parts[0].trim();
                    let rhs = parts[1].trim().parse::<f64>().unwrap_or(0.0);
                    if let Some(lhs_val) = evaluate_numeric(lhs, ctx) {
                        return lhs_val > rhs;
                    }
                }
            }
            true // Default to true for unknown conditions
        }
    }
}

fn evaluate_numeric(expr: &str, ctx: &AplContext) -> Option<f64> {
    match expr {
        "self.health_percent" | "self.hp_percent" => {
            Some(ctx.actor.current_hp as f64 / ctx.actor.max_hp as f64 * 100.0)
        }
        "self.hp" | "self.health" => Some(ctx.actor.current_hp as f64),
        "enemy.count" => Some(ctx.enemies().count() as f64),
        "ally.count" => Some(ctx.allies().count() as f64),
        _ => None,
    }
}

pub fn resolve_target(target_str: &str, ctx: &AplContext, rng: &mut impl rand::Rng) -> Option<usize> {
    let target_str = target_str.trim().to_lowercase();
    match target_str.as_str() {
        "nearest_enemy" | "nearest" => ctx.nearest_enemy().map(|a| a.id),
        "lowest_hp_enemy" | "lowest_hp" | "weakest" => ctx.lowest_hp_enemy().map(|a| a.id),
        "random_enemy" | "random" => ctx.random_enemy(rng).map(|a| a.id),
        _ => ctx.nearest_enemy().map(|a| a.id), // Default to nearest
    }
}

pub fn execute_apl(actor: &Actor, actors: &[Actor], rng: &mut impl rand::Rng) -> TurnActions {
    let ctx = AplContext { actor, actors };

    // Default APL if none specified
    let default_apl = vec![
        AplEntry {
            action: "attack".to_string(),
            condition: Some("enemy.in_range".to_string()),
            target: Some("nearest_enemy".to_string()),
        },
        AplEntry {
            action: "move".to_string(),
            condition: None,
            target: Some("nearest_enemy".to_string()),
        },
    ];

    let apl = if actor.apl.is_empty() { &default_apl } else { &actor.apl };

    let mut move_action = MoveAction::None;
    let mut attack_action = AttackAction::None;

    // Find the first valid move action and first valid attack action
    for entry in apl {
        // Check condition
        let condition_met = entry
            .condition
            .as_ref()
            .map(|c| evaluate_condition(c, &ctx))
            .unwrap_or(true);

        if !condition_met {
            continue;
        }

        match entry.action.to_lowercase().as_str() {
            "attack" => {
                // Only set attack if we haven't found one yet
                if matches!(attack_action, AttackAction::None) && ctx.has_enemy_in_range() {
                    let target_str = entry.target.as_deref().unwrap_or("nearest_enemy");
                    let in_range: Vec<_> = ctx.enemies_in_range().collect();
                    let target = match target_str.to_lowercase().as_str() {
                        "lowest_hp_enemy" | "lowest_hp" | "weakest" => {
                            in_range.iter().min_by_key(|e| e.current_hp).map(|a| a.id)
                        }
                        "random_enemy" | "random" => {
                            if in_range.is_empty() {
                                None
                            } else {
                                Some(in_range[rng.gen_range(0..in_range.len())].id)
                            }
                        }
                        _ => in_range.first().map(|a| a.id),
                    };

                    if let Some(target_id) = target {
                        attack_action = AttackAction::Attack { target_id };
                    }
                }
            }
            "move" => {
                // Only set move if we haven't found one yet
                if matches!(move_action, MoveAction::None) {
                    let target_str = entry.target.as_deref().unwrap_or("nearest_enemy");
                    match target_str.to_lowercase().as_str() {
                        "forward" => {
                            move_action = MoveAction::Move { direction: MoveDirection::Forward };
                        }
                        "backward" => {
                            move_action = MoveAction::Move { direction: MoveDirection::Backward };
                        }
                        _ => {
                            if let Some(target_id) = resolve_target(target_str, &ctx, rng) {
                                move_action = MoveAction::Move {
                                    direction: MoveDirection::Toward(target_id),
                                };
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // If we've found both actions, we can stop
        if !matches!(move_action, MoveAction::None) && !matches!(attack_action, AttackAction::None) {
            break;
        }
    }

    TurnActions {
        move_action,
        attack_action,
    }
}
