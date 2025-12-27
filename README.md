# Tunnel Fight

A generic OSR (Old School Renaissance) combat simulator. Like [SimulationCraft](https://www.simulationcraft.org/) for tabletop RPGs.

Run thousands of combat simulations to analyze encounter balance, optimize character builds, and test formations.

**Try it now: https://tunnel-fight.web.app**

## Features

- **Zone-based positioning**: Linear 6-zone combat system (ranged → reach → melee)
- **Action Priority Lists**: Configurable AI behavior with conditions and targeting
- **Dice-based stats**: HP, damage, and other values support dice notation (e.g., `1d8+2`)
- **Detailed statistics**: Win rates, TPK rates, casualties, HP loss, rounds to victory
- **Sample combat logs**: Debug and visualize individual fights
- **Fast**: Rust backend runs 30k iterations in seconds

## Quick Start

### Backend

```bash
cargo run
```

Server starts at http://localhost:3000

### Frontend

```bash
cd frontend
npm install
npm run dev
```

Open http://localhost:5173

## Configuration

Encounters are defined in YAML:

```yaml
name: Party vs Goblins
iterations: 10000

zone_capacity:
  ranged: ~      # null = infinite
  reach: 3
  melee: 3

side1:
  - name: Fighter
    hp: 1d8+2
    ac: 16
    attack_bonus: 5
    damage: 1d8+3
    speed: 1
    range: melee
    start_zone: melee
    apl:
      - action: attack
        if: enemy.in_range
        target: lowest_hp_enemy
      - action: move
        target: nearest_enemy

side2:
  - name: Goblin
    hp: 7
    ac: 13
    attack_bonus: 4
    damage: 1d6+2
    speed: 1
    range: melee
```

## Action Priority Lists (APL)

Each turn, actors get 1 move + 1 attack. The APL is scanned to find the first valid action of each type.

### Actions

| Action   | Description                        |
|----------|------------------------------------|
| `attack` | Attack an enemy (must be in range) |
| `move`   | Move toward a target or direction  |

### Conditions

| Condition                 | Description                    |
|---------------------------|--------------------------------|
| `enemy.in_range`          | Any enemy within weapon range  |
| `!enemy.in_range`         | No enemies in range            |
| `self.health_percent < N` | HP% below N                    |
| `self.hp < N`             | Current HP below N             |
| `enemy.count < N`         | Fewer than N enemies alive     |
| `ally.count < N`          | Fewer than N allies alive      |

### Targets

| Target            | Description              |
|-------------------|--------------------------|
| `nearest_enemy`   | Closest enemy            |
| `lowest_hp_enemy` | Enemy with least HP      |
| `random_enemy`    | Random enemy             |
| `forward`         | Move toward enemy side   |
| `backward`        | Move toward own side     |

## License

MIT
