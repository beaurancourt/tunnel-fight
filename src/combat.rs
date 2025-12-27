use rand::Rng;

use crate::apl::{execute_apl, AttackAction, MoveAction, MoveDirection};
use crate::types::{Actor, DamageDice, Encounter, InitiativeType, Phase, Side, WeaponRange, Zone, ZoneCapacities, parse_damage_dice};

#[derive(Debug, Clone)]
pub struct CombatEvent {
    pub round: u32,
    pub actor_id: usize,
    pub actor_name: String,
    pub event_type: EventType,
}

#[derive(Debug, Clone)]
pub enum EventType {
    Attack {
        target_id: usize,
        target_name: String,
        roll: i32,
        target_ac: i32,
        hit: bool,
        damage: i32,
    },
    Move {
        from: Zone,
        to: Zone,
    },
    Death {
        killer_id: Option<usize>,
    },
}

#[derive(Debug, Clone)]
pub struct CombatResult {
    pub winner: Option<Side>,
    pub rounds: u32,
    pub events: Vec<CombatEvent>,
    pub final_state: Vec<ActorState>,
}

#[derive(Debug, Clone)]
pub struct ActorState {
    pub id: usize,
    pub name: String,
    pub side: Side,
    pub max_hp: i32,
    pub final_hp: i32,
    pub alive: bool,
    pub zone: Zone,
}

pub struct CombatSimulator {
    actors: Vec<Actor>,
    events: Vec<CombatEvent>,
    round: u32,
    max_rounds: u32,
    zone_capacity: ZoneCapacities,
    initiative_type: InitiativeType,
    initiative_dice: DamageDice,
    phases: Vec<Phase>,
}

impl CombatSimulator {
    pub fn new(encounter: &Encounter, max_rounds: u32, rng: &mut impl Rng) -> Self {
        let mut actors = Vec::new();
        let mut id = 0;

        for template in &encounter.side1 {
            actors.push(Actor::from_template(id, template, Side::Side1, rng));
            id += 1;
        }

        for template in &encounter.side2 {
            actors.push(Actor::from_template(id, template, Side::Side2, rng));
            id += 1;
        }

        let initiative_dice = parse_damage_dice(&encounter.initiative.dice)
            .unwrap_or(DamageDice { count: 1, sides: 20, modifier: 0 });

        CombatSimulator {
            actors,
            events: Vec::new(),
            round: 0,
            max_rounds,
            zone_capacity: encounter.zone_capacity.clone(),
            initiative_type: encounter.initiative.initiative_type,
            initiative_dice,
            phases: encounter.initiative.phases.clone(),
        }
    }

    fn zone_has_capacity(&self, zone: Zone, exclude_actor_id: usize) -> bool {
        let capacity = self.zone_capacity.capacity_for(zone);
        match capacity {
            None => true, // Infinite capacity
            Some(cap) => {
                let count = self
                    .actors
                    .iter()
                    .filter(|a| a.zone == zone && a.is_alive() && a.id != exclude_actor_id)
                    .count() as u32;
                count < cap
            }
        }
    }

    fn zone_has_enemies(&self, zone: Zone, actor_side: Side) -> bool {
        self.actors
            .iter()
            .any(|a| a.zone == zone && a.is_alive() && a.side != actor_side)
    }

    fn can_enter_zone(&self, zone: Zone, actor_id: usize, actor_side: Side) -> bool {
        self.zone_has_capacity(zone, actor_id) && !self.zone_has_enemies(zone, actor_side)
    }

    pub fn run(&mut self, rng: &mut impl Rng) -> CombatResult {
        while !self.is_combat_over() && self.round < self.max_rounds {
            self.round += 1;
            match self.initiative_type {
                InitiativeType::Side => self.run_round_side(rng),
                InitiativeType::Individual => self.run_round_individual(rng),
                InitiativeType::SidePhases => self.run_round_side_phases(rng),
                InitiativeType::IndividualPhases => self.run_round_individual_phases(rng),
            }
        }

        CombatResult {
            winner: self.get_winner(),
            rounds: self.round,
            events: self.events.clone(),
            final_state: self
                .actors
                .iter()
                .map(|a| ActorState {
                    id: a.id,
                    name: a.name.clone(),
                    side: a.side,
                    max_hp: a.max_hp,
                    final_hp: a.current_hp,
                    alive: a.is_alive(),
                    zone: a.zone,
                })
                .collect(),
        }
    }

    /// Side-based initiative: one side acts completely, then the other
    fn run_round_side(&mut self, rng: &mut impl Rng) {
        // Determine which side goes first (50/50)
        let first_side = if rng.gen_bool(0.5) { Side::Side1 } else { Side::Side2 };
        let second_side = first_side.opposite();

        for side in [first_side, second_side] {
            // Get actors for this side, shuffled
            let mut order: Vec<usize> = self
                .actors
                .iter()
                .filter(|a| a.is_alive() && a.side == side)
                .map(|a| a.id)
                .collect();

            // Fisher-Yates shuffle
            for i in (1..order.len()).rev() {
                let j = rng.gen_range(0..=i);
                order.swap(i, j);
            }

            for actor_id in order {
                self.execute_full_turn(actor_id, rng);
                if self.is_combat_over() {
                    return;
                }
            }
        }
    }

    /// Individual initiative: each actor rolls initiative dice + modifier
    fn run_round_individual(&mut self, rng: &mut impl Rng) {
        // Roll initiative for each actor
        let mut initiatives: Vec<(usize, i32)> = self
            .actors
            .iter()
            .filter(|a| a.is_alive())
            .map(|a| {
                let roll = self.initiative_dice.roll(rng) + a.initiative_modifier;
                (a.id, roll)
            })
            .collect();

        // Sort by initiative (highest first), with random tiebreaker
        initiatives.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| {
            if rng.gen_bool(0.5) { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }
        }));

        for (actor_id, _) in initiatives {
            if !self.actors[actor_id].is_alive() {
                continue;
            }
            self.execute_full_turn(actor_id, rng);
            if self.is_combat_over() {
                return;
            }
        }
    }

    /// Side-based phases: each phase executes for both sides before moving to the next
    fn run_round_side_phases(&mut self, rng: &mut impl Rng) {
        // Determine which side goes first (50/50)
        let first_side = if rng.gen_bool(0.5) { Side::Side1 } else { Side::Side2 };
        let second_side = first_side.opposite();

        for phase in self.phases.clone() {
            match phase {
                Phase::Movement => {
                    for side in [first_side, second_side] {
                        let order = self.get_shuffled_side_order(side, rng);
                        for actor_id in order {
                            self.execute_movement_only(actor_id, rng);
                        }
                    }
                }
                Phase::Ranged => {
                    for side in [first_side, second_side] {
                        let order = self.get_shuffled_side_order(side, rng);
                        for actor_id in order {
                            if self.actors[actor_id].range == WeaponRange::Ranged {
                                self.execute_attack_only(actor_id, rng);
                                if self.is_combat_over() { return; }
                            }
                        }
                    }
                }
                Phase::Reach => {
                    for side in [first_side, second_side] {
                        let order = self.get_shuffled_side_order(side, rng);
                        for actor_id in order {
                            if self.actors[actor_id].range == WeaponRange::Reach {
                                self.execute_attack_only(actor_id, rng);
                                if self.is_combat_over() { return; }
                            }
                        }
                    }
                }
                Phase::Melee => {
                    for side in [first_side, second_side] {
                        let order = self.get_shuffled_side_order(side, rng);
                        for actor_id in order {
                            if self.actors[actor_id].range == WeaponRange::Melee {
                                self.execute_attack_only(actor_id, rng);
                                if self.is_combat_over() { return; }
                            }
                        }
                    }
                }
            }
            if self.is_combat_over() { return; }
        }
    }

    /// Individual phases: each phase executes in initiative order before moving to the next
    fn run_round_individual_phases(&mut self, rng: &mut impl Rng) {
        // Roll initiative for each actor
        let mut initiatives: Vec<(usize, i32)> = self
            .actors
            .iter()
            .filter(|a| a.is_alive())
            .map(|a| {
                let roll = self.initiative_dice.roll(rng) + a.initiative_modifier;
                (a.id, roll)
            })
            .collect();

        // Sort by initiative (highest first)
        initiatives.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| {
            if rng.gen_bool(0.5) { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }
        }));

        let order: Vec<usize> = initiatives.iter().map(|(id, _)| *id).collect();

        for phase in self.phases.clone() {
            match phase {
                Phase::Movement => {
                    for &actor_id in &order {
                        if self.actors[actor_id].is_alive() {
                            self.execute_movement_only(actor_id, rng);
                        }
                    }
                }
                Phase::Ranged => {
                    for &actor_id in &order {
                        if self.actors[actor_id].is_alive() && self.actors[actor_id].range == WeaponRange::Ranged {
                            self.execute_attack_only(actor_id, rng);
                            if self.is_combat_over() { return; }
                        }
                    }
                }
                Phase::Reach => {
                    for &actor_id in &order {
                        if self.actors[actor_id].is_alive() && self.actors[actor_id].range == WeaponRange::Reach {
                            self.execute_attack_only(actor_id, rng);
                            if self.is_combat_over() { return; }
                        }
                    }
                }
                Phase::Melee => {
                    for &actor_id in &order {
                        if self.actors[actor_id].is_alive() && self.actors[actor_id].range == WeaponRange::Melee {
                            self.execute_attack_only(actor_id, rng);
                            if self.is_combat_over() { return; }
                        }
                    }
                }
            }
            if self.is_combat_over() { return; }
        }
    }

    fn get_shuffled_side_order(&self, side: Side, rng: &mut impl Rng) -> Vec<usize> {
        let mut order: Vec<usize> = self
            .actors
            .iter()
            .filter(|a| a.is_alive() && a.side == side)
            .map(|a| a.id)
            .collect();

        for i in (1..order.len()).rev() {
            let j = rng.gen_range(0..=i);
            order.swap(i, j);
        }
        order
    }

    /// Execute a full turn: move then attack
    fn execute_full_turn(&mut self, actor_id: usize, rng: &mut impl Rng) {
        if !self.actors[actor_id].is_alive() {
            return;
        }

        // Get initial actions based on current state
        let turn_actions = {
            let actor = &self.actors[actor_id];
            execute_apl(actor, &self.actors, rng)
        };

        // Execute move first
        if let MoveAction::Move { direction } = turn_actions.move_action {
            self.execute_move(actor_id, direction);
        }

        // Re-evaluate for attack after moving (position may have changed)
        let attack_action = {
            let actor = &self.actors[actor_id];
            execute_apl(actor, &self.actors, rng).attack_action
        };

        // Execute attack
        if let AttackAction::Attack { target_id } = attack_action {
            self.execute_attack(actor_id, target_id, rng);
        }
    }

    /// Execute only the movement portion of a turn
    fn execute_movement_only(&mut self, actor_id: usize, rng: &mut impl Rng) {
        if !self.actors[actor_id].is_alive() {
            return;
        }

        let turn_actions = {
            let actor = &self.actors[actor_id];
            execute_apl(actor, &self.actors, rng)
        };

        if let MoveAction::Move { direction } = turn_actions.move_action {
            self.execute_move(actor_id, direction);
        }
    }

    /// Execute only the attack portion of a turn
    fn execute_attack_only(&mut self, actor_id: usize, rng: &mut impl Rng) {
        if !self.actors[actor_id].is_alive() {
            return;
        }

        let attack_action = {
            let actor = &self.actors[actor_id];
            execute_apl(actor, &self.actors, rng).attack_action
        };

        if let AttackAction::Attack { target_id } = attack_action {
            self.execute_attack(actor_id, target_id, rng);
        }
    }

    fn execute_attack(&mut self, attacker_id: usize, target_id: usize, rng: &mut impl Rng) {
        let attacker = &self.actors[attacker_id];
        let target = &self.actors[target_id];

        if !attacker.can_attack(target) {
            return;
        }

        let roll = rng.gen_range(1..=20) + attacker.attack_bonus;
        let hit = roll >= target.ac;
        let damage = if hit {
            attacker.damage.roll(rng)
        } else {
            0
        };

        let attacker_name = attacker.name.clone();
        let target_name = target.name.clone();
        let target_ac = target.ac;

        self.events.push(CombatEvent {
            round: self.round,
            actor_id: attacker_id,
            actor_name: attacker_name,
            event_type: EventType::Attack {
                target_id,
                target_name: target_name.clone(),
                roll,
                target_ac,
                hit,
                damage,
            },
        });

        if hit {
            self.actors[target_id].current_hp -= damage;

            if !self.actors[target_id].is_alive() {
                self.events.push(CombatEvent {
                    round: self.round,
                    actor_id: target_id,
                    actor_name: target_name,
                    event_type: EventType::Death {
                        killer_id: Some(attacker_id),
                    },
                });
            }
        }
    }

    fn execute_move(&mut self, actor_id: usize, direction: MoveDirection) {
        let actor = &self.actors[actor_id];
        let from_zone = actor.zone;
        let speed = actor.speed;
        let actor_side = actor.side;

        let to_zone = match direction {
            MoveDirection::Toward(target_id) => {
                let target = &self.actors[target_id];
                let mut current = from_zone;
                for _ in 0..speed {
                    if let Some(next) = current.toward(&target.zone) {
                        if self.can_enter_zone(next, actor_id, actor_side) {
                            current = next;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                current
            }
            MoveDirection::ToZone(zone) => {
                let mut current = from_zone;
                for _ in 0..speed {
                    if let Some(next) = current.toward(&zone) {
                        if self.can_enter_zone(next, actor_id, actor_side) {
                            current = next;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                current
            }
            MoveDirection::Forward => {
                let target_zone = match actor_side {
                    Side::Side1 => Zone::Side2Ranged,
                    Side::Side2 => Zone::Side1Ranged,
                };
                let mut current = from_zone;
                for _ in 0..speed {
                    if let Some(next) = current.toward(&target_zone) {
                        if self.can_enter_zone(next, actor_id, actor_side) {
                            current = next;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                current
            }
            MoveDirection::Backward => {
                let target_zone = match actor_side {
                    Side::Side1 => Zone::Side1Ranged,
                    Side::Side2 => Zone::Side2Ranged,
                };
                let mut current = from_zone;
                for _ in 0..speed {
                    if let Some(next) = current.toward(&target_zone) {
                        if self.can_enter_zone(next, actor_id, actor_side) {
                            current = next;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                current
            }
        };

        if to_zone != from_zone {
            let actor_name = self.actors[actor_id].name.clone();
            self.actors[actor_id].zone = to_zone;

            self.events.push(CombatEvent {
                round: self.round,
                actor_id,
                actor_name,
                event_type: EventType::Move {
                    from: from_zone,
                    to: to_zone,
                },
            });
        }
    }

    fn is_combat_over(&self) -> bool {
        let side1_alive = self
            .actors
            .iter()
            .any(|a| a.side == Side::Side1 && a.is_alive());
        let side2_alive = self
            .actors
            .iter()
            .any(|a| a.side == Side::Side2 && a.is_alive());

        !side1_alive || !side2_alive
    }

    fn get_winner(&self) -> Option<Side> {
        let side1_alive = self
            .actors
            .iter()
            .any(|a| a.side == Side::Side1 && a.is_alive());
        let side2_alive = self
            .actors
            .iter()
            .any(|a| a.side == Side::Side2 && a.is_alive());

        match (side1_alive, side2_alive) {
            (true, false) => Some(Side::Side1),
            (false, true) => Some(Side::Side2),
            _ => None,
        }
    }
}
