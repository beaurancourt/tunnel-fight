# Tunnel Fight

A generic OSR (Old School Renaissance) game simulator, similar to SimulationCraft for WoW. Simulates combat encounters to help players optimize builds and help GMs balance encounters.

## Architecture

- **Backend:** Rust REST API for performance (30k iterations default)
- **Frontend:** React app that generates YAML configs and displays results

## Core Mechanics

### Zone-Based Positioning
Linear 6-zone system (no grid):
```
Side 1 Ranged → Side 1 Reach → Side 1 Melee → Side 2 Melee → Side 2 Reach → Side 2 Ranged
```

### Zone Capacity
Each zone has a configurable capacity limit:
- **Ranged zones:** Infinite by default (null/~)
- **Reach zones:** 3 by default
- **Melee zones:** 3 by default

Actors cannot move into a full zone. Configure in YAML:
```yaml
zone_capacity:
  ranged: ~      # null = infinite
  reach: 3
  melee: 3
```

### Actor Attributes
- HP (fixed number or dice like "1d8+2")
- AC, attack bonus, damage dice
- Movement speed (zones per turn)
- Weapon range (melee/reach/ranged)
- Starting zone (ranged/reach/melee) - defaults to ranged

### Action Priority Lists (APL)

APLs define actor behavior. Each turn, an actor gets **1 move action** and **1 attack action**. The APL is scanned top-to-bottom to find the first valid move and the first valid attack. Move executes first, then attack is re-evaluated (so you can move into range and attack).

If no APL is specified, actors use a default: attack nearest enemy if in range, move toward nearest enemy.

#### Syntax
```yaml
apl:
  - action: attack
    if: enemy.in_range
    target: lowest_hp_enemy
  - action: move
    target: nearest_enemy
```

#### Actions

| Action   | Description                          | Target Required |
|----------|--------------------------------------|-----------------|
| `attack` | Attack an enemy (must be in range)   | Yes             |
| `move`   | Move toward a target or direction    | Yes             |

#### Conditions (`if`)

| Condition                    | Description                              |
|------------------------------|------------------------------------------|
| `enemy.in_range`             | True if any enemy is within weapon range |
| `!enemy.in_range`            | True if no enemies are in range          |
| `self.health_percent < N`    | True if HP% is below N                   |
| `self.health_percent > N`    | True if HP% is above N                   |
| `self.hp < N`                | True if current HP is below N            |
| `self.hp > N`                | True if current HP is above N            |
| `enemy.count < N`            | True if fewer than N enemies alive       |
| `enemy.count > N`            | True if more than N enemies alive        |
| `ally.count < N`             | True if fewer than N allies alive        |
| `ally.count > N`             | True if more than N allies alive         |
| `true` (or omit `if`)        | Always true                              |
| `false`                      | Never true (skip this entry)             |

#### Targets

| Target                              | For `attack`                      | For `move`                    |
|-------------------------------------|-----------------------------------|-------------------------------|
| `nearest_enemy` / `nearest`         | Attack nearest enemy in range     | Move toward nearest enemy     |
| `lowest_hp_enemy` / `lowest_hp` / `weakest` | Attack weakest enemy in range | Move toward weakest enemy     |
| `random_enemy` / `random`           | Attack random enemy in range      | Move toward random enemy      |
| `forward`                           | N/A                               | Move toward enemy side        |
| `backward`                          | N/A                               | Move toward own ranged zone   |

#### Default APL
If no APL is specified, actors use:
```yaml
apl:
  - action: attack
    if: enemy.in_range
    target: nearest_enemy
  - action: move
    target: nearest_enemy
```

### Initiative
Configurable - actors are "granted actions" via initiative or spell effects. Currently uses random turn order each round.

## Configuration Format

Actors and encounters defined in YAML:
```yaml
actor:
  name: Fighter
  hp: 10
  ac: 16
  attack_bonus: 5
  damage: 1d8+3
  speed: 1
  range: melee
  start_zone: melee    # optional: ranged (default), reach, melee
  apl:
    - action: attack
      if: enemy.in_range
    - action: move
      target: nearest_enemy
```

## Output

### Statistics (over many runs)
- TPK rate
- Average rounds to victory
- Average casualties
- Victory without casualty rate
- Raw HP lost
- Percent HP loss

### Debugging
- Sample combat logs
- Combat visualizations

## MVP Scope

- Basic actor stats and zone positioning
- Attack resolution (to-hit vs AC, damage)
- APL system with YAML syntax
- Rust backend with REST API
- React frontend with YAML generation
- Statistical output and sample combat logs

## Future Considerations

- Multiple rulesets support
- Spells and special abilities
- More complex actor behaviors
