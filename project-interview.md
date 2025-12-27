# Project Interview: OSR Game Simulator

## Q1: What is the core idea or concept for your project?

**Answer:** A generic OSR (Old School Renaissance) game simulator, similar to how SimulationCraft works for WoW. The goal is to eventually handle multiple rulesets, but the MVP will start small and abstract - no spells, simple actor behavior.

---

## Q2: What's the primary use case?

**Answer:** All of the above - player optimization and GM balancing. It's generally nice to see how impactful formations, gearing decisions, feat choices, and attributes are. It's also nice to get a sense of how deadly an encounter is.

---

## Q3: What kind of output do you envision?

**Answer:**
- **Statistics over many runs:** average damage, TPK rate, rounds to victory, average number of casualties, rate of victory without casualty, raw HP lost, and percent HP loss
- **Sample combat logs:** for debugging purposes
- **Visualizations:** of sample combats for the report
- **Architecture:** Web frontend making calls to a Rust backend for performance

---

## Q4: What core combat mechanics should the MVP model?

**Answer:**
- **Turn order:** Initiative system
- **Attack resolution:** To-hit rolls, AC, damage
- **Actor behavior:** Action priority lists (like SimC's APL system)
- **Positioning:** Zone-based (not grid-based), with a linear arrangement:
  ```
  Side 1 Ranged → Side 1 Reach → Side 1 Melee → Side 2 Melee → Side 2 Reach → Side 2 Ranged
  ```

---

## Q5: What attributes should an actor have in the MVP?

**Answer:** The basics are good enough:
- HP
- AC
- Attack bonus
- Damage dice
- Movement speed (zones per turn)
- Weapon range (melee/reach/ranged)

---

## Q6: How should the action priority list (APL) work?

**Answer:** Same model as SimC's APL:
- Executes top-to-bottom, running the first action whose conditions are met
- Actions can be guarded with if-statement conditions
- Should be configurable per actor
- Wants a **nicer/cleaner syntax** than SimC's `actions+=/ability,if=condition` format

---

## Q7: What syntax style for APLs?

**Answer:** YAML-style syntax preferred:
```yaml
- action: execute
  if: target.health_percent < 20
```

---

## Q8: How should actors and encounters be defined?

**Answer:** Also YAML, for example:
```yaml
actor:
  name: Fighter
  hp: 10
  ac: 16
  attack_bonus: 5
  damage: 1d8+3
  speed: 1
  range: melee
  apl:
    - action: attack
      if: enemy.in_range
    - action: move
      target: nearest_enemy
```

---

## Q9: What's the frontend tech stack and architecture?

**Answer:**
- **Framework:** React
- **Communication:** REST API
- **Flow:** React frontend generates YAML configs → sends to Rust backend → backend runs simulation → returns results → React displays them

---

## Q10: How many simulation iterations per request?

**Answer:** Configurable, defaulting to 30,000 iterations.

---

## Q11: How should initiative/turn order work?

**Answer:** Configurable in YAML. Actors are "granted actions" - this is typically done via initiative, but spell effects can also grant actions. Keeps the system flexible for different rulesets.

---

## Q12: What targeting options should APLs support?

**Answer:** All of the following:
- Nearest enemy
- Lowest HP enemy
- Highest threat
- Random
- Ally targeting (for future healing/buff support)

---

## Q13: What's the MVP scope?

**Answer:** The following is sufficient for MVP:
- Basic actor stats (HP, AC, attack bonus, damage, speed, range)
- Zone-based positioning (6-zone linear system)
- Attack resolution (to-hit vs AC, damage)
- APL system with YAML syntax
- Rust backend with REST API
- React frontend with YAML generation
- Statistical output (TPK rate, avg casualties, victory rate, HP loss, rounds to victory, etc.)
- Sample combat logs for debugging
- Combat visualizations

---

## Q14: What's the project name?

**Answer:** Tunnel Fight

---

## Q15: Anything else to mention?

**Answer:** Good for now - ready to proceed.

---

