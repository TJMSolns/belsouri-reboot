# Hierarchical Contextual Bandits for Per-Phase Configuration Optimization

**Version**: 1.0.0
**Last Updated**: 2026-01-04
**Audience**: Architects, ML Engineers, Search Engineers

---

## Purpose

This document establishes the architecture for **hierarchical contextual bandits** that optimize each phase of the three-phase search pipeline (preprocess → retrieve → rank) independently while preserving phase dependencies and enabling cross-tenant meta-learning for cold start.

**Key Insight**: Since the search pipeline decomposes into distinct functional phases, we can optimize each phase separately using contextual bandits, reducing the search space from exponential (N^3 complete configs) to linear (P + R + K components) while preserving causality through hierarchical decision-making.

---

## 1. Motivation: Why Per-Phase Optimization?

### 1.1 The Configuration Explosion Problem

**Monolithic Approach** (current thesis Section 5.2):
```scala
// Each config specifies ALL three phases
val configs = List(
  Config(intent=RuleBased, k=50,  fusion=0.4, boost=1.5),
  Config(intent=ML,        k=100, fusion=0.5, boost=2.0),
  Config(intent=RuleBased, k=100, fusion=0.3, boost=1.8)
)
// Bandit selects ONE complete pipeline
```

**Problem**: With 3 options per phase → 3 × 3 × 3 = **27 complete configurations** to explore.

**Hierarchical Approach** (this document):
```scala
// Separate bandits per phase
val phase1Configs = List(RuleBased, ML, LLM)           // 3 options
val phase2Configs = List(k50_alpha04, k100_alpha05, k100_alpha03) // 3 options
val phase3Configs = List(boost15, boost20, boost18)    // 3 options

// Total search space: 3 + 3 + 3 = 9 components
```

**Benefit**: Linear growth instead of exponential.

### 1.2 Multi-Tenant Cold Start Advantage

With per-phase optimization, we can aggregate learnings across tenants **per phase**:

| Tenant | Vertical | Phase 1 (Preprocess) | Phase 2 (Retrieve) | Phase 3 (Rank) |
|--------|----------|---------------------|-------------------|----------------|
| ElectronicExpress | Electronics | ML (0.7) | k=100, α=0.5 (0.8) | boost=2.0 (0.9) |
| ApplianceDirect | Home | RuleBased (0.6) | k=100, α=0.5 (0.7) | boost=1.5 (0.6) |
| OutdoorGearHub | Sporting | RuleBased (0.8) | k=150, α=0.4 (0.6) | boost=2.0 (0.7) |

**New Tenant** (FurnitureWorld, vertical=Home):
- Phase 1 prior: RuleBased (aggregate Home vertical → 0.6 weight)
- Phase 2 prior: k=100, α=0.5 (universal pattern → 0.75 weight)
- Phase 3 prior: boost=1.5 (Home vertical → 0.6 weight)

---

## 2. Architecture Overview

### 2.1 Hierarchical Decision Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    QUERY ARRIVES                             │
│                  (text, context)                             │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│  PHASE 1: PREPROCESSING BANDIT                              │
│                                                              │
│  Context: BaseContext(intent_hint, user_segment, device)    │
│  Decision: Select PreprocessConfig                          │
│  Output: EnrichedQuery(intent, concepts, embedding)         │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│  PHASE 2: RETRIEVAL BANDIT                                  │
│                                                              │
│  Context: BaseContext + IntentContext(enriched.intent)      │
│  Decision: Select RetrieveConfig (given Phase 1 output)     │
│  Output: CandidateSet(top-100 items)                        │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│  PHASE 3: RANKING BANDIT                                    │
│                                                              │
│  Context: BaseContext + IntentContext + CandidateContext    │
│  Decision: Select RankConfig (given Phase 1+2 output)       │
│  Output: RankedResults(sorted, diversified)                 │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│               USER INTERACTION                               │
│          (click, dwell, purchase) → REWARD                   │
│                                                              │
│    Hierarchical Attribution:                                │
│    - Phase 1 credit: f₁(reward, context)                    │
│    - Phase 2 credit: f₂(reward, context)                    │
│    - Phase 3 credit: f₃(reward, context)                    │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Core Abstraction

```scala
trait PhaseConfig {
  def execute(input: PhaseInput): PhaseOutput
}

trait ContextualBandit[C <: PhaseConfig] {
  def select(context: Context): C
  def update(config: C, context: Context, reward: Double): Unit
  def getWeights(context: Context): Map[C, Double]
}

case class HierarchicalSearchEngine(
  phase1Bandit: ContextualBandit[PreprocessConfig],
  phase2Bandit: ContextualBandit[RetrieveConfig],
  phase3Bandit: ContextualBandit[RankConfig],
  rewardAttribution: RewardAttributionStrategy
) {

  def search(query: RawQuery, context: BaseContext): RankedResults = {

    // Phase 1: Preprocessing
    val p1Config = phase1Bandit.select(context)
    val enriched = p1Config.preprocess(query)

    // Phase 2: Retrieval (context enriched with Phase 1 output)
    val context2 = context.withIntent(enriched.intent)
    val p2Config = phase2Bandit.select(context2)
    val candidates = p2Config.retrieve(enriched)

    // Phase 3: Ranking (context enriched with Phase 1+2 output)
    val context3 = context2.withCandidates(candidates.size, candidates.types)
    val p3Config = phase3Bandit.select(context3)
    val results = p3Config.rank(candidates, enriched)

    // Store for reward observation
    logDecision(query.id, p1Config, p2Config, p3Config, context3)

    results
  }

  def observeReward(queryId: QueryId, reward: Double): Unit = {
    val decision = getDecision(queryId)
    val attribution = rewardAttribution.attribute(
      reward,
      decision.p1Config,
      decision.p2Config,
      decision.p3Config,
      decision.context
    )

    phase1Bandit.update(decision.p1Config, decision.context, attribution.phase1Reward)
    phase2Bandit.update(decision.p2Config, decision.context2, attribution.phase2Reward)
    phase3Bandit.update(decision.p3Config, decision.context3, attribution.phase3Reward)
  }
}
```

---

## 3. Context Enrichment Strategy

### 3.1 Context Hierarchy

Each phase receives **progressively richer context** based on previous phases' outputs:

```scala
// Base context: always available
case class BaseContext(
  userSegment: UserSegment,      // NewUser, Returning, PowerUser
  deviceType: DeviceType,         // Mobile, Desktop, Tablet
  timeOfDay: TimeOfDay,           // Morning, Afternoon, Evening
  tenantId: TenantId              // For multi-tenant isolation
)

// Phase 1 context: base only
type Phase1Context = BaseContext

// Phase 2 context: base + intent inference from Phase 1
case class Phase2Context(
  base: BaseContext,
  inferredIntent: Intent,         // Learn, Shop, Troubleshoot, Compare, Explore
  conceptCount: Int,              // Number of concepts extracted
  hasEmbedding: Boolean          // Whether embedding was generated
) {
  def toFeatureVector: Vector[Double] = ???
}

// Phase 3 context: base + Phase 1 + retrieval stats from Phase 2
case class Phase3Context(
  base: BaseContext,
  inferredIntent: Intent,
  candidateCount: Int,            // How many candidates retrieved?
  candidateTypes: Map[ContentType, Int], // Distribution: products, guides, faqs
  avgRetrievalScore: Double       // Average BM25/k-NN fusion score
) {
  def toFeatureVector: Vector[Double] = ???
}
```

**Why This Matters**:
- Phase 2 can adapt retrieval strategy based on Phase 1's intent classification
- Phase 3 can adapt ranking based on what Phase 2 retrieved (e.g., if all products → no need for diversity boost)

### 3.2 Feature Engineering for Context

```scala
object ContextFeatures {

  def extractPhase1Features(ctx: BaseContext): Vector[Double] = {
    Vector(
      ctx.userSegment.ordinal.toDouble,
      ctx.deviceType match {
        case Mobile => 0.0
        case Desktop => 1.0
        case Tablet => 0.5
      },
      ctx.timeOfDay.hourOfDay.toDouble / 24.0
    )
  }

  def extractPhase2Features(ctx: Phase2Context): Vector[Double] = {
    extractPhase1Features(ctx.base) ++ Vector(
      ctx.inferredIntent.ordinal.toDouble / Intent.values.size,
      ctx.conceptCount.toDouble / 10.0,  // Normalize assuming max ~10 concepts
      if (ctx.hasEmbedding) 1.0 else 0.0
    )
  }

  def extractPhase3Features(ctx: Phase3Context): Vector[Double] = {
    extractPhase2Features(ctx.toPhase2Context) ++ Vector(
      ctx.candidateCount.toDouble / 100.0,  // Normalize assuming k=100 max
      ctx.candidateTypes.getOrElse(Product, 0).toDouble / ctx.candidateCount,
      ctx.candidateTypes.getOrElse(Guide, 0).toDouble / ctx.candidateCount,
      ctx.avgRetrievalScore
    )
  }
}
```

---

## 4. Thompson Sampling Implementation Per Phase

### 4.1 Contextual Thompson Sampling

```scala
class ContextualThompsonSampling[C <: PhaseConfig](
  configs: List[C],
  featureExtractor: Context => Vector[Double]
) extends ContextualBandit[C] {

  // Beta distributions per (config, context_bucket)
  // Context discretization: hash feature vector to bucket
  private val priors: mutable.Map[(C, ContextBucket), BetaDistribution] =
    mutable.Map.empty.withDefaultValue(BetaDistribution(alpha = 1.0, beta = 1.0))

  def select(context: Context): C = {
    val features = featureExtractor(context)
    val bucket = discretizeContext(features)

    // Sample from Beta distribution for each config
    val samples = configs.map { config =>
      val beta = priors((config, bucket))
      val sample = beta.sample()
      (config, sample)
    }

    // Select config with highest sampled value (Thompson Sampling)
    samples.maxBy(_._2)._1
  }

  def update(config: C, context: Context, reward: Double): Unit = {
    val features = featureExtractor(context)
    val bucket = discretizeContext(features)
    val key = (config, bucket)

    val current = priors(key)
    priors(key) = BetaDistribution(
      alpha = current.alpha + reward,
      beta = current.beta + (1.0 - reward)
    )
  }

  def getWeights(context: Context): Map[C, Double] = {
    val features = featureExtractor(context)
    val bucket = discretizeContext(features)

    configs.map { config =>
      val beta = priors((config, bucket))
      config -> beta.mean  // E[Beta(α,β)] = α/(α+β)
    }.toMap
  }

  private def discretizeContext(features: Vector[Double]): ContextBucket = {
    // Simple bucketing: quantize each feature to 0.0/0.5/1.0
    val quantized = features.map { f =>
      if (f < 0.33) 0.0
      else if (f < 0.67) 0.5
      else 1.0
    }
    ContextBucket(quantized.hashCode)
  }
}

case class ContextBucket(hash: Int) extends AnyVal
```

### 4.2 Instantiation for Each Phase

```scala
object BanditFactory {

  def createPhase1Bandit(
    preprocessConfigs: List[PreprocessConfig]
  ): ContextualBandit[PreprocessConfig] = {
    new ContextualThompsonSampling(
      preprocessConfigs,
      ctx => ContextFeatures.extractPhase1Features(ctx.asInstanceOf[BaseContext])
    )
  }

  def createPhase2Bandit(
    retrieveConfigs: List[RetrieveConfig]
  ): ContextualBandit[RetrieveConfig] = {
    new ContextualThompsonSampling(
      retrieveConfigs,
      ctx => ContextFeatures.extractPhase2Features(ctx.asInstanceOf[Phase2Context])
    )
  }

  def createPhase3Bandit(
    rankConfigs: List[RankConfig]
  ): ContextualBandit[RankConfig] = {
    new ContextualThompsonSampling(
      rankConfigs,
      ctx => ContextFeatures.extractPhase3Features(ctx.asInstanceOf[Phase3Context])
    )
  }
}
```

---

## 5. Reward Attribution Strategies

### 5.1 The Credit Assignment Problem

When a query results in reward r (e.g., 0.8 from click + dwell), **which phase deserves credit?**

**Scenario 1**: Intent misclassified (Phase 1 error)
```
Query: "best laptop for students"
Phase 1: Intent = "Compare" (WRONG, should be "Shop")
Phase 2: Retrieved comparison articles (following Phase 1's intent)
Phase 3: Ranked articles at top
User: Bounces (no click) → reward = 0.0

Who's at fault? Phase 1 (wrong intent), not Phase 2 or 3!
```

**Scenario 2**: Good intent, bad ranking (Phase 3 error)
```
Query: "best laptop for students"
Phase 1: Intent = "Shop" (CORRECT)
Phase 2: Retrieved 100 relevant laptops (good recall)
Phase 3: Ranked gaming laptops at top (WRONG, should be budget/education)
User: Scrolls to position 15, clicks → reward = 0.4

Who's at fault? Phase 3 (bad ranking), Phase 1+2 were fine!
```

### 5.2 Attribution Strategy 1: Equal Attribution (Simple)

```scala
object EqualAttribution extends RewardAttributionStrategy {

  def attribute(
    reward: Double,
    p1: PreprocessConfig,
    p2: RetrieveConfig,
    p3: RankConfig,
    context: Phase3Context
  ): Attribution = {
    Attribution(
      phase1Reward = reward / 3.0,
      phase2Reward = reward / 3.0,
      phase3Reward = reward / 3.0
    )
  }
}
```

**Pros**: Simple, no assumptions
**Cons**: Ignores which phase actually caused success/failure

### 5.3 Attribution Strategy 2: Learned Weights

```scala
class LearnedAttribution(
  // Neural network or linear model: (phase_outputs, reward) → (w1, w2, w3)
  model: AttributionModel
) extends RewardAttributionStrategy {

  def attribute(
    reward: Double,
    p1: PreprocessConfig,
    p2: RetrieveConfig,
    p3: RankConfig,
    context: Phase3Context
  ): Attribution = {

    val features = Vector(
      // Phase 1 quality indicators
      context.inferredIntent.confidence,  // How confident was intent classification?
      context.conceptCount.toDouble,      // Did we extract meaningful concepts?

      // Phase 2 quality indicators
      context.candidateCount.toDouble / 100.0,  // Recall (did we retrieve enough?)
      context.avgRetrievalScore,                // Relevance (how good were candidates?)

      // Phase 3 quality indicators
      context.candidateTypes.entropy,     // Diversity (did we avoid homogeneous results?)
      context.candidateTypes.getOrElse(context.inferredIntent.preferredType, 0).toDouble
    )

    val weights = model.predict(features, reward)  // → (w1, w2, w3) summing to 1.0

    Attribution(
      phase1Reward = weights(0) * reward,
      phase2Reward = weights(1) * reward,
      phase3Reward = weights(2) * reward
    )
  }
}
```

**Training**: Use historical (query, reward) pairs to learn which phase features correlate with success.

### 5.4 Attribution Strategy 3: Counterfactual Reasoning (Advanced)

```scala
object CounterfactualAttribution extends RewardAttributionStrategy {

  def attribute(
    reward: Double,
    p1: PreprocessConfig,
    p2: RetrieveConfig,
    p3: RankConfig,
    context: Phase3Context
  ): Attribution = {

    // Estimate: "What if we'd used a different Phase 1 config?"
    val alternativeP1Configs = getAllConfigs[PreprocessConfig].filter(_ != p1)
    val phase1Credit = alternativeP1Configs.map { altP1 =>
      estimateReward(altP1, p2, p3, context.base)
    }.mean

    // If actual reward >> avg alternative reward → Phase 1 deserves credit
    // If actual reward << avg alternative reward → Phase 1 deserves blame
    val phase1Attribution = (reward - phase1Credit) / reward

    // Repeat for Phase 2 and 3
    val phase2Attribution = ??? // Similar counterfactual for Phase 2
    val phase3Attribution = ??? // Similar counterfactual for Phase 3

    // Normalize to sum to 1.0
    val total = phase1Attribution + phase2Attribution + phase3Attribution
    Attribution(
      phase1Reward = (phase1Attribution / total) * reward,
      phase2Reward = (phase2Attribution / total) * reward,
      phase3Reward = (phase3Attribution / total) * reward
    )
  }

  private def estimateReward(
    p1: PreprocessConfig,
    p2: RetrieveConfig,
    p3: RankConfig,
    context: BaseContext
  ): Double = {
    // Use historical data: queries with similar context + config combo
    // Return avg reward observed
    ???
  }
}
```

**Pros**: Theoretically grounded (causal inference)
**Cons**: Requires significant historical data, computationally expensive

### 5.5 Recommended: Hybrid Attribution

```scala
object HybridAttribution extends RewardAttributionStrategy {

  def attribute(
    reward: Double,
    p1: PreprocessConfig,
    p2: RetrieveConfig,
    p3: RankConfig,
    context: Phase3Context
  ): Attribution = {

    // Default: equal split
    var w1 = 1.0 / 3.0
    var w2 = 1.0 / 3.0
    var w3 = 1.0 / 3.0

    // Heuristic adjustments based on observable signals

    // If intent confidence is low → penalize Phase 1
    if (context.inferredIntent.confidence < 0.5) {
      w1 *= 0.5  // Phase 1 gets less credit/blame
      w3 *= 1.5  // Phase 3 had to compensate
    }

    // If retrieval returned very few candidates → penalize Phase 2
    if (context.candidateCount < 20) {
      w2 *= 0.5
      w3 *= 1.5  // Ranking had limited material to work with
    }

    // If user clicked very late (position > 10) → penalize Phase 3
    if (reward > 0 && context.clickPosition.exists(_ > 10)) {
      w3 *= 0.5  // Ranking clearly suboptimal
      w2 *= 1.5  // But retrieval found it eventually
    }

    // Normalize
    val total = w1 + w2 + w3
    Attribution(
      phase1Reward = (w1 / total) * reward,
      phase2Reward = (w2 / total) * reward,
      phase3Reward = (w3 / total) * reward
    )
  }
}
```

---

## 6. Multi-Tenant Meta-Learning for Cold Start

### 6.1 Cross-Tenant Aggregation

```scala
object TenantMetaLearning {

  case class TenantProfile(
    tenantId: TenantId,
    vertical: Vertical,                    // Electronics, Appliances, Sporting, etc.
    scale: Scale,                          // Small (<1K products), Medium, Large
    phase1Weights: Map[PreprocessConfig, Double],
    phase2Weights: Map[RetrieveConfig, Double],
    phase3Weights: Map[RankConfig, Double]
  )

  def aggregateWeights[C](
    tenants: List[TenantProfile],
    phase: Phase,
    filter: TenantProfile => Boolean = _ => true
  ): Map[C, BetaDistribution] = {

    val relevantTenants = tenants.filter(filter)

    relevantTenants
      .flatMap { tenant =>
        val weights = phase match {
          case Phase1 => tenant.phase1Weights
          case Phase2 => tenant.phase2Weights
          case Phase3 => tenant.phase3Weights
        }

        weights.map { case (config, weight) =>
          // Convert weight to pseudo-observations
          // weight=0.8 after 100 queries → α=80, β=20
          val totalObs = 100  // Assume each tenant has ~100 queries
          val alpha = weight * totalObs
          val beta = (1 - weight) * totalObs
          (config.asInstanceOf[C], BetaDistribution(alpha, beta))
        }
      }
      .groupBy(_._1)
      .map { case (config, betas) =>
        // Aggregate Beta distributions by summing α and β
        val aggregated = betas.map(_._2).reduce { (b1, b2) =>
          BetaDistribution(b1.alpha + b2.alpha, b1.beta + b2.beta)
        }
        config -> aggregated
      }
  }

  def initializeNewTenant(
    newTenant: TenantId,
    vertical: Vertical,
    scale: Scale,
    existingTenants: List[TenantProfile]
  ): HierarchicalSearchEngine = {

    // Filter for similar tenants (same vertical, or all if < 3 similar)
    val similarTenants = existingTenants.filter(_.vertical == vertical)
    val tenants = if (similarTenants.size >= 3) similarTenants else existingTenants

    // Aggregate Phase 1 weights from similar tenants
    val phase1Priors = aggregateWeights[PreprocessConfig](
      tenants,
      Phase1,
      t => t.vertical == vertical  // Vertical-specific preprocessing
    )

    // Aggregate Phase 2 weights from ALL tenants (universal patterns)
    val phase2Priors = aggregateWeights[RetrieveConfig](
      existingTenants,  // Use all tenants, retrieval is more universal
      Phase2
    )

    // Aggregate Phase 3 weights from similar tenants
    val phase3Priors = aggregateWeights[RankConfig](
      tenants,
      Phase3,
      t => t.vertical == vertical || t.scale == scale
    )

    // Create bandits initialized with priors
    val phase1Bandit = new ContextualThompsonSampling(
      getAllConfigs[PreprocessConfig],
      ContextFeatures.extractPhase1Features,
      initialPriors = phase1Priors
    )

    val phase2Bandit = new ContextualThompsonSampling(
      getAllConfigs[RetrieveConfig],
      ContextFeatures.extractPhase2Features,
      initialPriors = phase2Priors
    )

    val phase3Bandit = new ContextualThompsonSampling(
      getAllConfigs[RankConfig],
      ContextFeatures.extractPhase3Features,
      initialPriors = phase3Priors
    )

    HierarchicalSearchEngine(phase1Bandit, phase2Bandit, phase3Bandit, HybridAttribution)
  }
}
```

### 6.2 Cold Start Example

**Scenario**: New tenant FurnitureWorld (vertical=Home, scale=Medium) onboards.

**Existing Tenants**:
1. ElectronicExpress (Electronics, Large): Phase1=ML(0.7), Phase2=k100α05(0.8), Phase3=boost20(0.9)
2. ApplianceDirect (Home, Medium): Phase1=RuleBased(0.6), Phase2=k100α05(0.7), Phase3=boost15(0.6)
3. OutdoorGearHub (Sporting, Small): Phase1=RuleBased(0.8), Phase2=k150α04(0.6), Phase3=boost20(0.7)

**Aggregation for FurnitureWorld**:

**Phase 1** (filter: vertical=Home):
- Only ApplianceDirect matches
- Initialize: RuleBased prior = Beta(60, 40) → mean = 0.6

**Phase 2** (no filter, universal pattern):
- All 3 tenants: k100α05 appears in EE (0.8) and AD (0.7)
- Aggregate: Beta(80+70, 20+30) = Beta(150, 50) → mean = 0.75
- Initialize: k100α05 with strong prior

**Phase 3** (filter: vertical=Home OR scale=Medium):
- ApplianceDirect matches both (Home + Medium): boost15 (0.6)
- Initialize: boost15 prior = Beta(60, 40)

**Result**: FurnitureWorld starts with informed priors, likely converges in 20-30 queries instead of 100+.

---

## 7. Trade-Offs and When to Use Hierarchical vs Monolithic

### 7.1 Comparison Matrix

| Aspect | Monolithic Config Selection | Hierarchical Per-Phase Bandits |
|--------|----------------------------|-------------------------------|
| **Search Space** | Exponential (P × R × K) | Linear (P + R + K) |
| **Convergence Speed** | Slower (more configs to explore) | Faster (fewer components) |
| **Phase Interactions** | Naturally captured (configs co-adapted) | Requires context enrichment |
| **Credit Assignment** | Not needed (single config gets reward) | Complex (reward attribution required) |
| **Multi-Tenant Cold Start** | Config-level priors | Phase-level priors (finer-grained) |
| **Debugging** | Easier (one config to blame) | Harder (which phase failed?) |
| **Computational Cost** | Lower (1 execution) | Similar (1 execution, 3 bandit updates) |
| **Mix-and-Match** | No (all-or-nothing) | Yes (best of each phase) |

### 7.2 Decision Guide

**Use Monolithic** when:
- ✅ Small config space (< 10 complete configs)
- ✅ Phases are tightly coupled (e.g., Phase 1 always determines Phase 3)
- ✅ Single-tenant system (no cross-tenant learning benefit)
- ✅ Simplicity > optimality

**Use Hierarchical** when:
- ✅ Large config space (> 20 complete configs)
- ✅ Multi-tenant platform (cold start is critical)
- ✅ Phases are relatively independent (preprocessing doesn't dictate ranking)
- ✅ You can invest in reward attribution logic
- ✅ You want to leverage cross-tenant learning per phase

### 7.3 Hybrid Approach (Recommended for Production)

```scala
object HybridApproach {

  // Start with monolithic for simplicity
  def initialDeployment(tenant: TenantId): SearchEngine = {
    MonolithicBanditEngine(
      configs = getTop10Configs(),  // Curated based on prior experience
      attribution = SimpleAttribution
    )
  }

  // After 1000 queries, switch to hierarchical for fine-tuning
  def graduateTo Hierarchical(
    tenant: TenantId,
    monolithicEngine: MonolithicBanditEngine
  ): HierarchicalSearchEngine = {

    // Extract learned preferences from monolithic phase
    val topConfigs = monolithicEngine.getTopConfigs(k = 3)

    // Decompose into phase components
    val phase1Candidates = topConfigs.map(_.preprocessConfig).distinct
    val phase2Candidates = topConfigs.map(_.retrieveConfig).distinct
    val phase3Candidates = topConfigs.map(_.rankConfig).distinct

    // Initialize hierarchical bandits with extracted components
    HierarchicalSearchEngine(
      phase1Bandit = createBanditFromConfigs(phase1Candidates),
      phase2Bandit = createBanditFromConfigs(phase2Candidates),
      phase3Bandit = createBanditFromConfigs(phase3Candidates),
      rewardAttribution = HybridAttribution
    )
  }
}
```

---

## 8. Implementation Guidance

### 8.1 Minimum Viable Implementation

**Phase 1**: Start with equal attribution, 3 configs per phase

```scala
// Configs
val preprocessConfigs = List(
  PreprocessConfig.RuleBased,
  PreprocessConfig.ML,
  PreprocessConfig.Hybrid
)

val retrieveConfigs = List(
  RetrieveConfig(k = 50, alpha = 0.4),
  RetrieveConfig(k = 100, alpha = 0.5),
  RetrieveConfig(k = 150, alpha = 0.6)
)

val rankConfigs = List(
  RankConfig(intentBoost = 1.5, maxPerType = 5),
  RankConfig(intentBoost = 2.0, maxPerType = 3),
  RankConfig(intentBoost = 1.8, maxPerType = 7)
)

// Initialize hierarchical engine
val engine = HierarchicalSearchEngine(
  phase1Bandit = BanditFactory.createPhase1Bandit(preprocessConfigs),
  phase2Bandit = BanditFactory.createPhase2Bandit(retrieveConfigs),
  phase3Bandit = BanditFactory.createPhase3Bandit(rankConfigs),
  rewardAttribution = EqualAttribution  // Start simple
)
```

**Deployment**:
1. Deploy alongside existing static config (shadow mode)
2. Log decisions and observe rewards for 1 week
3. Compare NDCG: hierarchical vs static
4. If hierarchical >= static, promote to production
5. After 1 month, upgrade to HybridAttribution

### 8.2 Monitoring and Observability

```yaml
metrics:
  - name: bandit_selection_count
    type: counter
    labels: [tenant_id, phase, config_id, context_bucket]

  - name: bandit_reward_observed
    type: histogram
    labels: [tenant_id, phase, config_id]
    buckets: [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]

  - name: config_weight
    type: gauge
    labels: [tenant_id, phase, config_id, context_bucket]
    description: "Current mean of Beta distribution (α/(α+β))"

  - name: attribution_split
    type: histogram
    labels: [tenant_id]
    description: "How reward was split: [phase1%, phase2%, phase3%]"

  - name: phase_convergence_time
    type: histogram
    labels: [tenant_id, phase]
    description: "Queries until weight variance < 0.1"
    buckets: [10, 25, 50, 100, 200, 500]
```

**Dashboards**:
- Per-tenant, per-phase config weights over time
- Attribution breakdown (stacked area chart: phase1 %, phase2 %, phase3 %)
- Convergence tracking (when did each phase stabilize?)
- Cross-tenant heatmap (which configs win for which verticals?)

### 8.3 Testing Strategy

**Unit Tests**:
```scala
class HierarchicalBanditSpec extends AnyFlatSpec with Matchers {

  "Phase 1 bandit" should "select different configs based on context" in {
    val bandit = BanditFactory.createPhase1Bandit(preprocessConfigs)

    val mobileContext = BaseContext(NewUser, Mobile, Morning, tenant1)
    val desktopContext = BaseContext(PowerUser, Desktop, Evening, tenant1)

    // After sufficient observations, contexts should diverge
    (1 to 100).foreach { _ =>
      val config = bandit.select(mobileContext)
      bandit.update(config, mobileContext, reward = 0.8)  // Mobile users like RuleBased
    }

    (1 to 100).foreach { _ =>
      val config = bandit.select(desktopContext)
      bandit.update(config, desktopContext, reward = 0.3)  // Desktop users dislike RuleBased
    }

    // Verify divergence
    val mobileWeights = bandit.getWeights(mobileContext)
    val desktopWeights = bandit.getWeights(desktopContext)

    mobileWeights(RuleBased) should be > desktopWeights(RuleBased)
  }

  "Reward attribution" should "assign credit based on phase performance" in {
    val attribution = HybridAttribution.attribute(
      reward = 0.8,
      p1 = PreprocessConfig.ML,
      p2 = RetrieveConfig(k = 100, alpha = 0.5),
      p3 = RankConfig(intentBoost = 2.0, maxPerType = 3),
      context = Phase3Context(
        base = BaseContext(Returning, Desktop, Afternoon, tenant1),
        inferredIntent = Intent.Shop.copy(confidence = 0.95),  // High confidence
        candidateCount = 95,
        candidateTypes = Map(Product -> 90, Guide -> 5),
        avgRetrievalScore = 0.82
      )
    )

    // High intent confidence → Phase 1 deserves credit
    attribution.phase1Reward should be > (attribution.phase2Reward)
  }
}
```

**Integration Tests**:
```scala
class HierarchicalEngineIntegrationSpec extends AnyFlatSpec {

  "Hierarchical engine" should "converge to optimal configs within 100 queries" in {
    val engine = createTestEngine()

    // Simulate 100 queries with synthetic feedback
    val queries = generateTestQueries(count = 100, vertical = Electronics)

    queries.foreach { query =>
      val results = engine.search(query, BaseContext(...))
      val reward = simulateUserFeedback(results, query.groundTruth)
      engine.observeReward(query.id, reward)
    }

    // Check: Phase 1 should have converged to ML (best for Electronics)
    val phase1Weights = engine.phase1Bandit.getWeights(BaseContext(...))
    phase1Weights(PreprocessConfig.ML) should be > 0.6

    // Check: Overall NDCG should improve
    val finalNDCG = evaluateNDCG(engine, testSet)
    val initialNDCG = evaluateNDCG(randomEngine, testSet)
    finalNDCG should be > initialNDCG
  }
}
```

**Property-Based Tests** (ScalaCheck):
```scala
class BanditProperties extends Properties("ContextualBandit") {

  property("weights sum to ~1.0") = forAll { (context: BaseContext) =>
    val bandit = BanditFactory.createPhase1Bandit(preprocessConfigs)
    val weights = bandit.getWeights(context).values.sum
    math.abs(weights - 1.0) < 0.01  // Allow small floating-point error
  }

  property("more reward → higher weight") = forAll(
    Gen.choose(0.0, 1.0),
    Gen.choose(0.0, 1.0)
  ) { (reward1, reward2) =>
    val bandit = BanditFactory.createPhase1Bandit(List(ConfigA, ConfigB))
    val context = BaseContext(Returning, Desktop, Afternoon, tenant1)

    // Update ConfigA with reward1, ConfigB with reward2
    (1 to 50).foreach { _ =>
      bandit.update(ConfigA, context, reward1)
      bandit.update(ConfigB, context, reward2)
    }

    val weights = bandit.getWeights(context)

    if (reward1 > reward2 + 0.1) weights(ConfigA) > weights(ConfigB)
    else if (reward2 > reward1 + 0.1) weights(ConfigB) > weights(ConfigA)
    else true  // Similar rewards → similar weights
  }
}
```

---

## 9. Migration Path from Monolithic to Hierarchical

### 9.1 Three-Phase Rollout

**Phase 1**: Shadow Mode (Weeks 1-2)
```scala
// Run both engines in parallel, compare
val monolithicResults = monolithicEngine.search(query, context)
val hierarchicalResults = hierarchicalEngine.search(query, context)

// Serve monolithic to user
sendToUser(monolithicResults)

// Log both for comparison
logResults("monolithic", monolithicResults, reward)
logResults("hierarchical", hierarchicalResults, estimatedReward)

// Weekly: compare NDCG, latency, convergence
```

**Phase 2**: A/B Test (Weeks 3-4)
```scala
// 10% traffic to hierarchical, 90% to monolithic
val engine = if (Random.nextDouble() < 0.1) hierarchicalEngine else monolithicEngine
val results = engine.search(query, context)
sendToUser(results)

// Monitor: conversion rate, bounce rate, NDCG
```

**Phase 3**: Full Rollout (Week 5+)
```scala
// 100% traffic to hierarchical
val results = hierarchicalEngine.search(query, context)
sendToUser(results)

// Keep monolithic as fallback for 30 days
```

### 9.2 Backward Compatibility

```scala
trait SearchEngine {
  def search(query: RawQuery, context: BaseContext): RankedResults
  def observeReward(queryId: QueryId, reward: Double): Unit
}

// Both engines implement same interface
class MonolithicBanditEngine extends SearchEngine { ??? }
class HierarchicalSearchEngine extends SearchEngine { ??? }

// Client code unchanged
val engine: SearchEngine = getEngine(tenant)
val results = engine.search(query, context)
```

---

## 10. Advanced Topics

### 10.1 Contextual Bandit Variants

**Linear UCB** (alternative to Thompson Sampling):
```scala
class LinearUCB[C <: PhaseConfig](
  configs: List[C],
  featureExtractor: Context => Vector[Double],
  alpha: Double = 1.0  // Exploration parameter
) extends ContextualBandit[C] {

  private val models: mutable.Map[C, LinearRegressionModel] = ???

  def select(context: Context): C = {
    val features = featureExtractor(context)

    configs.map { config =>
      val model = models(config)
      val prediction = model.predict(features)
      val uncertainty = model.uncertainty(features)
      val ucb = prediction + alpha * uncertainty
      (config, ucb)
    }.maxBy(_._2)._1
  }

  // ... update using ridge regression
}
```

**When to use**:
- Linear UCB: Provable regret bounds, good for risk-averse deployments
- Thompson Sampling: Often faster convergence, better empirical performance

### 10.2 Bandit Ensembles (Meta-Bandit)

```scala
// Meta-bandit selects between monolithic and hierarchical!
val metaBandit = ThompsonSampling(
  arms = List("monolithic", "hierarchical")
)

val engineType = metaBandit.select()
val engine = engineType match {
  case "monolithic" => monolithicEngine
  case "hierarchical" => hierarchicalEngine
}

val results = engine.search(query, context)
// Observe reward, update meta-bandit
```

### 10.3 Multi-Armed Bandit for Reward Attribution

```scala
// Treat attribution strategies as arms!
val attributionBandit = ThompsonSampling(
  arms = List(EqualAttribution, HybridAttribution, LearnedAttribution)
)

val strategy = attributionBandit.select()
val attribution = strategy.attribute(reward, p1, p2, p3, context)

// Meta-reward: did this attribution lead to faster convergence?
// Update attribution bandit based on next-query NDCG improvement
```

---

## 11. Related SBPF Documents

- **[Multi-Tenant-Search-Platform-Architecture.md](Multi-Tenant-Search-Platform-Architecture.md)**: Multi-tenant architecture, cross-tenant learning
- **[Universal-Commerce-Index-Schema.md](Universal-Commerce-Index-Schema.md)**: Index schema for commerce domain
- **[Web-Crawling-Best-Practices.md](Web-Crawling-Best-Practices.md)**: Crawling patterns for corpus building
- **[Automated-Search-Quality-Testing.md](Automated-Search-Quality-Testing.md)**: Quality tests for cold start

---

## 12. References

- **Agarwal et al. (2014)**: "Taming the Monster: A Fast and Simple Algorithm for Contextual Bandits" (LinUCB)
- **Russo et al. (2018)**: "A Tutorial on Thompson Sampling" (Foundations and Trends in ML)
- **Li et al. (2010)**: "A Contextual-Bandit Approach to Personalized News Article Recommendation"
- **Chapelle & Li (2011)**: "An Empirical Evaluation of Thompson Sampling" (NIPS)

---

## 13. Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-01-04 | Author | Initial SBPF for hierarchical bandit architecture |

---

**Status**: ✅ **Active** - This SBPF establishes the hierarchical contextual bandit architecture for per-phase optimization with multi-tenant meta-learning.

**Summary**: Hierarchical bandits optimize each search phase independently (preprocess, retrieve, rank) using contextual Thompson Sampling with progressive context enrichment. Reward attribution strategies (equal, learned, hybrid) assign credit to individual phases. Multi-tenant meta-learning aggregates phase weights across tenants for cold start initialization, reducing convergence time from 100+ to 20-30 queries. Recommended for multi-tenant platforms with large configuration spaces where fine-grained optimization and cross-tenant learning provide significant value.
