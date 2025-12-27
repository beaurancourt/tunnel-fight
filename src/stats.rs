use serde::Serialize;

use crate::combat::{CombatResult, EventType};
use crate::types::Side;

#[derive(Debug, Clone, Serialize)]
pub struct SimulationStats {
    pub iterations: u32,
    pub side1_win_rate: f64,
    pub side2_win_rate: f64,
    pub draw_rate: f64,
    pub avg_rounds: f64,
    pub avg_side1_casualties: f64,
    pub avg_side2_casualties: f64,
    pub side1_flawless_rate: f64,
    pub side2_flawless_rate: f64,
    pub avg_side1_hp_lost: f64,
    pub avg_side2_hp_lost: f64,
    pub avg_side1_hp_lost_percent: f64,
    pub avg_side2_hp_lost_percent: f64,
    pub side1_tpk_rate: f64,
    pub side2_tpk_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimulationResult {
    pub stats: SimulationStats,
    pub sample_combats: Vec<CombatLog>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CombatLog {
    pub winner: Option<String>,
    pub rounds: u32,
    pub events: Vec<CombatLogEntry>,
    pub final_state: Vec<ActorFinalState>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CombatLogEntry {
    pub round: u32,
    pub actor: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActorFinalState {
    pub name: String,
    pub side: String,
    pub hp: String,
    pub alive: bool,
    pub zone: String,
}

pub struct StatsCollector {
    results: Vec<CombatResult>,
    side1_total_actors: usize,
    side2_total_actors: usize,
    side1_total_hp: i32,
    side2_total_hp: i32,
}

impl StatsCollector {
    pub fn new(side1_count: usize, side2_count: usize, side1_hp: i32, side2_hp: i32) -> Self {
        StatsCollector {
            results: Vec::new(),
            side1_total_actors: side1_count,
            side2_total_actors: side2_count,
            side1_total_hp: side1_hp,
            side2_total_hp: side2_hp,
        }
    }

    pub fn add_result(&mut self, result: CombatResult) {
        self.results.push(result);
    }

    pub fn compute_stats(&self) -> SimulationStats {
        let n = self.results.len() as f64;
        if n == 0.0 {
            return SimulationStats {
                iterations: 0,
                side1_win_rate: 0.0,
                side2_win_rate: 0.0,
                draw_rate: 0.0,
                avg_rounds: 0.0,
                avg_side1_casualties: 0.0,
                avg_side2_casualties: 0.0,
                side1_flawless_rate: 0.0,
                side2_flawless_rate: 0.0,
                avg_side1_hp_lost: 0.0,
                avg_side2_hp_lost: 0.0,
                avg_side1_hp_lost_percent: 0.0,
                avg_side2_hp_lost_percent: 0.0,
                side1_tpk_rate: 0.0,
                side2_tpk_rate: 0.0,
            };
        }

        let mut side1_wins = 0;
        let mut side2_wins = 0;
        let mut draws = 0;
        let mut total_rounds = 0;
        let mut side1_casualties = 0;
        let mut side2_casualties = 0;
        let mut side1_flawless = 0;
        let mut side2_flawless = 0;
        let mut side1_hp_lost = 0;
        let mut side2_hp_lost = 0;
        let mut side1_tpk = 0;
        let mut side2_tpk = 0;

        for result in &self.results {
            total_rounds += result.rounds;

            match result.winner {
                Some(Side::Side1) => side1_wins += 1,
                Some(Side::Side2) => side2_wins += 1,
                None => draws += 1,
            }

            let mut s1_dead = 0;
            let mut s2_dead = 0;
            let mut s1_hp_loss = 0;
            let mut s2_hp_loss = 0;

            for actor in &result.final_state {
                let hp_lost = actor.max_hp - actor.final_hp.max(0);
                match actor.side {
                    Side::Side1 => {
                        s1_hp_loss += hp_lost;
                        if !actor.alive {
                            s1_dead += 1;
                        }
                    }
                    Side::Side2 => {
                        s2_hp_loss += hp_lost;
                        if !actor.alive {
                            s2_dead += 1;
                        }
                    }
                }
            }

            side1_casualties += s1_dead;
            side2_casualties += s2_dead;
            side1_hp_lost += s1_hp_loss;
            side2_hp_lost += s2_hp_loss;

            if s1_dead == 0 && result.winner == Some(Side::Side1) {
                side1_flawless += 1;
            }
            if s2_dead == 0 && result.winner == Some(Side::Side2) {
                side2_flawless += 1;
            }

            if s1_dead == self.side1_total_actors {
                side1_tpk += 1;
            }
            if s2_dead == self.side2_total_actors {
                side2_tpk += 1;
            }
        }

        SimulationStats {
            iterations: self.results.len() as u32,
            side1_win_rate: side1_wins as f64 / n * 100.0,
            side2_win_rate: side2_wins as f64 / n * 100.0,
            draw_rate: draws as f64 / n * 100.0,
            avg_rounds: total_rounds as f64 / n,
            avg_side1_casualties: side1_casualties as f64 / n,
            avg_side2_casualties: side2_casualties as f64 / n,
            side1_flawless_rate: side1_flawless as f64 / n * 100.0,
            side2_flawless_rate: side2_flawless as f64 / n * 100.0,
            avg_side1_hp_lost: side1_hp_lost as f64 / n,
            avg_side2_hp_lost: side2_hp_lost as f64 / n,
            avg_side1_hp_lost_percent: if self.side1_total_hp > 0 {
                (side1_hp_lost as f64 / n) / self.side1_total_hp as f64 * 100.0
            } else {
                0.0
            },
            avg_side2_hp_lost_percent: if self.side2_total_hp > 0 {
                (side2_hp_lost as f64 / n) / self.side2_total_hp as f64 * 100.0
            } else {
                0.0
            },
            side1_tpk_rate: side1_tpk as f64 / n * 100.0,
            side2_tpk_rate: side2_tpk as f64 / n * 100.0,
        }
    }

    pub fn get_sample_combats(&self, count: usize) -> Vec<CombatLog> {
        self.results
            .iter()
            .take(count)
            .map(|r| format_combat_log(r))
            .collect()
    }
}

fn format_combat_log(result: &CombatResult) -> CombatLog {
    let events: Vec<CombatLogEntry> = result
        .events
        .iter()
        .map(|e| {
            let description = match &e.event_type {
                EventType::Attack {
                    target_name,
                    roll,
                    target_ac,
                    hit,
                    damage,
                    ..
                } => {
                    if *hit {
                        format!(
                            "attacks {} (rolled {} vs AC {}) - HIT for {} damage",
                            target_name, roll, target_ac, damage
                        )
                    } else {
                        format!(
                            "attacks {} (rolled {} vs AC {}) - MISS",
                            target_name, roll, target_ac
                        )
                    }
                }
                EventType::Move { from, to } => {
                    format!("moves from {:?} to {:?}", from, to)
                }
                EventType::Death { killer_id: _ } => "dies!".to_string(),
            };

            CombatLogEntry {
                round: e.round,
                actor: e.actor_name.clone(),
                description,
            }
        })
        .collect();

    let final_state: Vec<ActorFinalState> = result
        .final_state
        .iter()
        .map(|a| ActorFinalState {
            name: a.name.clone(),
            side: format!("{:?}", a.side),
            hp: format!("{}/{}", a.final_hp.max(0), a.max_hp),
            alive: a.alive,
            zone: format!("{:?}", a.zone),
        })
        .collect();

    CombatLog {
        winner: result.winner.map(|s| format!("{:?}", s)),
        rounds: result.rounds,
        events,
        final_state,
    }
}
