# Automated Search Quality Testing from Site Structure

**Category**: SBPF (Shared Best Practices & Frameworks)
**Domain**: Commerce Search, Quality Assurance, Adaptive Systems
**Last Updated**: 2026-01-04
**Version**: 1.0.0

---

## Table of Contents

1. [Overview](#overview)
2. [Problem Statement](#problem-statement)
3. [Solution: Structure-Derived Quality Tests](#solution-structure-derived-quality-tests)
4. [Test Taxonomy](#test-taxonomy)
5. [Implementation Patterns](#implementation-patterns)
6. [Integration with Adaptive Ensemble](#integration-with-adaptive-ensemble)
7. [Best Practices](#best-practices)
8. [Measurement and Validation](#measurement-and-validation)
9. [Limitations and Mitigations](#limitations-and-mitigations)
10. [References](#references)

---

## Overview

**Purpose**: This document establishes patterns for auto-generating search quality tests from crawled e-commerce site structure, providing zero-cost ground truth for evaluating search configurations without human labeling.

**Key Innovation**: Commerce sites encode authoritative product-category relationships in their navigation structure (brand pages, category hierarchies, breadcrumbs, editorial collections). By treating these as ground truth, we can automatically generate thousands of quality tests that validate search relevance.

**Use Cases**:
- **Cold Start Evaluation**: Assess new search configurations before user traffic
- **Continuous Monitoring**: Daily regression testing to detect relevance drift
- **Reward Signal**: Provide objective feedback for adaptive ensemble weight learning

**Benefits**:
- Zero human labeling cost
- Comprehensive coverage (1,000+ tests vs typical 100-200 manual judgments)
- Objective, authoritative ground truth
- Enables offline experimentation

---

## Problem Statement

### The Cold Start Dilemma

Adaptive search systems face a critical challenge: **how to evaluate configurations before real user traffic?**

**Traditional Approaches:**

1. **Manual Relevance Judgments**
   - **Cost**: $50-200 per query (expert time)
   - **Scale**: Typically 100-200 labeled queries (budget constraints)
   - **Maintenance**: Must re-label when catalog changes

2. **Implicit Feedback (Clicks, Conversions)**
   - **Delay**: Requires weeks of traffic to accumulate
   - **Bias**: Click position bias, exploration/exploitation trade-off
   - **Sparsity**: Long-tail queries have no clicks

3. **Offline Metrics on Static Test Set**
   - **Staleness**: Test sets become outdated as catalog evolves
   - **Coverage**: Misses new products, categories, trends

### The False Choice

Organizations face a trade-off:
- **Quality**: Invest heavily in manual labeling → high cost, slow iteration
- **Speed**: Skip evaluation, deploy blindly → risk relevance degradation

### Our Solution

**Leverage site structure as authoritative ground truth**. If an e-commerce site curates a "Samsung TVs" page with 47 products, those 47 products **should** appear for query "Samsung TV"—no human judgment required.

---

## Solution: Structure-Derived Quality Tests

### Core Insight

E-commerce sites encode three types of ground truth:

1. **Taxonomic Relationships**: Brand pages, category pages, hierarchical breadcrumbs
2. **Editorial Curation**: "Best Budget Laptops 2026" collections, buying guides
3. **Navigational Intent**: Top-level navigation, featured sections

These relationships are **authoritative** (maintained by site operators) and **comprehensive** (cover entire catalog).

### Automated Test Generation Pipeline

```
Crawled Site Structure
         │
         ▼
┌─────────────────────┐
│ Structure Analyzer  │
│ - Extract brand pg  │
│ - Parse categories  │
│ - Identify clusters │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│  Test Generator     │
│ - Brand coherence   │
│ - Category mapping  │
│ - Navigational      │
│ - Collection        │
└─────────────────────┘
         │
         ▼
   1,000+ Quality Tests
         │
         ▼
┌─────────────────────┐
│  Test Executor      │
│ - Run vs config     │
│ - Compute scores    │
│ - Aggregate metrics │
└─────────────────────┘
         │
         ▼
  Configuration Ranking
```

---

## Test Taxonomy

### 4.1 Brand Coherence Tests

**Definition**: Products on a brand page should be returned for queries about that brand.

**Example**:
- Brand Page URL: `https://example.com/brands/samsung`
- Products on Page: {TV_1234, TV_5678, Phone_9012}
- Generated Query: "Samsung products"
- **Pass Condition**: recall@20 ≥ 0.70 (at least 70% of products in top 20 results)

**Implementation**:
```scala
case class BrandCoherenceTest(
  brandName: String,
  brandPageUrl: String,
  expectedProducts: Set[ProductId]
) extends AutomatedQualityTest {

  def generateQuery: String = s"$brandName products"

  def evaluate(results: List[SearchResult]): QualityScore = {
    val top20Ids = results.take(20).map(_.item.id).toSet
    val foundProducts = expectedProducts.intersect(top20Ids)
    val recall = foundProducts.size.toDouble / expectedProducts.size

    QualityScore(
      testType = "BrandCoherence",
      query = generateQuery,
      recallAt20 = recall,
      precisionAt20 = foundProducts.size.toDouble / top20Ids.size,
      penalty = if (recall < 0.70) (0.70 - recall) * -10.0 else 0.0
    )
  }
}
```

**Variants**:
- Strict: "Samsung TV" → only TVs from Samsung
- Broad: "Samsung" → any Samsung product
- Navigational: "Samsung official" → brand page should be top-1

### 4.2 Category Coherence Tests

**Definition**: Products in a category should be returned for category queries.

**Example**:
- Category: "4K Televisions"
- Breadcrumb: Home > Electronics > TVs > 4K TVs
- Products: {TV_4K_001, TV_4K_002, ...}
- Generated Queries: ["4K TVs", "4K televisions", "ultra HD TV"]
- **Pass Condition**: recall@20 ≥ 0.70 for primary query

**Implementation**:
```scala
case class CategoryCoherenceTest(
  categoryName: String,
  breadcrumbs: List[String],
  expectedProducts: Set[ProductId],
  synonyms: Set[String] = Set.empty
) extends AutomatedQualityTest {

  def generateQueries: List[String] = {
    val primary = categoryName
    val withSynonyms = synonyms.toList
    (primary :: withSynonyms)
  }

  def evaluate(results: List[SearchResult]): QualityScore = {
    // Test primary query
    val primaryResults = searchEngine.search(categoryName)
    val top20Ids = primaryResults.take(20).map(_.item.id).toSet
    val foundProducts = expectedProducts.intersect(top20Ids)

    QualityScore(
      testType = "CategoryCoherence",
      query = categoryName,
      recallAt20 = foundProducts.size.toDouble / expectedProducts.size,
      penalty = computePenalty(foundProducts.size, expectedProducts.size)
    )
  }

  private def computePenalty(found: Int, expected: Int): Double = {
    val missing = expected - found
    missing * -0.5  // Penalty scales with number of missing products
  }
}
```

### 4.3 Navigational Query Tests

**Definition**: Branded or entity queries should return the official page as top-1 result.

**Example**:
- Query: "Samsung support"
- Expected Top-1: Samsung brand page or Samsung support page
- **Pass Condition**: Expected page appears in top-3 results

**Implementation**:
```scala
case class NavigationalQueryTest(
  query: String,
  expectedUrl: String,
  acceptableUrls: Set[String] = Set.empty  // Alternative valid URLs
) extends AutomatedQualityTest {

  def evaluate(results: List[SearchResult]): QualityScore = {
    val top3Urls = results.take(3).map(_.item.url).toSet
    val allAcceptable = acceptableUrls + expectedUrl

    val found = top3Urls.intersect(allAcceptable).nonEmpty
    val rank = results.indexWhere(r => allAcceptable.contains(r.item.url))

    QualityScore(
      testType = "NavigationalQuery",
      query = query,
      recallAt3 = if (found) 1.0 else 0.0,
      mrr = if (rank >= 0) 1.0 / (rank + 1) else 0.0,
      penalty = if (!found) -5.0 else 0.0  // Navigational failures are severe
    )
  }
}
```

### 4.4 Collection Coherence Tests

**Definition**: Products in curated editorial collections should co-occur in search results.

**Example**:
- Collection: "Best Budget Laptops 2026"
- Products: {Laptop_A, Laptop_B, Laptop_C}
- Generated Query: "budget laptops"
- **Pass Condition**: At least 50% of collection products in top-20

**Implementation**:
```scala
case class CollectionCoherenceTest(
  collectionTitle: String,
  collectionUrl: String,
  curatedProducts: Set[ProductId],
  inferredQuery: String  // Extracted from title/description
) extends AutomatedQualityTest {

  def evaluate(results: List[SearchResult]): QualityScore = {
    val top20Ids = results.take(20).map(_.item.id).toSet
    val foundProducts = curatedProducts.intersect(top20Ids)

    QualityScore(
      testType = "CollectionCoherence",
      query = inferredQuery,
      recallAt20 = foundProducts.size.toDouble / curatedProducts.size,
      penalty = computeCollectionPenalty(foundProducts.size, curatedProducts.size)
    )
  }

  private def computeCollectionPenalty(found: Int, expected: Int): Double = {
    val recall = found.toDouble / expected
    if (recall < 0.5) (0.5 - recall) * -8.0 else 0.0
  }
}
```

---

## Implementation Patterns

### 5.1 Test Generation from Crawl Data

**Step 1: Structure Extraction**

```scala
case class CrawledSite(
  brandPages: List[BrandPage],
  categoryPages: List[CategoryPage],
  editorialCollections: List[Collection],
  navigationLinks: List[NavLink]
)

case class BrandPage(
  brandName: String,
  url: String,
  products: Set[ProductId],
  metadata: Map[String, String]
)

case class CategoryPage(
  categoryName: String,
  breadcrumbs: List[String],
  url: String,
  products: Set[ProductId],
  subcategories: List[String]
)
```

**Step 2: Test Generation**

```scala
object AutomatedTestGenerator {

  def generateTests(site: CrawledSite): List[AutomatedQualityTest] = {
    val brandTests = generateBrandTests(site.brandPages)
    val categoryTests = generateCategoryTests(site.categoryPages)
    val navTests = generateNavigationalTests(site.brandPages, site.categoryPages)
    val collectionTests = generateCollectionTests(site.editorialCollections)

    brandTests ++ categoryTests ++ navTests ++ collectionTests
  }

  private def generateBrandTests(pages: List[BrandPage]): List[BrandCoherenceTest] = {
    pages
      .filter(_.products.size >= 5)  // Skip brands with < 5 products
      .map { brand =>
        BrandCoherenceTest(
          brandName = brand.brandName,
          brandPageUrl = brand.url,
          expectedProducts = brand.products
        )
      }
  }

  private def generateCategoryTests(pages: List[CategoryPage]): List[CategoryCoherenceTest] = {
    pages
      .filter(_.products.size >= 10)  // Skip tiny categories
      .map { category =>
        val synonyms = inferSynonyms(category.categoryName)
        CategoryCoherenceTest(
          categoryName = category.categoryName,
          breadcrumbs = category.breadcrumbs,
          expectedProducts = category.products,
          synonyms = synonyms
        )
      }
  }

  private def inferSynonyms(categoryName: String): Set[String] = {
    // Use NLP or manual mapping
    categoryName.toLowerCase match {
      case name if name.contains("tv") => Set("television", "TVs", "televisions")
      case name if name.contains("laptop") => Set("notebooks", "portable computers")
      case _ => Set.empty
    }
  }

  private def generateNavigationalTests(
    brandPages: List[BrandPage],
    categoryPages: List[CategoryPage]
  ): List[NavigationalQueryTest] = {
    val brandNav = brandPages.map { brand =>
      NavigationalQueryTest(
        query = s"${brand.brandName} official",
        expectedUrl = brand.url,
        acceptableUrls = Set(brand.url, s"${brand.url}/")
      )
    }

    val categoryNav = categoryPages
      .filter(_.breadcrumbs.size == 1)  // Top-level categories only
      .map { category =>
        NavigationalQueryTest(
          query = category.categoryName,
          expectedUrl = category.url
        )
      }

    brandNav ++ categoryNav
  }

  private def generateCollectionTests(
    collections: List[Collection]
  ): List[CollectionCoherenceTest] = {
    collections
      .filter(_.products.size >= 5)
      .map { collection =>
        CollectionCoherenceTest(
          collectionTitle = collection.title,
          collectionUrl = collection.url,
          curatedProducts = collection.products,
          inferredQuery = inferQueryFromTitle(collection.title)
        )
      }
  }

  private def inferQueryFromTitle(title: String): String = {
    // Extract search query from collection title
    // "Best Budget Laptops 2026" → "budget laptops"
    title
      .toLowerCase
      .replaceAll("\\d{4}", "")  // Remove years
      .replaceAll("best |top |recommended ", "")  // Remove modifiers
      .trim
  }
}
```

### 5.2 Test Execution

```scala
trait AutomatedQualityTest {
  def evaluate(results: List[SearchResult]): QualityScore
  def generateQuery: String
  def testType: String
}

case class QualityScore(
  testType: String,
  query: String,
  recallAt20: Double,
  precisionAt20: Double = 0.0,
  mrr: Double = 0.0,
  penalty: Double = 0.0
) {
  def overallScore: Double = {
    val baseScore = (recallAt20 * 0.7) + (precisionAt20 * 0.3)
    math.max(0.0, baseScore + penalty)
  }
}

class TestExecutor(searchEngine: SearchEngine) {

  def executeTests(
    tests: List[AutomatedQualityTest],
    config: SearchConfig
  ): TestReport = {
    val scores = tests.map { test =>
      val query = test.generateQuery
      val results = searchEngine.search(query, config)
      test.evaluate(results)
    }

    TestReport(
      config = config,
      totalTests = tests.size,
      scores = scores,
      passRate = computePassRate(scores),
      avgRecall = scores.map(_.recallAt20).sum / scores.size,
      avgPenalty = scores.map(_.penalty).sum / scores.size
    )
  }

  private def computePassRate(scores: List[QualityScore]): Double = {
    val passed = scores.count(_.recallAt20 >= 0.70)
    passed.toDouble / scores.size
  }
}

case class TestReport(
  config: SearchConfig,
  totalTests: Int,
  scores: List[QualityScore],
  passRate: Double,
  avgRecall: Double,
  avgPenalty: Double
) {
  def rank: Int = ???  // Assigned by comparator
}
```

### 5.3 Batch Evaluation

```scala
object ConfigRanker {

  def rankConfigurations(
    configs: List[SearchConfig],
    tests: List[AutomatedQualityTest],
    searchEngine: SearchEngine
  ): List[(SearchConfig, TestReport)] = {

    val executor = new TestExecutor(searchEngine)

    val reports = configs.map { config =>
      val report = executor.executeTests(tests, config)
      (config, report)
    }

    // Rank by composite score
    reports.sortBy { case (_, report) =>
      val compositeScore =
        (report.passRate * 0.5) +
        (report.avgRecall * 0.4) +
        (math.min(0.0, report.avgPenalty) * -0.1)
      -compositeScore  // Descending
    }
  }
}
```

---

## Integration with Adaptive Ensemble

### 6.1 Three-Pillar Reward Signal

Automated tests are one of three feedback sources:

```scala
def computeReward(
  query: Query,
  config: SearchConfig,
  userFeedback: Option[UserFeedback],
  automatedScore: Option[Double]
): Double = {
  val userReward = userFeedback.map(computeUserReward).getOrElse(0.5)
  val automatedReward = automatedScore.getOrElse(0.5)

  // Dynamic weighting: cold start vs post-traffic
  val (userWeight, automatedWeight) = if (userFeedback.isEmpty) {
    (0.2, 0.8)  // Cold start: rely heavily on automated tests
  } else {
    (0.6, 0.4)  // Post-traffic: user feedback dominates
  }

  userWeight * userReward + automatedWeight * automatedReward
}
```

### 6.2 Cold Start Initialization

**Before User Traffic**:

```scala
object ColdStartInitializer {

  def initializeConfigWeights(
    configs: List[SearchConfig],
    tests: List[AutomatedQualityTest],
    searchEngine: SearchEngine
  ): Map[SearchConfig, Double] = {

    val rankedConfigs = ConfigRanker.rankConfigurations(configs, tests, searchEngine)

    // Initialize weights using softmax over test scores
    val scores = rankedConfigs.map { case (_, report) => report.passRate }
    val weights = softmax(scores)

    rankedConfigs.zip(weights).map { case ((config, _), weight) =>
      config -> weight
    }.toMap
  }

  private def softmax(scores: List[Double]): List[Double] = {
    val maxScore = scores.max
    val expScores = scores.map(s => math.exp((s - maxScore) * 5.0))  // Temperature = 0.2
    val sumExp = expScores.sum
    expScores.map(_ / sumExp)
  }
}
```

### 6.3 Continuous Monitoring

**Daily Regression Testing**:

```scala
class ContinuousQualityMonitor(
  tests: List[AutomatedQualityTest],
  searchEngine: SearchEngine,
  alertThreshold: Double = 0.85  // Alert if pass rate drops below 85%
) {

  def runDailyTests(currentConfig: SearchConfig): MonitoringReport = {
    val executor = new TestExecutor(searchEngine)
    val report = executor.executeTests(tests, currentConfig)

    if (report.passRate < alertThreshold) {
      sendAlert(s"Quality degradation: pass rate ${report.passRate} < $alertThreshold")
    }

    MonitoringReport(
      date = LocalDate.now(),
      config = currentConfig,
      testReport = report,
      alertTriggered = report.passRate < alertThreshold
    )
  }

  private def sendAlert(message: String): Unit = {
    // Send to monitoring system (Slack, PagerDuty, etc.)
    println(s"[ALERT] $message")
  }
}
```

---

## Best Practices

### 7.1 Test Quality

**✅ DO**:
- Filter brands/categories with ≥5 products (avoid sparse tests)
- Use multiple query variants per test (synonyms, typos)
- Set reasonable thresholds (70% recall@20 for brand tests)
- Weight tests by product count (larger categories = more important)

**❌ DON'T**:
- Generate tests for obscure brands with 1-2 products
- Expect 100% recall (some products may be out of stock)
- Treat all tests equally (weight by business importance)

### 7.2 Query Generation

**Good Query Patterns**:
- Brand: `"{brand} products"`, `"{brand}"`, `"{brand} {category}"`
- Category: `"{category}"`, `"{category} deals"`, `"best {category}"`
- Navigational: `"{brand} official"`, `"{brand} store"`, `"{category} page"`

**Bad Query Patterns**:
- Overly specific: `"Samsung 55-inch QLED 4K TV model QN55Q80TAFXZA"` (too narrow)
- Ambiguous: `"electronics"` (too broad, many valid results)

### 7.3 Threshold Tuning

| Test Type | Recall@20 Threshold | Rationale |
|-----------|---------------------|-----------|
| Brand Coherence | 70% | Allow for out-of-stock, discontinued items |
| Category Coherence | 70% | Same as brand |
| Navigational | 100% @ top-3 | Navigational queries must succeed |
| Collection | 50% | Editorial curation may be subjective |

### 7.4 Maintenance

**Re-crawl Frequency**:
- **Weekly**: For fast-moving catalogs (fashion, electronics)
- **Monthly**: For stable catalogs (appliances, furniture)

**Test Refresh**:
- Regenerate tests after each crawl
- Archive old tests for historical comparison
- Track test churn (how many tests changed?)

---

## Measurement and Validation

### 8.1 Test Coverage Metrics

```scala
case class TestCoverageReport(
  totalProducts: Int,
  productsCoveredByTests: Int,
  coverageRate: Double,
  testsByType: Map[String, Int],
  avgProductsPerTest: Double
) {
  def summary: String = {
    s"""
    |Test Coverage Report
    |====================
    |Total Products: $totalProducts
    |Products Covered: $productsCoveredByTests (${"%.1f".format(coverageRate * 100)}%)
    |Total Tests: ${testsByType.values.sum}
    |  - Brand Tests: ${testsByType.getOrElse("BrandCoherence", 0)}
    |  - Category Tests: ${testsByType.getOrElse("CategoryCoherence", 0)}
    |  - Navigational: ${testsByType.getOrElse("NavigationalQuery", 0)}
    |  - Collection: ${testsByType.getOrElse("CollectionCoherence", 0)}
    |Avg Products/Test: ${"%.1f".format(avgProductsPerTest)}
    """.stripMargin
  }
}
```

### 8.2 Correlation with User Feedback

**Validation**: Do automated tests predict user satisfaction?

```scala
case class CorrelationAnalysis(
  automatedScores: Map[SearchConfig, Double],
  userFeedbackScores: Map[SearchConfig, Double]
) {
  def spearmanCorrelation: Double = {
    // Rank configs by automated score
    val automatedRanks = automatedScores.toList.sortBy(-_._2).map(_._1).zipWithIndex.toMap

    // Rank configs by user feedback score
    val userRanks = userFeedbackScores.toList.sortBy(-_._2).map(_._1).zipWithIndex.toMap

    // Compute Spearman's ρ
    val dSquared = automatedRanks.map { case (config, autoRank) =>
      val userRank = userRanks(config)
      val d = autoRank - userRank
      d * d
    }.sum

    val n = automatedRanks.size
    1.0 - (6.0 * dSquared) / (n * (n * n - 1))
  }
}
```

**Expected Results**:
- **ρ > 0.75**: Automated tests strongly predict user feedback
- **ρ = 0.5-0.75**: Moderate correlation (supplement with user feedback)
- **ρ < 0.5**: Weak correlation (investigate test quality issues)

### 8.3 Drift Detection

**Track Pass Rate Over Time**:

```scala
class DriftDetector(historical: List[TestReport]) {

  def detectDrift(currentReport: TestReport): DriftAlert = {
    val recentAvgPassRate = historical.takeRight(7).map(_.passRate).sum / 7.0
    val drop = recentAvgPassRate - currentReport.passRate

    if (drop > 0.10) {
      DriftAlert(
        severity = "HIGH",
        message = s"Pass rate dropped ${drop * 100}% vs 7-day avg"
      )
    } else if (drop > 0.05) {
      DriftAlert(severity = "MEDIUM", message = s"Pass rate declined ${drop * 100}%")
    } else {
      DriftAlert(severity = "LOW", message = "No significant drift")
    }
  }
}
```

---

## Limitations and Mitigations

### 9.1 Limitations

| Limitation | Impact | Mitigation |
|------------|--------|------------|
| **Assumes well-curated site** | Poorly structured sites yield low-quality tests | Pre-validate crawl quality; skip sites with < 50 brands |
| **Cannot capture subjective relevance** | User preferences (color, price) not in site structure | Combine with user feedback (three-pillar approach) |
| **Requires periodic re-crawl** | Tests become stale as catalog changes | Weekly/monthly re-crawl; track test churn |
| **Brand/category ambiguity** | "Apple" (brand vs fruit), "Bass" (fish vs audio) | Disambiguate via breadcrumbs, product type filters |

### 9.2 Mitigation Strategies

**1. Quality Gating**:
```scala
def validateCrawlQuality(site: CrawledSite): Boolean = {
  site.brandPages.size >= 50 &&
  site.categoryPages.size >= 20 &&
  site.brandPages.map(_.products.size).sum >= 1000
}
```

**2. Hybrid Scoring**:
- Use automated tests for **cold start**
- Blend with user feedback after 100+ queries
- Trust user feedback for subjective queries ("best", "stylish")

**3. Test Weighting**:
```scala
def computeTestWeight(test: AutomatedQualityTest): Double = test match {
  case BrandCoherenceTest(_, _, products) =>
    math.log(products.size + 1)  // Larger brands = higher weight
  case NavigationalQueryTest(_, _, _) =>
    2.0  // Navigational failures are critical
  case _ => 1.0
}
```

---

## References

### Academic

- **Baeza-Yates, R., & Ribeiro-Neto, B. (2011).** *Modern Information Retrieval*. Chapter on test collection construction.
- **Voorhees, E. M. (2000).** "Variations in relevance judgments and the measurement of retrieval effectiveness." *Information Processing & Management*, 36(5), 697-716.

### Industry

- **Google Search Quality Rater Guidelines** (2023): Using site structure for evaluating page quality.
- **Amazon Search Relevance Metrics** (Mehta et al., 2023): Leveraging catalog structure for offline evaluation.

### Internal

- [CHARTER.md](../../../../CHARTER.md): Section 2.2 on Automated Quality Testing
- [thesis-self-tuning-adaptive-search.md](../../../../thesis-self-tuning-adaptive-search.md): Section 6.3 on Automated Tests
- [Semantic-Search-Intent-Modeling.md](./Semantic-Search-Intent-Modeling.md): Three-phase pipeline integration

---

## Appendix: Example Test Suite

**ElectronicExpress.com Test Coverage**:

| Test Type | Count | Example Query | Pass Threshold |
|-----------|-------|---------------|----------------|
| Brand Coherence | 147 | "Samsung products" | Recall@20 ≥ 0.70 |
| Category Coherence | 63 | "4K TVs" | Recall@20 ≥ 0.70 |
| Navigational | 42 | "Sony official" | MRR@3 = 1.0 |
| Collection | 891 | "budget laptops" | Recall@20 ≥ 0.50 |
| **Total** | **1,143** | - | - |

**Execution Time**:
- Single config: ~6 minutes (1,143 tests × 0.3s/test)
- 15 configs (parallel): ~48 minutes (15 × 6 / 2 workers)

**Storage**:
- Test definitions: ~2 MB (JSON)
- Historical reports (90 days): ~500 MB

---

**Document Status**: ✅ Ready for Implementation
**Next Review**: After Phase 1 crawling complete
**Stakeholders**: Tech Lead, QA, Product Owner
