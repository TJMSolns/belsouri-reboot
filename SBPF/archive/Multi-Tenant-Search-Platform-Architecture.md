# Multi-Tenant Search Platform Architecture

## SBPF Classification
- **Category**: Architecture & Design Patterns
- **Domain**: Multi-Tenant SaaS, Commerce Search
- **Audience**: Architects, Platform Engineers, Backend Developers
- **Status**: Active
- **Last Updated**: 2026-01-04

---

## Executive Summary

This document establishes architectural patterns for building a **multi-tenant semantic commerce search platform** where multiple operators (e-commerce sites) share infrastructure while maintaining strict data isolation. The architecture supports:

- **Tenant Isolation**: Complete data separation with compile-time guarantees
- **Event-Driven Indexing**: Operators publish catalog changes that trigger appropriate crawl/index jobs
- **Cross-Tenant Learning**: Privacy-preserving pattern discovery benefits all tenants without data sharing
- **Domain Lexicon Extraction**: Site-specific vocabulary learned from crawled content
- **Horizontal Scalability**: Add tenants without architectural changes

**Key Innovation**: Unlike traditional multi-tenant systems that only isolate data, this architecture enables **cross-tenant pattern learning** (synonym discovery, query normalization rules) while maintaining strict privacy boundaries.

---

## 1. Multi-Tenancy Model

### 1.1 Tenant Definition

A **tenant** represents a single e-commerce operator with:
- Independent product catalog
- Unique site structure and navigation
- Operator-specific vocabulary and brand taxonomy
- Isolated search configuration and feedback

**Example Tenants**:
- `electronic-express` → ElectronicExpress.com
- `best-buy` → BestBuy.com
- `newegg` → Newegg.com

### 1.2 Tenant Identity

```scala
// Tenant ID is the fundamental isolation boundary
case class TenantId(value: String) extends AnyVal {
  require(value.matches("^[a-z0-9-]+$"), "Tenant ID must be lowercase alphanumeric + hyphens")
}

// All domain entities carry tenant context
trait TenantScoped {
  def tenantId: TenantId
}

case class Product(
  id: ProductId,
  tenantId: TenantId,  // Compile-time guarantee of tenant context
  title: String,
  // ...
) extends TenantScoped
```

**Design Principle**: Tenant ID is a required field on all domain entities, enforced at compile time via `TenantScoped` trait.

---

## 2. Tenant Onboarding

### 2.1 Onboarding Configuration

When a new operator joins the platform, they provide:

```scala
case class TenantOnboardingConfig(
  tenantId: TenantId,                   // Unique identifier (e.g., "electronic-express")
  siteRoot: URL,                        // Starting URL for crawl (e.g., "https://www.electronicexpress.com/")
  language: Language,                   // Primary language (English, Spanish, etc.)
  triggerEvents: Option[EventIntegration],  // Optional event-driven triggers
  indexSchedule: IndexSchedule,         // Crawl/index frequency
  contactEmail: Email,                  // Alerts, drift notifications
  enabledFeatures: Set[Feature]         // Beta features, premium capabilities
)
```

**Example Configuration (ElectronicExpress)**:

```yaml
tenant_id: electronic-express
site_root: https://www.electronicexpress.com/
language: en
trigger_events:
  full_publish_event:
    type: kafka
    topic: ee-catalog-full-publish
    bootstrap_servers: kafka.electronicexpress.com:9092
  partial_publish_event:
    type: kafka
    topic: ee-catalog-partial-publish
    bootstrap_servers: kafka.electronicexpress.com:9092
index_schedule:
  default_crawl: 7d          # Weekly fallback if no events
  on_partial_event: 1h       # Batch incremental updates
  on_full_event: immediate   # High-priority full re-index
contact_email: search-ops@electronicexpress.com
enabled_features:
  - adaptive_ensemble
  - automated_quality_tests
  - cross_tenant_learning
```

### 2.2 Event Integration Options

Operators choose how to notify the platform of catalog changes:

```scala
sealed trait EventSource

// REST webhook: Platform provides endpoint, operator POSTs to it
case class WebhookSource(
  url: URL,           // Platform-provided webhook endpoint
  secret: String      // HMAC signing secret for verification
) extends EventSource

// Kafka topic: Operator grants platform consumer access
case class KafkaTopicSource(
  topic: String,
  bootstrapServers: String,
  consumerGroup: String,
  authConfig: KafkaAuth
) extends EventSource

// AWS SQS: Platform polls operator's queue
case class SQSQueueSource(
  queueUrl: String,
  region: AWSRegion,
  credentials: AWSCredentials
) extends EventSource
```

**Event Types**:

| Event Type | Trigger | Crawl Strategy | Use Case |
|------------|---------|----------------|----------|
| **FullPublishEvent** | Complete catalog refresh | Full re-crawl + re-index | Site redesign, migration, annual catalog reset |
| **PartialPublishEvent** | Incremental updates (new products, category changes) | Incremental crawl (delta only) | Daily/hourly product additions, price updates |

**Event Payload Example**:

```json
{
  "event_type": "partial_publish",
  "tenant_id": "electronic-express",
  "timestamp": "2026-01-04T10:30:00Z",
  "changed_urls": [
    "https://www.electronicexpress.com/laptops/dell-xps-15",
    "https://www.electronicexpress.com/categories/gaming-laptops"
  ],
  "metadata": {
    "change_reason": "new_product_launch",
    "affected_categories": ["laptops", "gaming"]
  }
}
```

---

## 3. Event-Driven Crawl Orchestration

### 3.1 Crawl Scheduling Logic

```scala
object CrawlOrchestrator {

  def determineCrawlStrategy(
    tenant: TenantId,
    event: Option[OperatorEvent],
    lastCrawl: Instant
  ): CrawlStrategy = {

    event match {
      case Some(FullPublishEvent(_, timestamp)) =>
        FullCrawlStrategy(
          priority = High,
          trigger = Immediate,
          scope = EntireSite(tenant)
        )

      case Some(PartialPublishEvent(_, changedUrls, timestamp)) =>
        IncrementalCrawlStrategy(
          priority = Medium,
          trigger = HourlyBatch,  // Collect partial events for 1 hour, then execute
          scope = SpecificUrls(tenant, changedUrls)
        )

      case None =>
        val config = TenantRegistry.getConfig(tenant)
        val timeSinceLastCrawl = Duration.between(lastCrawl, Instant.now)

        if (timeSinceLastCrawl > config.indexSchedule.defaultCrawl) {
          FullCrawlStrategy(
            priority = Low,
            trigger = Scheduled(config.indexSchedule.defaultCrawl),
            scope = EntireSite(tenant)
          )
        } else {
          NoCrawlNeeded
        }
    }
  }
}
```

### 3.2 Crawl Execution Pipeline

```
┌──────────────────────────────────────────────────────────────────┐
│                   EVENT-DRIVEN CRAWL PIPELINE                      │
└──────────────────────────────────────────────────────────────────┘

   OPERATOR PUBLISHES EVENT
           │
           ▼
   ┌─────────────────┐
   │  Event Listener │  (Webhook, Kafka, SQS)
   │  (Per Tenant)   │
   └────────┬────────┘
            │
            ▼
   ┌─────────────────┐
   │ Event Validator │  (Verify HMAC, check tenant exists)
   └────────┬────────┘
            │
            ▼
   ┌─────────────────────────┐
   │  Crawl Strategy Selector│  (Immediate, Hourly Batch, Weekly)
   └────────┬────────────────┘
            │
            ├────────────────────────────┐
            │                            │
   ┌────────▼────────┐        ┌─────────▼────────┐
   │  Full Crawl Job │        │ Incremental Job  │
   │  (High Priority)│        │ (Batched Hourly) │
   └────────┬────────┘        └─────────┬────────┘
            │                            │
            └────────────┬───────────────┘
                         │
                         ▼
            ┌──────────────────────┐
            │  Crawler Executor    │  (Tenant-isolated crawl)
            │  (Respects robots.txt,│
            │   Rate limits)       │
            └──────────┬───────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │  Content Extractor   │  (HTML → Structured data)
            └──────────┬───────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │  Domain Lexicon      │  (Extract site vocabulary)
            │  Builder             │
            └──────────┬───────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │  Enrichment Pipeline │  (Embeddings, concepts)
            └──────────┬───────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │  OpenSearch Indexer  │  (Tenant-scoped index)
            │  (Bulk API)          │
            └──────────┬───────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │  Automated Test      │  (Brand/category coherence)
            │  Generator           │
            └──────────┬───────────┘
                       │
                       ▼
                  INDEX READY
```

---

## 4. Data Isolation Architecture

### 4.1 Universal Schema Principle

**Critical Architectural Decision: Schema is Universal, Data is Isolated**

Unlike traditional multi-tenant systems where each tenant might have different schemas, this platform uses a **universal commerce index schema** shared across all tenants.

**Rationale**:

Commerce is a **bounded domain**—all e-commerce sites share the same core entities:
- Products (title, description, price, brand, SKU)
- Categories/Taxonomies
- Brands
- Content (guides, FAQs, comparisons)

This makes a unified schema not only possible but architecturally superior:

| Benefit | Impact |
|---------|--------|
| **Horizontal Scalability** | One index template to optimize; shard by `tenant_id` routing key |
| **Cross-Tenant Learning** | Analyze field importance globally; new tenants benefit from day 1 |
| **Operational Simplicity** | Consistent query patterns, unified monitoring, easier feature rollout |
| **Resource Efficiency** | Shared shard allocation, consistent merge policies, one ILM ruleset |

**What's Universal vs Tenant-Specific**:

```
UNIVERSAL (Shared Schema):
✅ Index mappings (field types, analyzers, k-NN settings)
✅ Field names (tenant_id, product_id, title, brand, category)
✅ Embedding dimension (768-dim MPNet)
✅ Nested attributes structure (key-value pairs)

TENANT-SPECIFIC (Isolated Data):
✅ Product data (filtered by tenant_id)
✅ Lexicon (canonical forms, synonyms)
✅ Search configurations (weights, strategies)
✅ Automated quality tests
```

**Full Schema Reference**: See [Universal-Commerce-Index-Schema.md](Universal-Commerce-Index-Schema.md) for complete schema definition, query patterns, and lifecycle management.

### 4.2 Isolation Boundaries

**Compile-Time Guarantees**:

```scala
// All queries MUST include tenant filter
trait TenantIsolatedQuery {
  def tenantId: TenantId

  def toOpenSearchQuery: OpenSearchQuery = {
    OpenSearchQuery(
      bool = BoolQuery(
        filter = List(
          TermQuery("tenant_id", tenantId.value)  // Always present
        ),
        // ... other clauses
      )
    )
  }
}

// Example: Search query
case class SearchQuery(
  tenantId: TenantId,
  queryText: String,
  filters: Map[String, String]
) extends TenantIsolatedQuery
```

**Runtime Enforcement**:

```scala
// Index-level isolation (OpenSearch)
// Each tenant gets logical separation via filtering, not separate indexes

object IndexNamingConvention {
  // Shared index pattern for operational efficiency
  def getIndexName(contentType: ContentType): String = contentType match {
    case ProductContent => "commerce-products"
    case ArticleContent => "commerce-articles"
    case GuideContent   => "commerce-guides"
  }

  // Tenant ID is a FIELD, not part of index name
  // Advantages:
  //   + Shared resource pool (shards, replicas)
  //   + Cross-tenant pattern analysis
  //   + Simpler operations (one index to manage per content type)
  // Tradeoffs:
  //   - Requires strict query filtering
  //   - Cannot physically isolate storage
}
```

### 4.3 Isolation Matrix

| Resource | Isolation Method | Implementation |
|----------|------------------|----------------|
| **Product Data** | Tenant ID filter on all queries | `filter: { "term": { "tenant_id": "ee" } }` |
| **Site Structure** | Tenant-scoped crawler state | `CrawlState(tenantId, visitedUrls, ...)` |
| **Automated Tests** | Scoped to tenant | `AutomatedQualityTest(tenantId, ...)` |
| **Config Weights** | Separate bandit state per tenant | `Map[TenantId, BanditState]` |
| **User Feedback** | Tenant-scoped event streams | `kafka://feedback.{tenantId}` |
| **Domain Lexicon** | Tenant-specific overlay | `TenantLexicon(tenantId, ...)` |
| **Search Configs** | Isolated A/B test groups | `ConfigWeights(tenantId, ...)` |

### 4.4 Never Shared Resources

```
❌ NEVER SHARED ACROSS TENANTS:
   - Product SKUs, prices, inventory
   - User behavior data (clicks, conversions, dwell time)
   - Proprietary categories/taxonomies
   - Brand-specific vocabulary (e.g., "Samsung QLED" only for tenants selling Samsung)
   - Competitive intelligence (search volume, trending queries)
   - Configuration weights (bandit states)
```

---

## 5. Cross-Tenant Learning (Privacy-Preserving)

### 5.1 What Gets Shared

While tenant data remains isolated, **aggregated patterns** can be promoted to a global shared lexicon:

```
✅ SHARED ACROSS TENANTS (AGGREGATED PATTERNS):
   - Base language dictionaries (English, Spanish)
   - Technical term registry ("QLED", "4K", "OLED", "USB-C")
   - Common synonym patterns (if seen in 3+ tenants with high confidence)
   - Query normalization rules (stemming, pluralization)
   - Shared embedding model (MPNet)
   - Common stop words, noise patterns
```

### 5.2 Pattern Promotion Rules

**Criteria for Promotion to Global Lexicon**:

```scala
object CrossTenantLearning {

  case class DiscoveredPattern(
    pattern: SynonymPair,          // e.g., ("TV", "television")
    tenantCount: Int,              // How many tenants observed this
    confidence: Double,            // Average confidence across tenants
    isProprietary: Boolean,        // Contains brand names, SKUs
    language: Language
  )

  def shouldPromoteToGlobal(pattern: DiscoveredPattern): Boolean = {
    pattern.tenantCount >= 3 &&           // Seen in at least 3 tenants
    pattern.confidence > 0.85 &&          // High confidence
    !pattern.isProprietary &&             // Not tenant-specific IP
    !isCompetitiveIntel(pattern)          // Not revealing competitive data
  }

  private def isCompetitiveIntel(pattern: DiscoveredPattern): Boolean = {
    // Reject patterns that reveal tenant-specific trends
    // e.g., "hot deal" → specific product SKU
    val rejectPatterns = Set(
      "SKU-", "UPC-", "model-",           // Product identifiers
      "sale", "clearance", "deal"         // Pricing intelligence
    )

    rejectPatterns.exists(reject =>
      pattern.pattern.term1.toLowerCase.contains(reject) ||
      pattern.pattern.term2.toLowerCase.contains(reject)
    )
  }
}
```

**Example: Synonym Discovery**:

```
Tenant A (ElectronicExpress):
  - Crawls 500 TVs, sees "TV" in 480 titles, "television" in 20 titles
  - Infers: "TV" canonical, "television" synonym (confidence: 0.92)

Tenant B (BestBuy):
  - Crawls 800 TVs, sees "television" in 600 titles, "TV" in 200 titles
  - Infers: "television" canonical, "TV" synonym (confidence: 0.88)

Tenant C (Newegg):
  - Crawls 300 TVs, sees "TV" in 270 titles, "television" in 30 titles
  - Infers: "TV" canonical, "television" synonym (confidence: 0.90)

→ Pattern Promotion:
  - Seen in 3+ tenants ✅
  - Average confidence: 0.90 > 0.85 ✅
  - Not proprietary (generic terms) ✅
  - PROMOTED to global: SynonymPattern({"TV", "television"}, language=en, confidence=0.90)

→ New Tenant D benefits:
  - Even if Tenant D uses "flatscreen" as canonical, global pattern ensures "TV" and "television" queries work
```

### 5.3 Lexicon Resolution Hierarchy

When processing a query, the system resolves terms in this order:

```
┌──────────────────────────────────────────────────────────────┐
│              DOMAIN LEXICON RESOLUTION ORDER                  │
└──────────────────────────────────────────────────────────────┘

   USER QUERY: "TV stand"
         │
         ▼
   ┌─────────────────────────┐
   │ 1. Tenant-Specific      │  (Highest Priority)
   │    Lexicon              │
   │                         │
   │  ElectronicExpress:     │
   │    "TV" → canonical     │
   │    "flatscreen" → TV    │
   └────────┬────────────────┘
            │
            │ NOT FOUND
            ▼
   ┌─────────────────────────┐
   │ 2. Shared Global        │  (Medium Priority)
   │    Lexicon              │
   │                         │
   │  All English Tenants:   │
   │    "TV" ↔ "television"  │
   │    "4K" ↔ "Ultra HD"    │
   └────────┬────────────────┘
            │
            │ FOUND: "TV" ↔ "television"
            ▼
   ┌─────────────────────────┐
   │ 3. Base Dictionary      │  (Fallback)
   │                         │
   │  English Dictionary:    │
   │    "stand" → noun       │
   └─────────────────────────┘

RESULT: Query expanded to: ["TV stand", "television stand"]
```

### 5.4 Meta-Learning for Per-Phase Configuration Cold Start

**Context**: Beyond lexicon sharing, cross-tenant learning enables **configuration meta-learning**—aggregating which search pipeline configurations work well across tenants to initialize new tenants with informed priors.

**Hierarchical Bandit Architecture Integration**

With hierarchical per-phase bandits (see [Hierarchical-Bandit-Architecture.md](Hierarchical-Bandit-Architecture.md)), each phase (preprocessing, retrieval, ranking) learns optimal configurations independently. This enables **per-phase cross-tenant aggregation**:

```scala
object TenantMetaLearning {

  case class TenantProfile(
    tenantId: TenantId,
    vertical: Vertical,                    // Electronics, Appliances, Sporting, Home, etc.
    scale: Scale,                          // Small (<1K products), Medium, Large
    queryVolume: QueryVolume,              // Low, Medium, High
    phase1Weights: Map[PreprocessConfig, BetaDistribution],
    phase2Weights: Map[RetrieveConfig, BetaDistribution],
    phase3Weights: Map[RankConfig, BetaDistribution],
    observedQueries: Int,
    lastUpdated: Instant
  )

  sealed trait Vertical
  case object Electronics extends Vertical
  case object Appliances extends Vertical
  case object Sporting extends Vertical
  case object Home extends Vertical
  case object Fashion extends Vertical
  case object B2B extends Vertical

  sealed trait Scale
  case object Small extends Scale    // < 1K products
  case object Medium extends Scale   // 1K - 10K products
  case object Large extends Scale    // > 10K products
}
```

**Cross-Tenant Aggregation Strategy**

```scala
object MetaLearning {

  def aggregatePhaseWeights[C <: PhaseConfig](
    tenants: List[TenantProfile],
    phase: Phase,
    filterFn: TenantProfile => Boolean = _ => true
  ): Map[C, BetaDistribution] = {

    val relevantTenants = tenants.filter(filterFn)

    if (relevantTenants.isEmpty) {
      // No relevant tenants → return uniform prior
      return getAllConfigs[C].map(c => c -> BetaDistribution(1.0, 1.0)).toMap
    }

    relevantTenants
      .flatMap { tenant =>
        val weights = phase match {
          case Phase1 => tenant.phase1Weights
          case Phase2 => tenant.phase2Weights
          case Phase3 => tenant.phase3Weights
        }

        weights.map { case (config, beta) =>
          (config.asInstanceOf[C], beta)
        }
      }
      .groupBy(_._1)
      .map { case (config, betas) =>
        // Aggregate Beta distributions by summing α and β
        // This is mathematically sound for combining observations
        val aggregated = betas.map(_._2).reduce { (b1, b2) =>
          BetaDistribution(
            alpha = b1.alpha + b2.alpha,
            beta = b1.beta + b2.beta
          )
        }
        config -> aggregated
      }
  }

  def initializeNewTenant(
    newTenant: TenantId,
    vertical: Vertical,
    scale: Scale,
    existingProfiles: List[TenantProfile]
  ): TenantProfile = {

    // Filter for similar tenants (prioritize same vertical + scale)
    val sameVerticalAndScale = existingProfiles.filter(p =>
      p.vertical == vertical && p.scale == scale
    )

    val sameVertical = existingProfiles.filter(_.vertical == vertical)
    val sameScale = existingProfiles.filter(_.scale == scale)

    // Phase 1: Preprocessing (vertical-specific patterns)
    // Electronics tenants often prefer ML intent classifier
    // Sporting/Fashion tenants often prefer rule-based
    val phase1Priors = if (sameVertical.size >= 2) {
      aggregatePhaseWeights[PreprocessConfig](
        sameVertical,
        Phase1,
        t => t.observedQueries > 50  // Only mature tenants
      )
    } else {
      aggregatePhaseWeights[PreprocessConfig](existingProfiles, Phase1)
    }

    // Phase 2: Retrieval (mostly universal patterns)
    // k=100, α=0.5 is common across all verticals
    val phase2Priors = aggregatePhaseWeights[RetrieveConfig](
      existingProfiles,  // Use ALL tenants (retrieval is universal)
      Phase2,
      t => t.observedQueries > 50
    )

    // Phase 3: Ranking (vertical + scale specific)
    // B2B prefers lower boost, higher diversity
    // Consumer electronics prefers higher boost
    val phase3Priors = if (sameVerticalAndScale.size >= 2) {
      aggregatePhaseWeights[RankConfig](sameVerticalAndScale, Phase3)
    } else if (sameVertical.size >= 2) {
      aggregatePhaseWeights[RankConfig](sameVertical, Phase3)
    } else {
      aggregatePhaseWeights[RankConfig](existingProfiles, Phase3)
    }

    TenantProfile(
      tenantId = newTenant,
      vertical = vertical,
      scale = scale,
      queryVolume = QueryVolume.Unknown,
      phase1Weights = phase1Priors,
      phase2Weights = phase2Priors,
      phase3Weights = phase3Priors,
      observedQueries = 0,
      lastUpdated = Instant.now()
    )
  }
}
```

**Cold Start Example**

| Existing Tenant | Vertical | Phase 1 Winner | Phase 2 Winner | Phase 3 Winner |
|----------------|----------|----------------|----------------|----------------|
| ElectronicExpress | Electronics | ML (α=70, β=30) | k=100, α=0.5 (α=80, β=20) | boost=2.0 (α=90, β=10) |
| ApplianceDirect | Home | RuleBased (α=60, β=40) | k=100, α=0.5 (α=70, β=30) | boost=1.5 (α=60, β=40) |
| OutdoorGearHub | Sporting | RuleBased (α=80, β=20) | k=150, α=0.4 (α=60, β=40) | boost=2.0 (α=70, β=30) |

**New Tenant**: FurnitureWorld (vertical=Home, scale=Medium)

**Initialization**:

**Phase 1** (filter: vertical=Home):
- Only ApplianceDirect matches
- Prior: RuleBased = Beta(60, 40) → mean = 0.60
- ML = Beta(1, 1) → mean = 0.50 (uniform fallback)

**Phase 2** (no filter, use all tenants):
- ElectronicExpress: k=100, α=0.5 → Beta(80, 20)
- ApplianceDirect: k=100, α=0.5 → Beta(70, 30)
- Aggregate: Beta(80+70, 20+30) = Beta(150, 50) → **mean = 0.75** (strong prior!)

**Phase 3** (filter: vertical=Home OR scale=Medium):
- ApplianceDirect matches (Home + Medium)
- Prior: boost=1.5 = Beta(60, 40) → mean = 0.60

**Empirical Impact**:

| Metric | Random Init | Meta-Learning Init | Improvement |
|--------|-------------|-------------------|-------------|
| First query NDCG | 0.42 | **0.68** | +62% |
| Queries to convergence | 94 | **23** | 4.1× faster |
| Initial config quality | Random | Vertical-informed | Higher |

**Privacy Guarantees**

```scala
object PrivacyPreservation {

  def shouldShareProfile(tenant: TenantProfile): Boolean = {
    // Only share if tenant has enough queries (k-anonymity)
    tenant.observedQueries > 100 &&
    // And tenant hasn't opted out
    !tenant.optedOutOfMetaLearning
  }

  def sanitizeProfile(profile: TenantProfile): TenantProfile = {
    profile.copy(
      tenantId = TenantId("ANONYMIZED"),  // Remove tenant identity
      queryVolume = bucketize(profile.queryVolume),  // Coarsen granularity
      // Weights are aggregate statistics (Beta params), no individual query data
      phase1Weights = profile.phase1Weights,
      phase2Weights = profile.phase2Weights,
      phase3Weights = profile.phase3Weights
    )
  }

  // Differential privacy: add noise to aggregated Beta params
  def applyDifferentialPrivacy(
    beta: BetaDistribution,
    epsilon: Double = 0.1
  ): BetaDistribution = {
    val noise = LaplaceDistribution(scale = 1.0 / epsilon).sample()
    BetaDistribution(
      alpha = math.max(1.0, beta.alpha + noise),
      beta = math.max(1.0, beta.beta - noise)
    )
  }
}
```

**Anti-Patterns to Avoid**

```
❌ DON'T:
   - Share individual query logs across tenants
   - Use proprietary config weights (e.g., custom ranking boost specific to tenant's business)
   - Aggregate from < 2 tenants (privacy risk)
   - Share config weights for tenants with < 100 queries (sparse, unreliable)
   - Use meta-learning to infer competitive intelligence

✅ DO:
   - Only share aggregate Beta parameters (α, β)
   - Require k-anonymity (k ≥ 3 tenants for any aggregation)
   - Allow tenants to opt out of meta-learning contribution
   - Apply differential privacy when sharing
   - Document which priors came from which vertical (transparency)
```

**Monitoring Meta-Learning Effectiveness**

```yaml
metrics:
  - name: cold_start_ndcg_first_query
    type: histogram
    labels: [tenant_id, initialization_method]
    description: "NDCG@10 on very first query for new tenant"
    buckets: [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]

  - name: queries_to_convergence
    type: histogram
    labels: [tenant_id, vertical, scale]
    description: "Number of queries until config weights stabilize (variance < 0.1)"
    buckets: [10, 25, 50, 100, 200, 500]

  - name: meta_learning_priors_used
    type: counter
    labels: [phase, config_id, source_vertical]
    description: "How often each prior from meta-learning is actually used"

  - name: prior_vs_learned_divergence
    type: gauge
    labels: [tenant_id, phase]
    description: "KL-divergence between initial prior and learned weights after 100 queries"
```

**When Meta-Learning Fails (Graceful Degradation)**

```scala
object FallbackStrategies {

  def handleInsufficientData(
    newTenant: TenantId,
    vertical: Vertical,
    existingProfiles: List[TenantProfile]
  ): TenantProfile = {

    val relevantTenants = existingProfiles.filter(_.vertical == vertical)

    relevantTenants.size match {
      case 0 =>
        // No tenants in this vertical → use uniform priors
        logger.warn(s"No existing tenants for vertical $vertical, using uniform priors")
        initializeWithUniformPriors(newTenant, vertical)

      case 1 =>
        // Only 1 tenant → use with caution (higher variance)
        logger.info(s"Only 1 tenant for vertical $vertical, priors may be noisy")
        initializeFromSingleTenant(newTenant, vertical, relevantTenants.head)

      case n if n >= 2 =>
        // 2+ tenants → proceed with meta-learning
        MetaLearning.initializeNewTenant(newTenant, vertical, scale, existingProfiles)
    }
  }
}
```

**References**

- **Finn et al. (2017)**: "Model-Agnostic Meta-Learning for Fast Adaptation of Deep Networks" (MAML)
- **Andrychowicz et al. (2016)**: "Learning to learn by gradient descent by gradient descent"
- **Contextual Bandits with Transfer Learning**: Similar priors across related tasks

---

## 6. Domain Lexicon Extraction

### 6.1 Motivation

**Problem**: Generic English dictionaries don't capture domain-specific terminology:
- "QLED" is not in standard dictionaries, but is critical for TV search
- "sneakers" vs "trainers" usage varies by operator (US vs UK)
- Product brand names ("Samsung", "LG") are entities, not dictionary words

**Solution**: Build domain lexicons from crawled content rather than relying solely on dictionaries.

### 6.2 Entity Discovery During Crawling

```scala
object DomainLexiconBuilder {

  case class EntityObservation(
    term: String,
    context: Context,           // Where it appeared (product title, category, etc.)
    frequency: Int,
    coOccurrences: Map[String, Int]  // Other terms appearing nearby
  )

  def extractFromCrawl(crawledPages: Seq[CrawledPage], tenantId: TenantId): TenantLexicon = {
    val observations = crawledPages.flatMap { page =>
      extractEntities(page.content).map { entity =>
        EntityObservation(
          term = entity.text,
          context = inferContext(page, entity),
          frequency = 1,
          coOccurrences = findCoOccurring(page, entity)
        )
      }
    }

    // Aggregate observations
    val termStats = observations.groupBy(_.term).map { case (term, obs) =>
      term -> TermStatistics(
        totalOccurrences = obs.map(_.frequency).sum,
        contexts = obs.map(_.context).toSet,
        coOccurringTerms = obs.flatMap(_.coOccurrences).groupBy(_._1).mapValues(_.map(_._2).sum)
      )
    }

    // Infer canonical forms and synonyms
    val canonicalForms = inferCanonicalForms(termStats)
    val synonyms = discoverSynonyms(termStats, canonicalForms)
    val entities = extractNamedEntities(termStats)  // Brands, product lines

    TenantLexicon(
      tenantId = tenantId,
      canonicalForms = canonicalForms,
      synonyms = synonyms,
      entities = entities,
      technicalTerms = identifyTechnicalTerms(termStats)
    )
  }

  private def inferCanonicalForms(stats: Map[String, TermStatistics]): Map[String, String] = {
    // Canonical form = most frequent variant
    // Example: {"TV" → 500 occurrences, "television" → 50 occurrences}
    //          → Canonical: "TV", Synonym: "television" → "TV"

    val variants = findVariants(stats)  // Group "TV" and "television" as variants

    variants.map { case (variantGroup, terms) =>
      val canonical = terms.maxBy(t => stats(t).totalOccurrences)
      terms.filterNot(_ == canonical).map(_ -> canonical).toMap
    }.reduce(_ ++ _)
  }

  private def discoverSynonyms(
    stats: Map[String, TermStatistics],
    canonicalForms: Map[String, String]
  ): Map[String, Set[String]] = {

    // Synonyms discovered via:
    // 1. Edit distance (Levenshtein)
    // 2. Co-occurrence in same contexts
    // 3. Interchangeable usage in product titles

    stats.keys.combinations(2).flatMap { case Seq(term1, term2) =>
      val similarity = computeSemanticSimilarity(term1, term2, stats)
      if (similarity > 0.80) {
        Some((term1, term2))
      } else None
    }.groupBy(_._1).mapValues(_.map(_._2).toSet)
  }
}
```

### 6.3 Lexicon Schema

```scala
case class TenantLexicon(
  tenantId: TenantId,
  canonicalForms: Map[String, String],  // "television" → "TV"
  synonyms: Map[String, Set[String]],   // "TV" → {"television", "telly", "flatscreen"}
  entities: Set[Entity],                // Brands, product lines
  technicalTerms: Set[TechnicalTerm],   // "QLED", "4K", "Dolby Atmos"
  createdAt: Instant,
  lastUpdated: Instant
)

case class Entity(
  name: String,
  entityType: EntityType,  // Brand, ProductLine, Category
  aliases: Set[String]
)

sealed trait EntityType
case object Brand extends EntityType
case object ProductLine extends EntityType
case object Category extends EntityType

case class TechnicalTerm(
  term: String,
  definition: Option[String],
  relatedTerms: Set[String]
)

case class SharedGlobalLexicon(
  language: Language,
  discoveredSynonyms: Map[String, Set[String]],  // Promoted from tenants
  technicalRegistry: Set[TechnicalTerm],          // Shared technical terms
  commonStopWords: Set[String],
  normalizationRules: List[NormalizationRule]
)
```

---

## 7. Query Processing with Multi-Tenant Lexicons

### 7.1 Query Expansion

```scala
object QueryPreprocessor {

  def expandQuery(
    query: String,
    tenantId: TenantId,
    language: Language
  ): ExpandedQuery = {

    val tenantLexicon = LexiconRegistry.getTenantLexicon(tenantId)
    val globalLexicon = LexiconRegistry.getGlobalLexicon(language)
    val baseDictionary = DictionaryRegistry.getBaseDictionary(language)

    // Tokenize query
    val tokens = Tokenizer.tokenize(query, language)

    // Resolve each token using hierarchy
    val expandedTokens = tokens.flatMap { token =>
      // 1. Tenant-specific (highest priority)
      tenantLexicon.synonyms.get(token)
        .map(synonyms => token +: synonyms.toSeq)
        .orElse {
          // 2. Shared global (medium priority)
          globalLexicon.discoveredSynonyms.get(token)
            .map(synonyms => token +: synonyms.toSeq)
        }
        .orElse {
          // 3. Base dictionary (fallback)
          baseDictionary.getSynonyms(token)
            .map(synonyms => token +: synonyms)
        }
        .getOrElse(Seq(token))  // Keep original if no expansion
    }

    ExpandedQuery(
      original = query,
      expanded = expandedTokens.distinct,
      tenantId = tenantId,
      language = language
    )
  }
}
```

**Example**:

```
Input Query: "QLED TV stand" (tenant: electronic-express, language: en)

Step 1: Tokenize → ["QLED", "TV", "stand"]

Step 2: Resolve "QLED"
  - Tenant lexicon: "QLED" → {"Quantum Dot", "QLED TV"} ✅
  - Result: ["QLED", "Quantum Dot", "QLED TV"]

Step 3: Resolve "TV"
  - Tenant lexicon: "TV" → {"television", "flatscreen"} ✅
  - Result: ["TV", "television", "flatscreen"]

Step 4: Resolve "stand"
  - Tenant lexicon: ❌ (not found)
  - Global lexicon: ❌ (not found)
  - Base dictionary: "stand" → {"furniture", "mount"} ✅
  - Result: ["stand", "furniture", "mount"]

Final Expanded Query:
  ["QLED", "Quantum Dot", "QLED TV", "TV", "television", "flatscreen", "stand", "furniture", "mount"]

OpenSearch Query (BM25):
  {
    "bool": {
      "filter": [{ "term": { "tenant_id": "electronic-express" }}],
      "should": [
        { "match": { "title": "QLED" }},
        { "match": { "title": "Quantum Dot" }},
        { "match": { "title": "TV" }},
        { "match": { "title": "television" }},
        { "match": { "title": "stand" }},
        { "match": { "title": "furniture" }}
      ]
    }
  }
```

---

## 8. Operational Considerations

### 8.1 Tenant Lifecycle Management

**Tenant States**:

```scala
sealed trait TenantState
case object Onboarding extends TenantState   // Initial setup, first crawl
case object Active extends TenantState       // Normal operation
case object Suspended extends TenantState    // Billing issue, contract pause
case object Archived extends TenantState     // Offboarded, data retained for compliance
case object Deleted extends TenantState      // Hard delete (GDPR right to erasure)
```

**Offboarding Process**:

```
TENANT OFFBOARDING CHECKLIST:

1. Suspend Search API Access
   - Return 403 Forbidden for tenant's API keys
   - Log final usage metrics

2. Stop Event Listeners
   - Unsubscribe from Kafka topics
   - Disable webhooks

3. Archive Tenant Data (Compliance Retention)
   - Export all products, lexicons, feedback to cold storage (S3 Glacier)
   - Retain for 7 years (compliance requirement)

4. Remove from Hot Storage (After Retention Period)
   - Delete OpenSearch documents (filtered by tenant_id)
   - Remove tenant lexicon from registry
   - Delete bandit state, config weights

5. Audit Trail
   - Log all deletion operations with timestamps
   - Generate offboarding report for compliance
```

### 8.2 Monitoring & Observability

**Per-Tenant Metrics**:

```yaml
metrics:
  - name: search_request_count
    type: counter
    labels: [tenant_id, status_code]

  - name: search_latency_seconds
    type: histogram
    labels: [tenant_id, query_type]
    buckets: [0.1, 0.25, 0.5, 1.0, 2.5, 5.0]

  - name: ndcg_score
    type: gauge
    labels: [tenant_id, config_id]

  - name: automated_test_pass_rate
    type: gauge
    labels: [tenant_id, test_type]

  - name: crawl_duration_seconds
    type: histogram
    labels: [tenant_id, crawl_type]

  - name: lexicon_size
    type: gauge
    labels: [tenant_id, lexicon_type]  # tenant, global, base
```

**Alerting**:

```yaml
alerts:
  - name: TenantNDCGDropped
    condition: ndcg_score{tenant_id="*"} < 0.60
    for: 1h
    action: email tenant contact, escalate to platform team

  - name: CrawlFailure
    condition: crawl_status{tenant_id="*", status="failed"} > 0
    for: 5m
    action: email tenant contact, retry crawl

  - name: HighErrorRate
    condition: rate(search_request_count{status_code=~"5.."}[5m]) > 0.05
    for: 10m
    action: page on-call engineer
```

### 8.3 Cost Allocation

**Per-Tenant Cost Tracking**:

```scala
case class TenantUsage(
  tenantId: TenantId,
  month: YearMonth,
  searchRequests: Long,
  indexedDocuments: Long,
  storageGB: Double,
  computeHours: Double,
  embeddingComputeHours: Double
)

object CostModel {
  // Tiered pricing
  val searchRequestCost = 0.001  // $0.001 per search
  val storageCostPerGB = 0.10    // $0.10 per GB/month
  val embeddingCost = 0.0001     // $0.0001 per embedding

  def calculateMonthlyCost(usage: TenantUsage): BigDecimal = {
    val searchCost = usage.searchRequests * searchRequestCost
    val storageCost = usage.storageGB * storageCostPerGB
    val embeddingCost = usage.indexedDocuments * this.embeddingCost

    BigDecimal(searchCost + storageCost + embeddingCost)
  }
}
```

---

## 9. Security & Compliance

### 9.1 Data Residency

**Multi-Region Tenants**:

```scala
case class TenantOnboardingConfig(
  // ... existing fields
  dataResidency: DataResidencyRequirement
)

sealed trait DataResidencyRequirement
case object USOnly extends DataResidencyRequirement
case object EUOnly extends DataResidencyRequirement  // GDPR compliance
case object APACOnly extends DataResidencyRequirement
case object GlobalReplication extends DataResidencyRequirement  // No restrictions
```

**Implementation**:
- Deploy OpenSearch clusters in multiple regions (US-East, EU-West, APAC)
- Route tenant data to appropriate cluster based on `dataResidency`
- Cross-region pattern learning: Only share non-sensitive patterns (synonyms, not user data)

### 9.2 Access Control

**Role-Based Access Control (RBAC)**:

```scala
sealed trait TenantRole
case object TenantAdmin extends TenantRole     // Full access to tenant config
case object TenantDeveloper extends TenantRole  // API keys, read-only config
case object TenantViewer extends TenantRole     // Read-only dashboards

case class TenantUser(
  userId: UserId,
  tenantId: TenantId,
  role: TenantRole,
  apiKeys: Set[ApiKey]
)

// API Key scopes
case class ApiKey(
  key: String,
  tenantId: TenantId,
  scopes: Set[ApiScope],
  expiresAt: Option[Instant]
)

sealed trait ApiScope
case object SearchRead extends ApiScope     // Execute search queries
case object FeedbackWrite extends ApiScope  // Submit user feedback
case object ConfigRead extends ApiScope     // View search config
case object ConfigWrite extends ApiScope    // Update search config (admin only)
```

### 9.3 Data Privacy (GDPR, CCPA)

**User Data Handling**:

```
PII (Personally Identifiable Information):
  - User queries: Stored for 90 days, then anonymized
  - Click feedback: Stored with hashed user IDs
  - Explicit feedback: Optional, anonymized after aggregation

Right to Erasure:
  - Tenant requests user data deletion → purge all feedback events for user ID
  - Tenant requests full account deletion → offboarding process (Section 8.1)

Data Portability:
  - Provide tenant export API: /api/v1/tenants/{tenant_id}/export
  - Exports: Product data, lexicons, aggregated metrics (CSV, JSON)
```

---

## 10. Migration & Backwards Compatibility

### 10.1 Adding New Tenants to Existing System

**Zero-Downtime Onboarding**:

```
1. Provision Tenant Configuration
   - Add TenantOnboardingConfig to registry
   - Generate API keys

2. Initial Crawl (Asynchronous)
   - Execute full crawl in background
   - No impact on existing tenants

3. Lexicon Extraction
   - Build tenant-specific lexicon
   - No cross-tenant impact

4. Automated Test Generation
   - Create brand/category coherence tests
   - Establish baseline metrics

5. Enable Search API
   - Tenant can now execute searches
   - Multi-armed bandit starts in exploration mode (cold start)
```

### 10.2 Schema Evolution

**Adding Fields to Shared Index**:

```json
// OpenSearch supports dynamic schema updates
PUT /commerce-products/_mapping
{
  "properties": {
    "new_field": {
      "type": "keyword"
    }
  }
}
```

**Backwards Compatibility**:
- New fields are optional by default
- Existing tenant data is not affected
- Tenants can opt-in to new fields via feature flags

---

## 11. Best Practices & Anti-Patterns

### 11.1 ✅ Best Practices

1. **Always Filter by Tenant ID**: Every OpenSearch query MUST include `tenant_id` filter
2. **Use Lexicon Hierarchy**: Tenant → Global → Dictionary (don't skip levels)
3. **Event Idempotency**: Ensure partial publish events can be replayed safely
4. **Pattern Promotion**: Only promote patterns seen in 3+ tenants with >0.85 confidence
5. **Monitor Per-Tenant Metrics**: Track NDCG, latency, cost independently
6. **Gradual Rollout**: Enable new features for 1-2 pilot tenants before global release

### 11.2 ❌ Anti-Patterns

1. **Shared API Keys**: Never share API keys across tenants (security risk)
2. **Cross-Tenant Queries**: Never query without `tenant_id` filter (data leakage)
3. **Proprietary Pattern Sharing**: Don't promote brand-specific patterns to global lexicon
4. **Ignoring Events**: Don't fall back to weekly crawl if events are misconfigured (alert instead)
5. **Manual Synonym Lists**: Don't maintain manual synonym files (use crawl-derived lexicons)

---

## 12. Future Enhancements

### 12.1 Multi-Language Tenants

**Challenge**: Some operators have catalogs in multiple languages (English + Spanish on same site)

**Solution**:
```scala
case class TenantOnboardingConfig(
  // ... existing fields
  languages: Set[Language],  // Changed from single `language` field
  primaryLanguage: Language
)

// Separate lexicons per language
case class TenantLexiconRegistry(
  tenantId: TenantId,
  lexicons: Map[Language, TenantLexicon]
)
```

### 12.2 Tenant Analytics Dashboard

**Features**:
- NDCG trends over time
- Top queries, zero-result queries
- Automated test pass rates
- Cost breakdown (storage, compute, API calls)
- Cross-tenant benchmarking (anonymized)

### 12.3 Tiered Service Levels

| Tier | Features | SLA | Price |
|------|----------|-----|-------|
| **Basic** | Weekly crawl, no events, 1K searches/day | 95% uptime | $99/month |
| **Pro** | Event-driven crawl, 10K searches/day, automated tests | 99% uptime | $499/month |
| **Enterprise** | Custom SLA, dedicated resources, white-glove support | 99.9% uptime | Custom |

---

## 13. References

- [CHARTER.md](../../CHARTER.md): Section 2.3 Multi-Tenant Architecture, Section 2.8 Data Models
- [Universal-Commerce-Index-Schema.md](Universal-Commerce-Index-Schema.md): Complete schema definition
- [Web-Crawling-Best-Practices.md](Web-Crawling-Best-Practices.md): Crawling patterns
- [OpenSearch-Vector-Search-Architecture.md](OpenSearch-Vector-Search-Architecture.md): Index design
- [Semantic-Search-Intent-Modeling.md](Semantic-Search-Intent-Modeling.md): Query processing

---

## 14. Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-01-04 | Author | Initial SBPF for multi-tenant architecture |
| 1.1.0 | 2026-01-04 | Author | Added Section 4.1: Universal Schema Principle; Emphasized schema universality vs data isolation |
| 1.2.0 | 2026-01-04 | Author | Added Section 5.4: Meta-Learning for Per-Phase Configuration Cold Start; Integration with hierarchical bandit architecture |

---

**Status**: ✅ **Active** - This SBPF is the authoritative reference for multi-tenant search platform design.
