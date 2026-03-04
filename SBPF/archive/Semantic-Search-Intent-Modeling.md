# Semantic Search and Intent Modeling for Commerce

**Version**: 1.0.0
**Last Updated**: 2026-01-04
**Audience**: Architects, ML Engineers, Developers

---

## Purpose

This document establishes best practices for building **intent-aware semantic search** systems for digital commerce. It covers the three-phase functional pipeline (preprocessing → retrieval → ranking), intent classification strategies, concept extraction, and techniques for aligning search results with user goals.

---

## The Three-Phase Functional Pipeline

### Architecture Principle: Pure Functions

Each phase is implemented as a **pure function** (deterministic, no side effects) to enable:
- **Composability**: Reuse phases in non-search contexts (recommendations, analytics)
- **Testability**: Property-based testing with ScalaCheck
- **Debuggability**: Reproduce results exactly for any input

```scala
// Phase 1: Preprocessing
def preprocess(query: RawQuery): EnrichedQuery = {
  val normalized = normalize(query.text)
  val concepts = extractConcepts(normalized)
  val intent = classifyIntent(normalized, concepts)
  val embedding = generateEmbedding(normalized)

  EnrichedQuery(query, normalized, concepts, intent, embedding)
}

// Phase 2: Retrieval
def retrieve(enrichedQuery: EnrichedQuery, index: SearchIndex): CandidateSet = {
  val sparseResults = index.searchBM25(enrichedQuery.normalized)
  val denseResults = index.searchKNN(enrichedQuery.embedding, k = 100)
  val filtered = applyFilters(sparseResults ++ denseResults, enrichedQuery.filters)

  CandidateSet(filtered)
}

// Phase 3: Ranking
def rank(candidates: CandidateSet, enrichedQuery: EnrichedQuery): RankedResults = {
  val intentScored = scoreByIntent(candidates, enrichedQuery.intent)
  val diversified = diversifyByType(intentScored)
  val final = sortByRelevance(diversified)

  RankedResults(final, enrichedQuery.intent)
}
```

---

## Phase 1: Query Preprocessing

### 1.1 Text Normalization

**Goal**: Canonicalize query text to improve retrieval consistency.

**Techniques**:
1. **Lowercasing**: "LAPTOP" → "laptop"
2. **Whitespace normalization**: "best  laptop" → "best laptop"
3. **Punctuation removal**: "best laptop!" → "best laptop"
4. **Stopword removal** (optional): "the best laptop" → "best laptop"
5. **Stemming/Lemmatization**: "running shoes" → "run shoe" (use cautiously; can harm precision)

**Implementation**:
```scala
object TextNormalizer {
  private val stopwords = Set("the", "a", "an", "in", "on", "at", "for", "to")

  def normalize(text: String): String = {
    text
      .toLowerCase
      .replaceAll("[^a-z0-9\\s]", "")  // Remove punctuation
      .replaceAll("\\s+", " ")         // Collapse whitespace
      .trim
  }

  def removeStopwords(text: String): String = {
    text.split("\\s+")
      .filterNot(stopwords.contains)
      .mkString(" ")
  }
}

// Property-based test
property("normalization should be idempotent") {
  forAll { (text: String) =>
    val normalized = TextNormalizer.normalize(text)
    TextNormalizer.normalize(normalized) shouldEqual normalized
  }
}
```

**Trade-off**: Aggressive normalization improves recall but may harm precision (e.g., "IPhone" vs "iphone").

---

### 1.2 Concept Extraction

**Goal**: Identify domain-specific concepts (activities, attributes, categories) from query text.

**Approach 1: Dictionary-Based (Simple, Fast)**

```scala
case class Concept(name: String, synonyms: Set[String], category: String)

object ConceptDictionary {
  val concepts: Set[Concept] = Set(
    Concept("hiking", Set("hiking", "trekking", "trail", "outdoor walking"), "activity"),
    Concept("gaming", Set("gaming", "video games", "esports", "gaming pc"), "activity"),
    Concept("energy efficient", Set("energy efficient", "low power", "eco-friendly", "energy star"), "attribute"),
    Concept("4k", Set("4k", "ultra hd", "uhd", "2160p"), "attribute")
  )

  def extract(text: String): Set[Concept] = {
    concepts.filter { concept =>
      (concept.synonyms ++ Set(concept.name)).exists(text.toLowerCase.contains)
    }
  }
}

// Example
ConceptDictionary.extract("best laptop for gaming and 4K video editing")
// Result: Set(Concept("gaming", ...), Concept("4k", ...))
```

**Approach 2: NLP-Based (Advanced, Slower)**

Use NER (Named Entity Recognition) or keyphrase extraction:

```scala
import com.johnsnowlabs.nlp.pretrained.PretrainedPipeline

object NLPConceptExtractor {
  private lazy val pipeline = PretrainedPipeline("recognize_entities_dl", lang = "en")

  def extract(text: String): Set[Concept] = {
    val result = pipeline.annotate(text)
    val entities = result("entities").toSet

    // Map entities to domain concepts
    entities.flatMap(mapToConcept)
  }

  private def mapToConcept(entity: String): Option[Concept] = {
    // Custom mapping logic
    entity.toLowerCase match {
      case e if e.contains("gaming") => Some(Concept("gaming", Set("gaming"), "activity"))
      case e if e.contains("energy") => Some(Concept("energy efficient", Set("energy efficient"), "attribute"))
      case _ => None
    }
  }
}
```

**Recommendation**: Start with dictionary-based (fast, deterministic); upgrade to NLP if needed.

---

### 1.3 Intent Classification

**Goal**: Infer user intent to guide ranking strategy.

**Intent Taxonomy for Commerce**:

| Intent | Definition | Query Examples | Content to Surface |
|--------|------------|----------------|-------------------|
| **Learn** | Informational; seeking knowledge | "how to choose laptop", "OLED vs QLED" | Guides, articles, comparisons |
| **Shop** | Transactional; ready to buy | "buy gaming laptop", "best laptop under $800" | Products, bundles |
| **Troubleshoot** | Problem-solving; fix or diagnose | "laptop won't turn on", "TV flickering fix" | FAQs, manuals, troubleshooting guides |
| **Compare** | Evaluation; side-by-side analysis | "MacBook vs ThinkPad", "compare 4K TVs" | Comparison tables, reviews |
| **Explore** | Browsing; no specific goal | "new laptops", "what's popular" | Categories, trending products, curated collections |

#### Rule-Based Intent Classifier (Simple)

```scala
sealed trait Intent
case object LearnIntent extends Intent
case object ShopIntent extends Intent
case object TroubleshootIntent extends Intent
case object CompareIntent extends Intent
case object ExploreIntent extends Intent

object RuleBasedIntentClassifier {
  def classify(query: String, concepts: Set[Concept]): Intent = {
    val lower = query.toLowerCase

    // Troubleshoot patterns
    if (lower.matches(".*(won't|doesn't|not working|broken|fix|repair|error).*")) {
      return TroubleshootIntent
    }

    // Compare patterns
    if (lower.matches(".*(vs|versus|compare|difference between).*")) {
      return CompareIntent
    }

    // Learn patterns
    if (lower.matches(".*(how to|what is|guide|tutorial|best way|tips|advice|choose).*")) {
      return LearnIntent
    }

    // Shop patterns (transactional language)
    if (lower.matches(".*(buy|purchase|price|cheapest|under \\$|deals|sale).*")) {
      return ShopIntent
    }

    // Default: Explore (browsing)
    ExploreIntent
  }
}

// Property-based test
property("intent classifier should handle negations correctly") {
  forAll { (item: String) =>
    val query = s"$item won't turn on"
    RuleBasedIntentClassifier.classify(query, Set.empty) shouldEqual TroubleshootIntent
  }
}
```

#### ML-Based Intent Classifier (Advanced)

Train a text classifier on labeled queries:

```scala
import smile.classification.RandomForest
import smile.feature.Bag

case class TrainingExample(query: String, intent: Intent)

object MLIntentClassifier {
  private lazy val model: RandomForest = trainModel()

  private def trainModel(): RandomForest = {
    val trainingData: List[TrainingExample] = List(
      TrainingExample("how to choose a laptop", LearnIntent),
      TrainingExample("buy gaming laptop under $1000", ShopIntent),
      TrainingExample("TV won't turn on", TroubleshootIntent),
      TrainingExample("MacBook vs ThinkPad", CompareIntent)
      // ... 1000+ examples
    )

    val (features, labels) = trainingData.map { ex =>
      (featurize(ex.query), intentToLabel(ex.intent))
    }.unzip

    RandomForest.fit(features.toArray, labels.toArray)
  }

  private def featurize(query: String): Array[Double] = {
    // Bag-of-words or TF-IDF features
    Bag.of(query.split("\\s+")).toArray.map(_.toDouble)
  }

  def classify(query: String): Intent = {
    val features = featurize(query)
    val label = model.predict(features)
    labelToIntent(label)
  }
}
```

**Recommendation**: Start with rule-based (fast, interpretable); add ML if accuracy < 90%.

---

### 1.4 Embedding Generation

**Goal**: Convert query text into a dense vector for semantic retrieval.

**Model Selection** (see [Embedding-Strategies-Commerce.md](Embedding-Strategies-Commerce.md) for details):
- **MPNet** (multilingual-mpnet-base-v2): 768-dim, multilingual, proven for semantic search
- **BGE** (BAAI/bge-large-en): 1024-dim, state-of-the-art for English
- **E5** (intfloat/e5-large-v2): 1024-dim, strong zero-shot performance

**Implementation (Python via HTTP API)**:
```python
from sentence_transformers import SentenceTransformer

model = SentenceTransformer('sentence-transformers/multi-qa-mpnet-base-dot-v1')

def generate_embedding(text: str) -> List[float]:
    embedding = model.encode(text, convert_to_tensor=False)
    return embedding.tolist()

# Example
embedding = generate_embedding("best laptops for students")
# Result: [0.123, -0.456, ..., 0.789]  (768 dimensions)
```

**Scala Integration (via HTTP microservice)**:
```scala
import akka.http.scaladsl.Http
import akka.http.scaladsl.model._

object EmbeddingService {
  private val embeddingApiUrl = "http://localhost:8000/embed"

  def generate(text: String): Future[Array[Float]] = {
    val request = HttpRequest(
      method = HttpMethods.POST,
      uri = embeddingApiUrl,
      entity = HttpEntity(ContentTypes.`application/json`, s"""{"text": "$text"}""")
    )

    Http().singleRequest(request).flatMap { response =>
      Unmarshal(response).to[EmbeddingResponse].map(_.embedding)
    }
  }
}

case class EmbeddingResponse(embedding: Array[Float])
```

---

## Phase 2: Retrieval (Candidate Generation)

### 2.1 Hybrid Retrieval Strategy

**Goal**: Maximize recall by combining sparse (BM25) and dense (k-NN) retrieval.

```scala
def hybridRetrieve(enrichedQuery: EnrichedQuery, index: OpenSearchIndex): CandidateSet = {
  // Parallel execution
  val sparseFuture = Future { index.searchBM25(enrichedQuery.normalized, size = 100) }
  val denseFuture = Future { index.searchKNN(enrichedQuery.embedding, k = 100) }

  for {
    sparse <- sparseFuture
    dense <- denseFuture
  } yield {
    val combined = (sparse ++ dense).distinctBy(_.id)
    CandidateSet(combined)
  }
}
```

**Score Fusion** (see [OpenSearch-Vector-Search-Architecture.md](OpenSearch-Vector-Search-Architecture.md)):
- Linear combination: `score = α × BM25 + β × k-NN`
- Reciprocal rank fusion (RRF): `score = 1/(k + rank_bm25) + 1/(k + rank_knn)`

---

### 2.2 Attribute Filtering

**Goal**: Apply structured filters (category, price, brand) to narrow candidates before ranking.

```scala
case class Filters(
  categories: Option[Set[String]],
  priceRange: Option[(Double, Double)],
  brands: Option[Set[String]],
  attributes: Map[String, String]
)

def applyFilters(candidates: CandidateSet, filters: Filters): CandidateSet = {
  var filtered = candidates.items

  filters.categories.foreach { cats =>
    filtered = filtered.filter(item => cats.contains(item.category))
  }

  filters.priceRange.foreach { case (min, max) =>
    filtered = filtered.filter(item => item.price >= min && item.price <= max)
  }

  filters.brands.foreach { brands =>
    filtered = filtered.filter(item => brands.contains(item.brand))
  }

  CandidateSet(filtered)
}
```

**OpenSearch Implementation**: Use `filter` clause (not `must`) to avoid affecting scores.

---

## Phase 3: Ranking (Intent-Aware Re-Scoring)

### 3.1 Intent-Based Type Weighting

**Goal**: Promote content types that align with detected intent.

```scala
object IntentWeighting {
  def weight(item: Item, intent: Intent): Double = {
    (item.`type`, intent) match {
      // Learn intent: Prioritize guides and articles
      case ("guide", LearnIntent) => 2.0
      case ("article", LearnIntent) => 1.8
      case ("product", LearnIntent) => 0.5

      // Shop intent: Prioritize products
      case ("product", ShopIntent) => 2.0
      case ("guide", ShopIntent) => 0.8

      // Troubleshoot intent: Prioritize FAQs and manuals
      case ("faq", TroubleshootIntent) => 2.0
      case ("manual", TroubleshootIntent) => 1.8
      case ("product", TroubleshootIntent) => 0.3

      // Compare intent: Prioritize comparison articles
      case ("comparison", CompareIntent) => 2.0
      case ("article", CompareIntent) => 1.5
      case ("product", CompareIntent) => 1.0

      // Explore intent: Balanced
      case (_, ExploreIntent) => 1.0

      // Default: neutral
      case _ => 1.0
    }
  }
}

def rankByIntent(candidates: CandidateSet, enrichedQuery: EnrichedQuery): RankedResults = {
  val scored = candidates.items.map { item =>
    val baseScore = item.retrievalScore
    val intentBoost = IntentWeighting.weight(item, enrichedQuery.intent)
    val finalScore = baseScore * intentBoost

    ScoredItem(item, finalScore)
  }

  RankedResults(scored.sortBy(-_.score))
}
```

**BDD Scenario**:
```gherkin
Scenario: Learning intent prioritizes guides over products
  Given a query "how to choose a laptop" with intent "learn"
  And candidates include:
    | Type    | Title                     | Base Score |
    | guide   | Laptop Buying Guide       | 0.8        |
    | product | Dell XPS 15               | 0.9        |
  When ranking with intent weighting
  Then "Laptop Buying Guide" should rank #1
  And "Dell XPS 15" should rank #2
```

---

### 3.2 Diversity and Type Distribution

**Goal**: Avoid homogeneous result sets (e.g., all products, no guides).

**Greedy Diversification Algorithm**:
```scala
def diversify(items: List[ScoredItem], maxPerType: Int = 3): List[ScoredItem] = {
  val typeCounts = mutable.Map[String, Int]().withDefaultValue(0)

  items.filter { item =>
    if (typeCounts(item.`type`) < maxPerType) {
      typeCounts(item.`type`) += 1
      true
    } else {
      false
    }
  }
}

// Example: Limit products to top 3, guides to top 2
def diversifyByIntent(items: List[ScoredItem], intent: Intent): List[ScoredItem] = {
  intent match {
    case LearnIntent => diversify(items, maxPerType = Map("guide" -> 5, "product" -> 3))
    case ShopIntent => diversify(items, maxPerType = Map("product" -> 10, "guide" -> 2))
    case _ => items
  }
}
```

**Trade-off**: Improves diversity but may lower top-1 relevance.

---

### 3.3 Personalization (Future Enhancement)

**Approach**: Adjust scores based on user behavior (clicks, purchases, dwell time).

```scala
case class UserProfile(
  preferredCategories: Set[String],
  priceRange: (Double, Double),
  recentlyViewed: List[String]
)

def personalizeScore(item: ScoredItem, user: UserProfile): Double = {
  var score = item.score

  // Boost items in preferred categories
  if (user.preferredCategories.contains(item.category)) {
    score *= 1.2
  }

  // Boost items in user's price range
  if (item.price >= user.priceRange._1 && item.price <= user.priceRange._2) {
    score *= 1.1
  }

  // Penalize recently viewed (reduce redundancy)
  if (user.recentlyViewed.contains(item.id)) {
    score *= 0.8
  }

  score
}
```

**Note**: Requires user tracking; out of scope for POC.

---

## Evaluation Metrics

### Offline Metrics (Pre-Deployment)

| Metric | Formula | Interpretation | Target |
|--------|---------|----------------|--------|
| **NDCG@k** | `Σ (2^rel - 1) / log2(i + 1)` | Ranking quality (discounted by position) | >0.75 |
| **MRR** | `1 / rank_first_relevant` | How quickly users find relevant result | >0.70 |
| **Recall@k** | `relevant_in_top_k / total_relevant` | Coverage of relevant items | >0.85 |
| **Precision@k** | `relevant_in_top_k / k` | Proportion of relevant results | >0.60 |

### Online Metrics (Post-Deployment)

| Metric | Definition | Target |
|--------|------------|--------|
| **CTR** (Click-Through Rate) | `clicks / impressions` | >20% |
| **Add-to-Cart Rate** | `add_to_cart / clicks` | >5% |
| **Zero-Result Rate** | `queries_with_0_results / total_queries` | <5% |
| **Session Abandonment** | `sessions_without_click / total_sessions` | <15% |

### Intent Classification Accuracy

```scala
test("intent classifier should achieve >90% accuracy on validation set") {
  val validationSet: List[(String, Intent)] = loadValidationSet()

  val correct = validationSet.count { case (query, trueIntent) =>
    IntentClassifier.classify(query) == trueIntent
  }

  val accuracy = correct.toDouble / validationSet.size
  accuracy should be > 0.90
}
```

---

## Testing Strategies

### 1. Unit Tests (Pure Functions)

```scala
test("concept extraction should identify 'hiking' from 'best shoes for hiking'") {
  val concepts = ConceptDictionary.extract("best shoes for hiking")
  concepts.map(_.name) should contain("hiking")
}

test("intent weighting should boost guides for learn intent") {
  val guide = Item(id = "1", `type` = "guide", title = "Guide")
  val weight = IntentWeighting.weight(guide, LearnIntent)
  weight should be > 1.5
}
```

### 2. Property-Based Tests (Invariants)

```scala
property("ranking should preserve top-k order after re-scoring") {
  forAll { (items: List[ScoredItem]) =>
    val ranked = rank(items, LearnIntent)
    ranked.zip(ranked.tail).forall { case (a, b) => a.score >= b.score }
  }
}

property("diversification should respect maxPerType constraint") {
  forAll { (items: List[ScoredItem], maxPerType: Int) =>
    val diversified = diversify(items, maxPerType)
    diversified.groupBy(_.`type`).values.forall(_.size <= maxPerType)
  }
}
```

### 3. BDD Scenarios (End-to-End)

```gherkin
Feature: Intent-Aware Semantic Search

  Scenario: Spanish query retrieves English products
    Given an index with product "Nike Running Shoes"
    When user searches for "zapatillas para correr" in Spanish
    Then "Nike Running Shoes" should appear in top 10
    And NDCG@10 should be > 0.80

  Scenario: Troubleshoot intent surfaces FAQs
    Given a query "TV won't turn on" with intent "troubleshoot"
    When retrieving and ranking candidates
    Then top 3 results should be type "faq" or "manual"
    And products should rank below position 5
```

---

## Anti-Patterns

### ❌ Skipping Intent Classification
**Problem**: All queries treated identically; poor user experience.
**Solution**: Always classify intent; use it to guide ranking.

### ❌ Using Only BM25 (No Semantic Retrieval)
**Problem**: Misses cross-language and synonym queries.
**Solution**: Always use hybrid (BM25 + k-NN).

### ❌ Returning Homogeneous Result Types
**Problem**: All products or all guides; user can't find what they need.
**Solution**: Apply diversity constraints.

### ❌ Hardcoding Intent Weights
**Problem**: Cannot adapt to new data or user behavior.
**Solution**: Make weights configurable; A/B test different values.

### ❌ Ignoring Zero-Result Queries
**Problem**: Users frustrated; lost sales.
**Solution**: Track zero-result rate; add fallback suggestions (e.g., "Did you mean...?").

---

## Related SBPF Documents

- **[Web-Crawling-Best-Practices.md](Web-Crawling-Best-Practices.md)**: Building the corpus for semantic search
- **[OpenSearch-Vector-Search-Architecture.md](OpenSearch-Vector-Search-Architecture.md)**: Indexing and hybrid retrieval
- **[Embedding-Strategies-Commerce.md](Embedding-Strategies-Commerce.md)**: Generating embeddings for queries and items

---

## References

- **Three-Phase Search Model**: See [thesis.md](../../../thesis.md)
- **Intent Classification**: Carmel et al. (2017) - "Estimating the Query Difficulty for Information Retrieval" ([ACM](https://dl.acm.org/doi/10.1145/3077136.3080750))
- **NDCG Metric**: Järvelin & Kekäläinen (2002) - "Cumulated Gain-Based Evaluation" ([ACM](https://dl.acm.org/doi/10.1145/582415.582418))
- **Sentence Transformers**: Reimers & Gurevych (2019) - "Sentence-BERT" ([arXiv](https://arxiv.org/abs/1908.10084))

---

**Summary**: Implement search as a three-phase functional pipeline. Classify intent using rules or ML. Weight results by intent to align with user goals. Use hybrid retrieval (BM25 + k-NN) for maximum recall. Apply diversity constraints to avoid homogeneous results. Measure with NDCG, MRR, and CTR. Test with property-based and BDD approaches.
