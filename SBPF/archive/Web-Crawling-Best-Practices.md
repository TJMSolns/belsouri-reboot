# Web Crawling Best Practices for Semantic Commerce Search

**Version**: 1.1.0
**Last Updated**: 2026-01-04
**Audience**: Developers, Architects

---

## Purpose

This document establishes best practices for web crawling in the context of building semantic search corpora for digital commerce. Unlike general-purpose web crawling (e.g., for SEO or archival), commerce-focused crawling must extract **semantic relationships** encoded in site structure, navigation patterns, and editorial content to enable intent-aware, concept-driven search.

---

## Key Principles

### 1. Crawl for Context, Not Just Content

**Problem**: Structured product feeds (CSV, JSON) contain titles, descriptions, and attributes, but miss the **semantic context** that reveals how the merchant understands their own domain.

**Solution**: Crawl the live site to capture:
- **Navigation hierarchies**: Category trees, breadcrumbs, facets
- **Editorial relationships**: "Related products," "People also buy," comparison tables
- **Content clustering**: Buying guides near product categories, troubleshooting FAQs near product pages
- **URL semantics**: `/gaming-laptops/4k-display/120hz/` reveals attributes (gaming, 4K, 120Hz)
- **Page depth**: Distance from homepage indicates importance/specificity

**Example**:
```scala
case class CrawledPage(
  url: String,
  title: String,
  content: String,
  breadcrumbs: List[String],           // ["Home", "Electronics", "Laptops", "Gaming"]
  relatedLinks: List[String],          // Semantic adjacency
  depth: Int,                          // Distance from homepage
  urlAttributes: Map[String, String]   // Extracted from URL path
)
```

---

### 2. Respect Robots.txt and Rate Limits

**Ethical crawling prevents blocking and legal issues.**

**Best Practices**:
- **Parse `robots.txt`** before crawling; honor `Disallow` directives
- **Set User-Agent** to identify your crawler (e.g., `CommerceSearchBot/1.0`)
- **Rate limit**: 1-2 requests/second max (slower for small sites)
- **Backoff on errors**: Exponential backoff for 429 (Too Many Requests) or 503 (Service Unavailable)
- **Crawl during off-peak hours** (if known)

**Implementation (Scala)**:
```scala
object CrawlerConfig {
  val userAgent = "CommerceSearchBot/1.0 (+https://github.com/yourorg/commerce-search)"
  val maxRequestsPerSecond = 1.5
  val backoffMultiplier = 2
  val maxRetries = 3
}

def fetchWithBackoff(url: String, retries: Int = 0): Future[Response] = {
  HttpClient.get(url, headers = Map("User-Agent" -> CrawlerConfig.userAgent))
    .recoverWith {
      case RateLimitException if retries < CrawlerConfig.maxRetries =>
        val delay = Math.pow(CrawlerConfig.backoffMultiplier, retries).seconds
        after(delay)(fetchWithBackoff(url, retries + 1))
      case e => Future.failed(e)
    }
}
```

---

### 3. Extract Structured Data and Microdata

**E-commerce sites often embed structured data (Schema.org, JSON-LD) that provides clean product metadata.**

**Target Formats**:
- **JSON-LD**: `<script type="application/ld+json">` with `@type: Product`
- **Microdata**: HTML attributes like `itemprop="name"`, `itemprop="price"`
- **Open Graph**: `<meta property="og:title">` for social sharing metadata
- **Breadcrumb lists**: `@type: BreadcrumbList` (reveals category hierarchy)

**Extraction Pattern (Scala with JSoup)**:
```scala
import org.jsoup.Jsoup
import org.jsoup.nodes.Document
import play.api.libs.json._

def extractJsonLd(html: String): List[JsValue] = {
  val doc: Document = Jsoup.parse(html)
  val scripts = doc.select("script[type='application/ld+json']")

  scripts.asScala.toList.flatMap { script =>
    Try(Json.parse(script.html())).toOption
  }
}

def extractProduct(jsonLd: JsValue): Option[ProductData] = {
  (jsonLd \ "@type").asOpt[String] match {
    case Some("Product") =>
      Some(ProductData(
        name = (jsonLd \ "name").as[String],
        description = (jsonLd \ "description").asOpt[String],
        category = (jsonLd \ "category").asOpt[String],
        brand = (jsonLd \ "brand" \ "name").asOpt[String],
        price = (jsonLd \ "offers" \ "price").asOpt[Double]
      ))
    case _ => None
  }
}
```

---

### 4. Normalize and Deduplicate URLs

**Challenge**: Same content at multiple URLs (query params, trailing slashes, `www` vs non-`www`)

**Solution**:
- **Canonicalize URLs**: Remove tracking params, normalize slashes, enforce lowercase
- **Respect `<link rel="canonical">`**: Use canonical URL if declared
- **Hash-based deduplication**: Hash content to detect duplicates even with different URLs

**Normalization (Scala)**:
```scala
import java.net.URL
import scala.util.Try

def normalizeUrl(rawUrl: String): Option[String] = {
  Try {
    val url = new URL(rawUrl)
    val protocol = url.getProtocol.toLowerCase
    val host = url.getHost.toLowerCase.replaceFirst("^www\\.", "")
    val path = url.getPath.replaceAll("/+$", "")  // Remove trailing slashes
    val cleanQuery = Option(url.getQuery).map(cleanQueryParams).getOrElse("")

    s"$protocol://$host$path" + (if (cleanQuery.nonEmpty) s"?$cleanQuery" else "")
  }.toOption
}

def cleanQueryParams(query: String): String = {
  // Remove tracking params like utm_source, fbclid
  val trackingParams = Set("utm_source", "utm_medium", "utm_campaign", "fbclid", "gclid")
  query.split("&")
    .filterNot(param => trackingParams.exists(param.startsWith))
    .sorted  // Consistent ordering
    .mkString("&")
}
```

---

### 5. Parse and Store Navigation Context

**Key insight**: Breadcrumbs and category pages reveal the merchant's **conceptual taxonomy**.

**What to Extract**:
- **Breadcrumbs**: `Home > Electronics > Laptops > Gaming` → hierarchy depth and path
- **Category filters**: Facets on category pages (e.g., "Screen Size: 15-inch, 17-inch")
- **Related categories**: Links to sibling/parent categories

**Storage Pattern**:
```scala
case class NavigationContext(
  breadcrumbs: List[String],
  categoryPath: String,         // "/electronics/laptops/gaming"
  facets: Map[String, Set[String]],  // "Screen Size" -> {"15-inch", "17-inch"}
  siblingCategories: List[String]
)

// Store alongside content for indexing
case class CrawledItem(
  url: String,
  title: String,
  content: String,
  navContext: NavigationContext,
  structuredData: Option[ProductData]
)
```

---

### 6. Handle JavaScript-Rendered Content

**Challenge**: Many e-commerce sites use client-side rendering (React, Vue) that JSoup cannot parse.

**Solutions**:
1. **Headless browser** (Selenium, Playwright): Slow but accurate
2. **Pre-rendered snapshots**: Use services like Prerender.io (if available)
3. **API endpoint discovery**: Inspect network traffic for JSON APIs (faster than rendering)

**Headless Browser Pattern (Playwright + Scala)**:
```scala
import com.microsoft.playwright._

object HeadlessCrawler {
  def fetch(url: String): String = {
    val playwright = Playwright.create()
    val browser = playwright.chromium().launch()
    val page = browser.newPage()

    page.navigate(url)
    page.waitForLoadState(LoadState.NETWORKIDLE)  // Wait for JS to finish

    val html = page.content()

    browser.close()
    playwright.close()

    html
  }
}
```

**Trade-off**: 10-100x slower than static HTML parsing. Use only when necessary (detect via initial JSoup parse).

---

### 7. Incremental and Resumable Crawling

**Problem**: Crawls can fail mid-process (network errors, rate limits, crashes).

**Solution**: Implement **stateful crawling** with persistent queue.

**Architecture**:
```scala
trait CrawlQueue {
  def enqueue(url: String, priority: Int): Unit
  def dequeue(): Option[String]
  def markCompleted(url: String): Unit
  def markFailed(url: String, reason: String): Unit
}

// Example: Redis-backed queue (persistent)
class RedisCrawlQueue(redis: RedisClient) extends CrawlQueue {
  def enqueue(url: String, priority: Int): Unit = {
    redis.zadd("crawl:pending", priority, url)
  }

  def dequeue(): Option[String] = {
    redis.zpopmin("crawl:pending", 1).headOption
  }

  def markCompleted(url: String): Unit = {
    redis.sadd("crawl:completed", url)
  }

  def markFailed(url: String, reason: String): Unit = {
    redis.hset("crawl:failed", url, reason)
  }
}
```

**Resumption**: On restart, read from `crawl:pending` and skip `crawl:completed`.

---

### 8. Event-Driven Crawling for Multi-Tenant Systems

**Context**: In a multi-tenant search platform, operators (e-commerce sites) publish catalog change events that should trigger appropriate crawl/index jobs. This enables near-real-time index updates without constant polling.

#### 8.1 Event Integration Patterns

**Operators notify the platform of catalog changes via**:

```scala
sealed trait EventSource

// Webhook: Platform provides endpoint, operator POSTs events
case class WebhookSource(
  url: URL,           // Platform-provided endpoint
  secret: String      // HMAC signature verification
) extends EventSource

// Kafka: Platform consumes from operator's topic
case class KafkaTopicSource(
  topic: String,
  bootstrapServers: String,
  consumerGroup: String
) extends EventSource

// AWS SQS: Platform polls operator's queue
case class SQSQueueSource(
  queueUrl: String,
  region: AWSRegion
) extends EventSource
```

**Event Types**:

| Event Type | Scope | Crawl Strategy | Use Case |
|------------|-------|----------------|----------|
| **FullPublishEvent** | Complete catalog | Full re-crawl + re-index (immediate) | Site redesign, migration, annual catalog reset |
| **PartialPublishEvent** | Delta (changed URLs) | Incremental crawl (hourly batched) | Daily product additions, price updates, category changes |

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

#### 8.2 Crawl Scheduling Logic

**Event-Driven Scheduler**:

```scala
object CrawlOrchestrator {

  def determineCrawlStrategy(
    tenant: TenantId,
    event: Option[OperatorEvent],
    lastCrawl: Instant,
    config: TenantConfig
  ): CrawlStrategy = {

    event match {
      // IMMEDIATE: Full catalog refresh
      case Some(FullPublishEvent(_, timestamp)) =>
        FullCrawlStrategy(
          priority = High,
          trigger = Immediate,
          scope = EntireSite(tenant),
          reason = "Operator triggered full publish"
        )

      // HOURLY BATCH: Accumulate partial events, execute once per hour
      case Some(PartialPublishEvent(_, changedUrls, timestamp)) =>
        IncrementalCrawlStrategy(
          priority = Medium,
          trigger = HourlyBatch,
          scope = SpecificUrls(tenant, changedUrls),
          reason = s"Operator published ${changedUrls.size} changed URLs"
        )

      // WEEKLY FALLBACK: No events received
      case None =>
        val timeSinceLastCrawl = Duration.between(lastCrawl, Instant.now)
        if (timeSinceLastCrawl > config.indexSchedule.defaultCrawl) {
          FullCrawlStrategy(
            priority = Low,
            trigger = Scheduled(config.indexSchedule.defaultCrawl),
            scope = EntireSite(tenant),
            reason = "Scheduled weekly crawl (no events)"
          )
        } else {
          NoCrawlNeeded
        }
    }
  }
}
```

**Crawl Frequency Matrix**:

| Event Received | Trigger Policy | Priority | Example |
|----------------|----------------|----------|---------|
| `full_publish` | Immediate (within 5 min) | High | Site publishes new holiday catalog → full re-index immediately |
| `partial_publish` | Hourly batch (accumulate 1h) | Medium | 50 new products added → batch and crawl at top of hour |
| No events | Weekly (fallback schedule) | Low | Operator has no event integration → crawl every 7 days |

#### 8.3 Incremental Crawl Implementation

**Challenge**: For partial events, only crawl changed URLs to minimize load.

**Implementation**:

```scala
trait CrawlStrategy {
  def execute(tenant: TenantId): Future[CrawlResult]
}

case class IncrementalCrawlStrategy(
  priority: Priority,
  trigger: TriggerPolicy,
  scope: CrawlScope,
  reason: String
) extends CrawlStrategy {

  def execute(tenant: TenantId): Future[CrawlResult] = {
    scope match {
      case SpecificUrls(_, urls) =>
        // Crawl only changed URLs
        val results = urls.map { url =>
          for {
            html <- CrawlerExecutor.fetch(url)
            extracted <- ContentExtractor.extract(html, url)
            enriched <- Enricher.enrich(extracted, tenant)
          } yield enriched
        }

        Future.sequence(results).map { items =>
          CrawlResult(
            tenant = tenant,
            itemsProcessed = items.size,
            newItems = items.count(_.isNew),
            updatedItems = items.count(!_.isNew),
            timestamp = Instant.now()
          )
        }

      case EntireSite(_) =>
        // Fallback to full crawl if scope is entire site
        FullCrawlStrategy(priority, trigger, scope, reason).execute(tenant)
    }
  }
}
```

#### 8.4 Event Validation & Security

**HMAC Signature Verification (Webhooks)**:

```scala
object WebhookValidator {

  def validateSignature(
    payload: String,
    signature: String,
    secret: String
  ): Boolean = {
    val mac = Mac.getInstance("HmacSHA256")
    val secretKey = new SecretKeySpec(secret.getBytes("UTF-8"), "HmacSHA256")
    mac.init(secretKey)

    val computed = mac.doFinal(payload.getBytes("UTF-8"))
    val expectedSignature = s"sha256=${Hex.encodeHexString(computed)}"

    // Constant-time comparison to prevent timing attacks
    MessageDigest.isEqual(
      expectedSignature.getBytes("UTF-8"),
      signature.getBytes("UTF-8")
    )
  }

  def processWebhook(request: HttpRequest): Either[ValidationError, OperatorEvent] = {
    val signature = request.headers.get("X-Hub-Signature-256")
    val payload = request.body

    signature match {
      case Some(sig) if validateSignature(payload, sig, getTenantSecret(request)) =>
        parseEvent(payload)  // Proceed to parse event JSON

      case Some(_) =>
        Left(InvalidSignature("HMAC signature verification failed"))

      case None =>
        Left(MissingSignature("X-Hub-Signature-256 header required"))
    }
  }
}
```

#### 8.5 Tenant Isolation in Event-Driven Crawling

**Critical**: Each tenant's crawl must be isolated to prevent data leakage.

```scala
case class TenantCrawlContext(
  tenantId: TenantId,
  siteRoot: URL,
  allowedDomains: Set[String],  // Only crawl tenant's domains
  lexicon: TenantLexicon,       // Tenant-specific vocabulary
  crawlState: CrawlState         // Visited URLs, queue state
)

object TenantIsolationGuard {

  def validateUrl(url: URL, context: TenantCrawlContext): Either[IsolationViolation, URL] = {
    val domain = url.getHost

    if (context.allowedDomains.contains(domain)) {
      Right(url)
    } else {
      Left(IsolationViolation(
        s"URL $url outside allowed domains for tenant ${context.tenantId}. Allowed: ${context.allowedDomains}"
      ))
    }
  }

  def enforceIsolation(crawlJob: CrawlJob): CrawlJob = {
    crawlJob.copy(
      beforeFetch = (url) => validateUrl(url, crawlJob.context),
      afterExtract = (item) => item.copy(tenantId = crawlJob.context.tenantId)  // Always tag with tenant
    )
  }
}
```

#### 8.6 Monitoring Event-Driven Crawls

**Metrics to Track**:

```yaml
metrics:
  - name: event_received_total
    type: counter
    labels: [tenant_id, event_type]

  - name: crawl_triggered_total
    type: counter
    labels: [tenant_id, trigger_policy]

  - name: crawl_duration_seconds
    type: histogram
    labels: [tenant_id, crawl_type]
    buckets: [5, 30, 60, 300, 600]

  - name: urls_processed
    type: counter
    labels: [tenant_id, crawl_type]

  - name: event_to_index_latency_seconds
    type: histogram
    labels: [tenant_id]
    description: "Time from event received to index updated"
    buckets: [60, 300, 600, 1800, 3600]
```

**Alerting**:

```yaml
alerts:
  - name: CrawlFailure
    condition: crawl_status{tenant_id="*", status="failed"} > 0
    for: 5m
    action: email tenant contact, retry crawl

  - name: HighEventToIndexLatency
    condition: event_to_index_latency_seconds{quantile="0.95"} > 3600
    for: 30m
    action: page on-call, investigate crawl bottleneck

  - name: MissedPartialEvents
    condition: rate(event_received_total{event_type="partial_publish"}[1h]) == 0
      AND tenant_has_event_integration == true
    for: 2h
    action: email tenant, check event source connectivity
```

#### 8.7 Best Practices for Event-Driven Crawling

**✅ DO**:
1. **Batch partial events**: Accumulate for 1 hour to avoid excessive crawling
2. **Validate events**: Verify HMAC signatures, check tenant exists, sanitize URLs
3. **Prioritize full publishes**: Process immediately (within 5 minutes)
4. **Isolate tenant crawls**: Enforce allowed domains, tag all data with tenant ID
5. **Monitor latency**: Track time from event received to index updated
6. **Fallback to scheduled**: If no events for 7 days, trigger full crawl anyway

**❌ DON'T**:
1. **Don't crawl on every event**: Batch partials to avoid overload
2. **Don't trust event URLs blindly**: Validate domain belongs to tenant
3. **Don't skip weekly fallback**: Events can be misconfigured or fail
4. **Don't share crawl state across tenants**: Isolated queues, isolated lexicons
5. **Don't ignore failed events**: Retry with exponential backoff, alert on persistent failures

#### 8.8 Integration with Existing Crawl Infrastructure

**Adapting stateful crawling (Section 7) for event-driven**:

```scala
// Extend existing CrawlQueue with event-driven priorities
trait EventAwareCrawlQueue extends CrawlQueue {

  // Priority levels: 100 (immediate), 50 (hourly), 10 (weekly)
  def enqueueEvent(event: OperatorEvent, tenant: TenantId): Unit = event match {
    case FullPublishEvent(_, _) =>
      // Clear existing queue for tenant, enqueue full crawl at highest priority
      clearTenantQueue(tenant)
      enqueue(s"${tenant.value}:full-crawl", priority = 100)

    case PartialPublishEvent(_, urls, _) =>
      // Add changed URLs to hourly batch queue
      urls.foreach { url =>
        enqueue(s"${tenant.value}:$url", priority = 50)
      }
  }

  def clearTenantQueue(tenant: TenantId): Unit = {
    // Remove all pending URLs for tenant (full publish supersedes partials)
    redis.zremrangebyscore(s"crawl:pending:${tenant.value}", "-inf", "+inf")
  }
}
```

---

### 9. Content Extraction Heuristics

**Goal**: Extract main content while ignoring boilerplate (headers, footers, ads).

**Techniques**:
1. **Boilerplate removal**: Use libraries like `boilerpipe` or `readability` (port to Scala)
2. **CSS selectors**: If site structure is known (e.g., `<div class="product-description">`)
3. **Density-based**: Prefer `<div>` with highest text-to-tag ratio
4. **Semantic HTML5**: Prefer `<article>`, `<main>`, `<section>` over generic `<div>`

**Example (Readability-style heuristic)**:
```scala
import org.jsoup.Jsoup
import org.jsoup.nodes.Element

def extractMainContent(html: String): String = {
  val doc = Jsoup.parse(html)

  // Prefer semantic tags
  val article = doc.select("article, main, [role=main]").first()
  if (article != null) return article.text()

  // Fallback: highest text density
  val candidates = doc.select("div, section")
  candidates.asScala.maxByOption(textDensity).map(_.text()).getOrElse("")
}

def textDensity(elem: Element): Double = {
  val text = elem.ownText()
  val tags = elem.select("*").size()
  if (tags == 0) 0.0 else text.length.toDouble / tags
}
```

---

### 9. Politeness and Ethical Crawling

**Principles**:
- **Crawl only public pages**: Do not attempt to bypass authentication
- **Respect `noindex` meta tags**: Skip pages with `<meta name="robots" content="noindex">`
- **Identify yourself**: Use descriptive User-Agent and provide contact URL
- **Cache responses**: Don't re-fetch same URL unnecessarily (use ETag, Last-Modified headers)

**ETag-based caching**:
```scala
case class CachedResponse(etag: String, content: String, timestamp: Instant)

object CrawlCache {
  private val cache = mutable.Map[String, CachedResponse]()

  def fetch(url: String): Future[String] = {
    cache.get(url) match {
      case Some(cached) if cached.timestamp.isAfter(Instant.now().minus(1.day)) =>
        // Use cached version if < 1 day old
        Future.successful(cached.content)

      case Some(cached) =>
        // Conditional request with ETag
        HttpClient.get(url, headers = Map("If-None-Match" -> cached.etag))
          .map {
            case NotModified => cached.content  // 304: Use cache
            case Response(newEtag, newContent) =>
              cache(url) = CachedResponse(newEtag, newContent, Instant.now())
              newContent
          }

      case None =>
        // First fetch
        HttpClient.get(url).map { case Response(etag, content) =>
          cache(url) = CachedResponse(etag, content, Instant.now())
          content
        }
    }
  }
}
```

---

### 10. Store Raw HTML for Debugging

**Why**: Parsing logic may need refinement; having raw HTML allows re-processing without re-crawling.

**Pattern**:
```scala
case class CrawlResult(
  url: String,
  rawHtml: String,              // Store for debugging/re-parsing
  extractedData: CrawledItem,
  crawledAt: Instant
)

// Persist to object storage (S3) or database (compressed)
def persist(result: CrawlResult): Unit = {
  val compressed = Gzip.compress(result.rawHtml)
  S3.put(s"crawl-raw/${result.url.hashCode}.html.gz", compressed)

  Database.insert("crawled_items", result.extractedData)
}
```

---

## Anti-Patterns

### ❌ Crawling Without Rate Limiting
**Problem**: Gets your IP banned; violates ethical standards.
**Solution**: Always enforce rate limits (see Principle 2).

### ❌ Ignoring Redirects
**Problem**: Misses canonical content.
**Solution**: Follow 301/302 redirects, store final URL.

### ❌ Scraping Dynamic Content Without Headless Browser
**Problem**: Misses JavaScript-rendered product data.
**Solution**: Use headless browser or discover underlying JSON APIs.

### ❌ Not Handling Crawl Failures Gracefully
**Problem**: Crash on single error; lose progress.
**Solution**: Use persistent queue; retry with backoff; log failures.

### ❌ Storing Only Extracted Data (No Raw HTML)
**Problem**: Cannot refine extraction logic later.
**Solution**: Store raw HTML (compressed) for re-processing.

---

## Testing Crawlers

### Unit Tests
```scala
test("normalizeUrl should remove tracking params") {
  val input = "https://www.example.com/product?id=123&utm_source=google"
  val expected = "https://example.com/product?id=123"
  normalizeUrl(input) shouldEqual Some(expected)
}

test("extractJsonLd should parse multiple JSON-LD scripts") {
  val html = """
    <script type="application/ld+json">{"@type":"Product","name":"Laptop"}</script>
    <script type="application/ld+json">{"@type":"BreadcrumbList","itemListElement":[]}</script>
  """
  val jsonLd = extractJsonLd(html)
  jsonLd should have length 2
}
```

### Property-Based Tests (ScalaCheck)
```scala
property("normalized URLs should be idempotent") {
  forAll { (url: String) =>
    val normalized = normalizeUrl(url)
    normalized.flatMap(normalizeUrl) shouldEqual normalized
  }
}
```

### Integration Tests (Testcontainers)
```scala
test("crawl should handle rate limiting") {
  // Spin up mock HTTP server that returns 429 on burst
  val mockServer = new MockWebServer()
  mockServer.enqueue(new MockResponse().setResponseCode(429))
  mockServer.enqueue(new MockResponse().setBody("<html>OK</html>"))

  val result = Crawler.fetch(mockServer.url("/test").toString)

  result shouldBe defined  // Should retry and succeed
  mockServer.getRequestCount shouldEqual 2
}
```

---

## Recommended Tools (Scala/JVM)

| Tool | Purpose | Maven/SBT Dependency |
|------|---------|----------------------|
| **JSoup** | HTML parsing (static content) | `org.jsoup:jsoup:1.17+` |
| **Playwright** | Headless browser (JS rendering) | `com.microsoft.playwright:playwright:1.40+` |
| **Akka HTTP** | HTTP client (non-blocking) | `com.typesafe.akka:akka-http_2.13:10.5+` |
| **Circe** | JSON parsing (JSON-LD) | `io.circe:circe-core_2.13:0.14+` |
| **Redisson** | Redis client (crawl queue) | `org.redisson:redisson:3.25+` |
| **ScalaCheck** | Property-based testing | `org.scalacheck:scalacheck_2.13:1.17+` |

---

## Related SBPF Documents

- **[Multi-Tenant-Search-Platform-Architecture.md](Multi-Tenant-Search-Platform-Architecture.md)**: Multi-tenant architecture, event-driven crawling, lexicon extraction
- **[OpenSearch-Vector-Search-Architecture.md](OpenSearch-Vector-Search-Architecture.md)**: How to index crawled content with embeddings
- **[Semantic-Search-Intent-Modeling.md](Semantic-Search-Intent-Modeling.md)**: Using crawled context for intent inference
- **[Embedding-Strategies-Commerce.md](Embedding-Strategies-Commerce.md)**: Generating embeddings from crawled content

---

## References

- **Robots Exclusion Protocol**: [robotstxt.org](https://www.robotstxt.org/)
- **Schema.org Product**: [schema.org/Product](https://schema.org/Product)
- **Boilerpipe (Java)**: Content extraction library ([GitHub](https://github.com/kohlschutter/boilerpipe))
- **Playwright**: Headless browser automation ([playwright.dev](https://playwright.dev/))

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-01-04 | Initial version covering core crawling best practices |
| 1.1.0 | 2026-01-04 | Added Section 8: Event-Driven Crawling for Multi-Tenant Systems |

---

**Summary**: Crawling for semantic search requires extracting **context** (navigation, relationships) beyond raw content. Follow ethical practices (robots.txt, rate limits), use incremental crawling with persistent queues, and store raw HTML for iterative refinement. In multi-tenant systems, implement event-driven crawling to respond to operator catalog changes with appropriate priority (immediate for full publishes, hourly batched for partials, weekly fallback). Testing should include unit, property-based, and integration tests.
