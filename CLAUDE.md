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

### Zone Capacity & Frontage
Zones have capacity in "frontage units". Each actor has a frontage (default 3) that determines how much space they occupy. An actor can only enter a zone if their frontage fits in the remaining capacity.

**Default capacities:**
- **Ranged zones:** Infinite (null/~)
- **Reach zones:** 10
- **Melee zones:** 10

**Example:** With melee capacity 10:
- 3 fighters (frontage 3 each) = 9 frontage, fits
- 2 zombies (frontage 5 each) = 10 frontage, fits
- 3 zombies would need 15 frontage, doesn't fit

```yaml
zone_capacity:
  ranged: ~      # null = infinite
  reach: 10
  melee: 10
```

### Actor Attributes
- HP (fixed number or dice like "1d8+2")
- AC, attack bonus, damage dice
- Movement speed (zones per turn)
- Weapon range (melee/reach/ranged)
- Starting zone (ranged/reach/melee) - defaults to ranged
- Frontage (default 3) - space occupied in a zone

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
| `guard`  | Raise AC by 2 until next turn        | No              |
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
  frontage: 3          # optional: space occupied (default: 3)
  apl:
    - action: attack
      if: enemy.in_range
    - action: move
      target: nearest_enemy
```

## Example: Spearwall Formation

A classic defensive formation: shield-bearers hold the front line while polearms strike from behind.

```yaml
name: Spearwall vs Zombies
iterations: 10000

zone_capacity:
  ranged: ~
  reach: 10
  melee: 10

side1:
  # Front line - shield wall (guard only, AC 21 when guarding)
  - name: Mace 1
    hp: 1d6
    ac: 19
    attack_bonus: 0
    damage: 1d6
    range: melee
    start_zone: melee
    frontage: 3
    apl:
      - action: guard

  - name: Mace 2
    hp: 1d6
    ac: 19
    attack_bonus: 0
    damage: 1d6
    range: melee
    start_zone: melee
    frontage: 3
    apl:
      - action: guard

  - name: Mace 3
    hp: 1d6
    ac: 19
    attack_bonus: 0
    damage: 1d6
    range: melee
    start_zone: melee
    frontage: 3
    apl:
      - action: guard

  # Polearm line - attack from reach, only advance if maces are dead
  - name: Polearm 1
    hp: 1d6
    ac: 14
    attack_bonus: 2
    damage: 1d8+2
    range: reach
    start_zone: reach
    frontage: 3
    apl:
      - action: attack
        if: enemy.in_range
        target: nearest_enemy
      - action: move
        if: ally.count < 3
        target: nearest_enemy

  - name: Polearm 2
    hp: 1d6
    ac: 14
    attack_bonus: 2
    damage: 1d8+2
    range: reach
    start_zone: reach
    frontage: 3
    apl:
      - action: attack
        if: enemy.in_range
        target: nearest_enemy
      - action: move
        if: ally.count < 3
        target: nearest_enemy

  - name: Polearm 3
    hp: 1d6
    ac: 14
    attack_bonus: 2
    damage: 1d8+2
    range: reach
    start_zone: reach
    frontage: 3
    apl:
      - action: attack
        if: enemy.in_range
        target: nearest_enemy
      - action: move
        if: ally.count < 3
        target: nearest_enemy

side2:
  # Zombies with frontage 5 - only 2 can engage at once
  # 2 in melee (already engaging)
  - name: Zombie 1
    hp: 2d8
    ac: 12
    attack_bonus: 1
    damage: 1d8
    frontage: 5
    start_zone: melee
    apl:
      - action: attack
        if: enemy.in_range
        target: random_enemy
      - action: move
        target: nearest_enemy

  - name: Zombie 2
    hp: 2d8
    ac: 12
    attack_bonus: 1
    damage: 1d8
    frontage: 5
    start_zone: melee
    apl:
      - action: attack
        if: enemy.in_range
        target: random_enemy
      - action: move
        target: nearest_enemy

  # 2 in reach (next wave)
  - name: Zombie 3
    hp: 2d8
    ac: 12
    attack_bonus: 1
    damage: 1d8
    frontage: 5
    start_zone: reach
    apl:
      - action: attack
        if: enemy.in_range
        target: random_enemy
      - action: move
        target: nearest_enemy

  - name: Zombie 4
    hp: 2d8
    ac: 12
    attack_bonus: 1
    damage: 1d8
    frontage: 5
    start_zone: reach
    apl:
      - action: attack
        if: enemy.in_range
        target: random_enemy
      - action: move
        target: nearest_enemy

  # 6 in ranged (reserves)
  - name: Zombie 5
    hp: 2d8
    ac: 12
    attack_bonus: 1
    damage: 1d8
    frontage: 5
    apl:
      - action: attack
        if: enemy.in_range
        target: random_enemy
      - action: move
        target: nearest_enemy

  - name: Zombie 6
    hp: 2d8
    ac: 12
    attack_bonus: 1
    damage: 1d8
    frontage: 5
    apl:
      - action: attack
        if: enemy.in_range
        target: random_enemy
      - action: move
        target: nearest_enemy

  - name: Zombie 7
    hp: 2d8
    ac: 12
    attack_bonus: 1
    damage: 1d8
    frontage: 5
    apl:
      - action: attack
        if: enemy.in_range
        target: random_enemy
      - action: move
        target: nearest_enemy

  - name: Zombie 8
    hp: 2d8
    ac: 12
    attack_bonus: 1
    damage: 1d8
    frontage: 5
    apl:
      - action: attack
        if: enemy.in_range
        target: random_enemy
      - action: move
        target: nearest_enemy

  - name: Zombie 9
    hp: 2d8
    ac: 12
    attack_bonus: 1
    damage: 1d8
    frontage: 5
    apl:
      - action: attack
        if: enemy.in_range
        target: random_enemy
      - action: move
        target: nearest_enemy

  - name: Zombie 10
    hp: 2d8
    ac: 12
    attack_bonus: 1
    damage: 1d8
    frontage: 5
    apl:
      - action: attack
        if: enemy.in_range
        target: random_enemy
      - action: move
        target: nearest_enemy
```

**Key tactics:**
- **Maces guard** (AC 21) - zombies need natural 20 to hit
- **Polearms attack from reach** - safe behind the shield wall
- **Frontage limits engagement** - only 2 zombies (frontage 5) fit in melee at once
- **Polearms hold position** (`ally.count < 3`) - only advance when all maces are dead

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
