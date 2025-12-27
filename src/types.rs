use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Side1,
    Side2,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::Side1 => Side::Side2,
            Side::Side2 => Side::Side1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Zone {
    Side1Ranged,
    Side1Reach,
    Side1Melee,
    Side2Melee,
    Side2Reach,
    Side2Ranged,
}

impl Zone {
    pub fn side(&self) -> Side {
        match self {
            Zone::Side1Ranged | Zone::Side1Reach | Zone::Side1Melee => Side::Side1,
            Zone::Side2Melee | Zone::Side2Reach | Zone::Side2Ranged => Side::Side2,
        }
    }

    pub fn distance_to(&self, other: &Zone) -> u32 {
        let zones = [
            Zone::Side1Ranged,
            Zone::Side1Reach,
            Zone::Side1Melee,
            Zone::Side2Melee,
            Zone::Side2Reach,
            Zone::Side2Ranged,
        ];
        let self_idx = zones.iter().position(|z| z == self).unwrap() as i32;
        let other_idx = zones.iter().position(|z| z == other).unwrap() as i32;
        (self_idx - other_idx).unsigned_abs()
    }

    pub fn toward(&self, target: &Zone) -> Option<Zone> {
        if self == target {
            return None;
        }
        let zones = [
            Zone::Side1Ranged,
            Zone::Side1Reach,
            Zone::Side1Melee,
            Zone::Side2Melee,
            Zone::Side2Reach,
            Zone::Side2Ranged,
        ];
        let self_idx = zones.iter().position(|z| z == self).unwrap();
        let target_idx = zones.iter().position(|z| z == target).unwrap();
        if target_idx > self_idx {
            Some(zones[self_idx + 1])
        } else {
            Some(zones[self_idx - 1])
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponRange {
    Melee,
    Reach,
    Ranged,
}

impl WeaponRange {
    pub fn max_distance(&self) -> u32 {
        match self {
            WeaponRange::Melee => 1,
            WeaponRange::Reach => 2,
            WeaponRange::Ranged => 6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageDice {
    pub count: u32,
    pub sides: u32,
    pub modifier: i32,
}

impl DamageDice {
    pub fn roll(&self, rng: &mut impl rand::Rng) -> i32 {
        let mut total = self.modifier;
        for _ in 0..self.count {
            total += rng.gen_range(1..=self.sides) as i32;
        }
        total.max(0)
    }
}

impl fmt::Display for DamageDice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifier == 0 {
            write!(f, "{}d{}", self.count, self.sides)
        } else if self.modifier > 0 {
            write!(f, "{}d{}+{}", self.count, self.sides, self.modifier)
        } else {
            write!(f, "{}d{}{}", self.count, self.sides, self.modifier)
        }
    }
}

pub fn parse_damage_dice(s: &str) -> Result<DamageDice, String> {
    let s = s.trim().to_lowercase();

    let (dice_part, modifier) = if let Some(idx) = s.find('+') {
        let (dice, mod_str) = s.split_at(idx);
        (dice, mod_str[1..].parse::<i32>().map_err(|e| e.to_string())?)
    } else if let Some(idx) = s.rfind('-') {
        if idx == 0 {
            return Err("Invalid dice format".to_string());
        }
        let (dice, mod_str) = s.split_at(idx);
        (dice, mod_str.parse::<i32>().map_err(|e| e.to_string())?)
    } else {
        (s.as_str(), 0)
    };

    let parts: Vec<&str> = dice_part.split('d').collect();
    if parts.len() != 2 {
        return Err("Invalid dice format: expected NdM".to_string());
    }

    let count = parts[0].parse::<u32>().map_err(|e| e.to_string())?;
    let sides = parts[1].parse::<u32>().map_err(|e| e.to_string())?;

    Ok(DamageDice { count, sides, modifier })
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StartingZone {
    #[default]
    Ranged,
    Reach,
    Melee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HpValue {
    Fixed(i32),
    Dice(String),
}

impl HpValue {
    pub fn roll(&self, rng: &mut impl rand::Rng) -> i32 {
        match self {
            HpValue::Fixed(v) => *v,
            HpValue::Dice(s) => {
                if let Ok(dice) = parse_damage_dice(s) {
                    dice.roll(rng).max(1) // Minimum 1 HP
                } else {
                    1
                }
            }
        }
    }

    pub fn expected_value(&self) -> f64 {
        match self {
            HpValue::Fixed(v) => *v as f64,
            HpValue::Dice(s) => {
                if let Ok(dice) = parse_damage_dice(s) {
                    // Expected value of NdM is N * (M+1) / 2
                    let dice_avg = dice.count as f64 * (dice.sides as f64 + 1.0) / 2.0;
                    (dice_avg + dice.modifier as f64).max(1.0)
                } else {
                    1.0
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorTemplate {
    pub name: String,
    pub hp: HpValue,
    pub ac: i32,
    pub attack_bonus: i32,
    #[serde(deserialize_with = "deserialize_damage_dice")]
    pub damage: DamageDice,
    #[serde(default = "default_speed")]
    pub speed: u32,
    #[serde(default)]
    pub range: WeaponRange,
    #[serde(default)]
    pub start_zone: StartingZone,
    #[serde(default)]
    pub apl: Vec<AplEntry>,
}

fn default_speed() -> u32 {
    1
}

fn deserialize_damage_dice<'de, D>(deserializer: D) -> Result<DamageDice, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_damage_dice(&s).map_err(serde::de::Error::custom)
}

impl Default for WeaponRange {
    fn default() -> Self {
        WeaponRange::Melee
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AplEntry {
    pub action: String,
    #[serde(rename = "if")]
    pub condition: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Actor {
    pub id: usize,
    pub name: String,
    pub side: Side,
    pub max_hp: i32,
    pub current_hp: i32,
    pub ac: i32,
    pub attack_bonus: i32,
    pub damage: DamageDice,
    pub speed: u32,
    pub range: WeaponRange,
    pub zone: Zone,
    pub apl: Vec<AplEntry>,
}

impl Actor {
    pub fn from_template(id: usize, template: &ActorTemplate, side: Side, rng: &mut impl rand::Rng) -> Self {
        let zone = match (side, template.start_zone) {
            (Side::Side1, StartingZone::Ranged) => Zone::Side1Ranged,
            (Side::Side1, StartingZone::Reach) => Zone::Side1Reach,
            (Side::Side1, StartingZone::Melee) => Zone::Side1Melee,
            (Side::Side2, StartingZone::Ranged) => Zone::Side2Ranged,
            (Side::Side2, StartingZone::Reach) => Zone::Side2Reach,
            (Side::Side2, StartingZone::Melee) => Zone::Side2Melee,
        };
        let hp = template.hp.roll(rng);
        Actor {
            id,
            name: template.name.clone(),
            side,
            max_hp: hp,
            current_hp: hp,
            ac: template.ac,
            attack_bonus: template.attack_bonus,
            damage: template.damage.clone(),
            speed: template.speed,
            range: template.range,
            zone,
            apl: template.apl.clone(),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    pub fn can_attack(&self, target: &Actor) -> bool {
        let distance = self.zone.distance_to(&target.zone);
        distance <= self.range.max_distance()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneCapacities {
    #[serde(default = "default_ranged_capacity")]
    pub ranged: Option<u32>,  // None means infinite
    #[serde(default = "default_reach_capacity")]
    pub reach: u32,
    #[serde(default = "default_melee_capacity")]
    pub melee: u32,
}

impl Default for ZoneCapacities {
    fn default() -> Self {
        ZoneCapacities {
            ranged: None,
            reach: 3,
            melee: 3,
        }
    }
}

fn default_ranged_capacity() -> Option<u32> {
    None
}

fn default_reach_capacity() -> u32 {
    3
}

fn default_melee_capacity() -> u32 {
    3
}

impl ZoneCapacities {
    pub fn capacity_for(&self, zone: Zone) -> Option<u32> {
        match zone {
            Zone::Side1Ranged | Zone::Side2Ranged => self.ranged,
            Zone::Side1Reach | Zone::Side2Reach => Some(self.reach),
            Zone::Side1Melee | Zone::Side2Melee => Some(self.melee),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encounter {
    pub name: Option<String>,
    pub side1: Vec<ActorTemplate>,
    pub side2: Vec<ActorTemplate>,
    #[serde(default = "default_iterations")]
    pub iterations: u32,
    #[serde(default)]
    pub zone_capacity: ZoneCapacities,
}

fn default_iterations() -> u32 {
    30000
}
