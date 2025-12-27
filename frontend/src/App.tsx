import { useState } from 'react'
import axios from 'axios'
import './App.css'

interface SimulationStats {
  iterations: number
  side1_win_rate: number
  side2_win_rate: number
  draw_rate: number
  avg_rounds: number
  avg_side1_casualties: number
  avg_side2_casualties: number
  side1_flawless_rate: number
  side2_flawless_rate: number
  avg_side1_hp_lost: number
  avg_side2_hp_lost: number
  avg_side1_hp_lost_percent: number
  avg_side2_hp_lost_percent: number
  side1_tpk_rate: number
  side2_tpk_rate: number
}

interface CombatLogEntry {
  round: number
  actor: string
  description: string
}

interface ActorFinalState {
  name: string
  side: string
  hp: string
  alive: boolean
  zone: string
}

interface CombatLog {
  winner: string | null
  rounds: number
  events: CombatLogEntry[]
  final_state: ActorFinalState[]
}

interface SimulationResult {
  stats: SimulationStats
  sample_combats: CombatLog[]
}

const DEFAULT_ENCOUNTER = `name: Party vs Goblins
iterations: 10000

# Zone capacity (optional - these are the defaults)
zone_capacity:
  ranged: ~      # null = infinite
  reach: 3
  melee: 3

side1:
  - name: Fighter
    hp: 12
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

  - name: Ranger
    hp: 9
    ac: 14
    attack_bonus: 5
    damage: 1d8+2
    speed: 1
    range: ranged
    start_zone: ranged
    apl:
      - action: attack
        if: enemy.in_range
        target: nearest_enemy
      - action: move
        target: nearest_enemy

side2:
  - name: Goblin 1
    hp: 7
    ac: 13
    attack_bonus: 4
    damage: 1d6+2
    speed: 1
    range: melee
    start_zone: melee

  - name: Goblin 2
    hp: 7
    ac: 13
    attack_bonus: 4
    damage: 1d6+2
    speed: 1
    range: melee
    start_zone: melee

  - name: Goblin 3
    hp: 7
    ac: 13
    attack_bonus: 4
    damage: 1d6+2
    speed: 1
    range: melee
    start_zone: melee
`

function Docs({ onClose }: { onClose: () => void }) {
  return (
    <div className="docs-overlay" onClick={onClose}>
      <div className="docs-panel" onClick={e => e.stopPropagation()}>
        <button className="docs-close" onClick={onClose}>×</button>
        <h2>Documentation</h2>

        <section>
          <h3>Zone-Based Positioning</h3>
          <p>Combat uses a linear 6-zone system:</p>
          <pre>Side1 Ranged → Side1 Reach → Side1 Melee → Side2 Melee → Side2 Reach → Side2 Ranged</pre>
          <p>Each zone has capacity limits (configurable):</p>
          <ul>
            <li><strong>Ranged:</strong> Infinite (default)</li>
            <li><strong>Reach:</strong> 3 (default)</li>
            <li><strong>Melee:</strong> 3 (default)</li>
          </ul>
        </section>

        <section>
          <h3>Actor Attributes</h3>
          <table>
            <tbody>
              <tr><td><code>name</code></td><td>Actor name</td></tr>
              <tr><td><code>hp</code></td><td>Hit points (number or dice like "1d8+2")</td></tr>
              <tr><td><code>ac</code></td><td>Armor class (ascending)</td></tr>
              <tr><td><code>attack_bonus</code></td><td>Added to d20 attack roll</td></tr>
              <tr><td><code>damage</code></td><td>Damage dice (e.g., 1d8+3)</td></tr>
              <tr><td><code>speed</code></td><td>Zones moved per turn (default: 1)</td></tr>
              <tr><td><code>range</code></td><td>melee (adjacent), reach (2 zones), ranged (2+ zones)</td></tr>
              <tr><td><code>start_zone</code></td><td>ranged (default), reach, melee</td></tr>
              <tr><td><code>initiative_modifier</code></td><td>Bonus to initiative roll (default: 0)</td></tr>
            </tbody>
          </table>
        </section>

        <section>
          <h3>Initiative Systems</h3>
          <p>Configure under the <code>initiative:</code> block:</p>
          <h4>Types</h4>
          <table>
            <tbody>
              <tr><td><code>side</code></td><td>One side acts completely, then the other (default)</td></tr>
              <tr><td><code>individual</code></td><td>Each actor rolls initiative dice + modifier, acts in order</td></tr>
              <tr><td><code>side_phases</code></td><td>Phased combat by side: phases execute in order for each side</td></tr>
              <tr><td><code>individual_phases</code></td><td>Phased combat: phases execute in order, actors act by initiative within each phase</td></tr>
            </tbody>
          </table>
          <h4>Options</h4>
          <table>
            <tbody>
              <tr><td><code>type</code></td><td>Initiative type (see above)</td></tr>
              <tr><td><code>dice</code></td><td>Dice formula for rolls (default: 1d20)</td></tr>
              <tr><td><code>phases</code></td><td>Phase order for phase-based systems</td></tr>
            </tbody>
          </table>
          <h4>Example</h4>
          <pre>{`initiative:
  type: individual_phases
  dice: 1d6
  phases:
    - movement
    - ranged
    - melee`}</pre>
        </section>

        <section>
          <h3>Action Priority Lists (APL)</h3>
          <p>Each turn, an actor gets <strong>1 move</strong> and <strong>1 attack</strong>. The APL is scanned to find the first valid move and attack. Move executes first, then attack is re-evaluated (so you can move into range and attack).</p>

          <h4>Actions</h4>
          <table>
            <tbody>
              <tr><td><code>attack</code></td><td>Attack an enemy (must be in range)</td></tr>
              <tr><td><code>move</code></td><td>Move toward a target or direction</td></tr>
            </tbody>
          </table>

          <h4>Conditions (if)</h4>
          <table>
            <tbody>
              <tr><td><code>enemy.in_range</code></td><td>Any enemy within weapon range</td></tr>
              <tr><td><code>!enemy.in_range</code></td><td>No enemies in range</td></tr>
              <tr><td><code>self.health_percent &lt; N</code></td><td>HP% below N</td></tr>
              <tr><td><code>self.health_percent &gt; N</code></td><td>HP% above N</td></tr>
              <tr><td><code>self.hp &lt; N</code></td><td>Current HP below N</td></tr>
              <tr><td><code>enemy.count &lt; N</code></td><td>Fewer than N enemies alive</td></tr>
              <tr><td><code>ally.count &lt; N</code></td><td>Fewer than N allies alive</td></tr>
            </tbody>
          </table>

          <h4>Targets</h4>
          <table>
            <tbody>
              <tr><td><code>nearest_enemy</code></td><td>Closest enemy</td></tr>
              <tr><td><code>lowest_hp_enemy</code></td><td>Enemy with least HP</td></tr>
              <tr><td><code>random_enemy</code></td><td>Random enemy</td></tr>
              <tr><td><code>forward</code></td><td>Move toward enemy side</td></tr>
              <tr><td><code>backward</code></td><td>Move toward own ranged zone</td></tr>
            </tbody>
          </table>
        </section>

        <section>
          <h3>Example APL</h3>
          <pre>{`apl:
  - action: attack
    if: enemy.in_range
    target: lowest_hp_enemy
  - action: move
    target: nearest_enemy`}</pre>
        </section>

        <section>
          <h3>Zone Capacity Config</h3>
          <pre>{`zone_capacity:
  ranged: ~      # null = infinite
  reach: 3
  melee: 3`}</pre>
        </section>
      </div>
    </div>
  )
}

function App() {
  const [yaml, setYaml] = useState(DEFAULT_ENCOUNTER)
  const [result, setResult] = useState<SimulationResult | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedCombat, setSelectedCombat] = useState<number>(0)
  const [showDocs, setShowDocs] = useState(false)

  const runSimulation = async () => {
    setLoading(true)
    setError(null)
    try {
      const apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:3000';
      const response = await axios.post<SimulationResult>(`${apiUrl}/simulate`, {
        encounter_yaml: yaml,
        sample_count: 5
      })
      setResult(response.data)
      setSelectedCombat(0)
    } catch (err: unknown) {
      if (axios.isAxiosError(err)) {
        setError(err.response?.data?.error || err.message)
      } else {
        setError('An error occurred')
      }
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="app">
      <header>
        <h1>Tunnel Fight</h1>
        <p className="subtitle">OSR Combat Simulator</p>
        <button className="docs-button" onClick={() => setShowDocs(true)}>Docs</button>
      </header>

      {showDocs && <Docs onClose={() => setShowDocs(false)} />}

      <main>
        <section className="config-section">
          <h2>Encounter Configuration</h2>
          <textarea
            value={yaml}
            onChange={(e) => setYaml(e.target.value)}
            placeholder="Enter encounter YAML..."
            spellCheck={false}
          />
          <button onClick={runSimulation} disabled={loading}>
            {loading ? 'Simulating...' : 'Run Simulation'}
          </button>
          {error && <div className="error">{error}</div>}
        </section>

        {result && (
          <section className="results-section">
            <h2>Results ({result.stats.iterations.toLocaleString()} iterations)</h2>

            <div className="stats-grid">
              <div className="stat-card">
                <h3>Win Rates</h3>
                <div className="stat-row">
                  <span>Side 1:</span>
                  <span className="value">{result.stats.side1_win_rate.toFixed(1)}%</span>
                </div>
                <div className="stat-row">
                  <span>Side 2:</span>
                  <span className="value">{result.stats.side2_win_rate.toFixed(1)}%</span>
                </div>
                <div className="stat-row">
                  <span>Draw:</span>
                  <span className="value">{result.stats.draw_rate.toFixed(1)}%</span>
                </div>
              </div>

              <div className="stat-card">
                <h3>Combat Duration</h3>
                <div className="stat-row">
                  <span>Avg Rounds:</span>
                  <span className="value">{result.stats.avg_rounds.toFixed(1)}</span>
                </div>
              </div>

              <div className="stat-card">
                <h3>Casualties</h3>
                <div className="stat-row">
                  <span>Side 1 Avg:</span>
                  <span className="value">{result.stats.avg_side1_casualties.toFixed(2)}</span>
                </div>
                <div className="stat-row">
                  <span>Side 2 Avg:</span>
                  <span className="value">{result.stats.avg_side2_casualties.toFixed(2)}</span>
                </div>
              </div>

              <div className="stat-card">
                <h3>TPK Rate</h3>
                <div className="stat-row">
                  <span>Side 1:</span>
                  <span className="value danger">{result.stats.side1_tpk_rate.toFixed(1)}%</span>
                </div>
                <div className="stat-row">
                  <span>Side 2:</span>
                  <span className="value danger">{result.stats.side2_tpk_rate.toFixed(1)}%</span>
                </div>
              </div>

              <div className="stat-card">
                <h3>Flawless Victory</h3>
                <div className="stat-row">
                  <span>Side 1:</span>
                  <span className="value success">{result.stats.side1_flawless_rate.toFixed(1)}%</span>
                </div>
                <div className="stat-row">
                  <span>Side 2:</span>
                  <span className="value success">{result.stats.side2_flawless_rate.toFixed(1)}%</span>
                </div>
              </div>

              <div className="stat-card">
                <h3>HP Lost (Avg)</h3>
                <div className="stat-row">
                  <span>Side 1:</span>
                  <span className="value">{result.stats.avg_side1_hp_lost.toFixed(1)} ({result.stats.avg_side1_hp_lost_percent.toFixed(1)}%)</span>
                </div>
                <div className="stat-row">
                  <span>Side 2:</span>
                  <span className="value">{result.stats.avg_side2_hp_lost.toFixed(1)} ({result.stats.avg_side2_hp_lost_percent.toFixed(1)}%)</span>
                </div>
              </div>
            </div>

            <div className="combat-log-section">
              <h3>Sample Combats</h3>
              <div className="combat-tabs">
                {result.sample_combats.map((_, i) => (
                  <button
                    key={i}
                    className={selectedCombat === i ? 'active' : ''}
                    onClick={() => setSelectedCombat(i)}
                  >
                    Combat {i + 1}
                  </button>
                ))}
              </div>

              {result.sample_combats[selectedCombat] && (
                <div className="combat-log">
                  <div className="combat-header">
                    <span>Winner: {result.sample_combats[selectedCombat].winner || 'Draw'}</span>
                    <span>Rounds: {result.sample_combats[selectedCombat].rounds}</span>
                  </div>

                  <div className="final-state">
                    <h4>Final State</h4>
                    <div className="actors-grid">
                      {result.sample_combats[selectedCombat].final_state.map((actor, i) => (
                        <div key={i} className={`actor-state ${actor.alive ? 'alive' : 'dead'}`}>
                          <span className="actor-name">{actor.name}</span>
                          <span className="actor-hp">{actor.hp}</span>
                          <span className="actor-zone">{actor.zone}</span>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="events">
                    <h4>Combat Log</h4>
                    {result.sample_combats[selectedCombat].events.map((event, i) => (
                      <div key={i} className="event">
                        <span className="round">R{event.round}</span>
                        <span className="actor">{event.actor}</span>
                        <span className="description">{event.description}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </section>
        )}
      </main>
    </div>
  )
}

export default App
