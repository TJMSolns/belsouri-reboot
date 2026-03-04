# Universal Commerce Index Schema

## SBPF Classification
- **Category**: Data Modeling & Index Architecture
- **Domain**: Commerce Search, Multi-Tenant SaaS
- **Audience**: Architects, Backend Engineers, Platform Engineers
- **Status**: Active
- **Last Updated**: 2026-01-04

---

## Executive Summary

This document defines the **canonical index schema** for all commerce tenants in the Open Commerce Search platform. Unlike traditional multi-tenant systems where each tenant might have different schemas, this architecture leverages the **bounded domain of commerce** to establish a universal schema that works for all e-commerce operators.

**Key Principle**: **Schema is universal, data is isolated.**

All commerce sites share the same core domain model (products, brands, categories, prices), making a unified schema not only possible but architecturally superior. This enables:
- **Horizontal scalability**: One index structure to optimize, shard, and manage
- **Cross-tenant learning**: Analyze field importance globally, new tenants benefit immediately
- **Operational simplicity**: Consistent query patterns, monitoring, and feature rollout
- **Resource efficiency**: Shared infrastructure (shards, replicas, merge policies)

**Trade-off**: Tenants cannot customize the schema, but this constraint is a strength—it forces a robust, commerce-specific domain model that serves all operators.

---

## 1. Design Philosophy

### 1.1 Why Universal Schema?

**Commerce is a Well-Defined Domain**

Unlike general-purpose SaaS platforms (CRM vs Project Management vs Inventory), **all commerce sites share the same core entities**:
- Products (title, description, price, brand, SKU, images)
- Categories/Taxonomies
- Brands
- Content (guides, FAQs, comparisons)
- Attributes (size, color, technical specs)

This is Domain-Driven Design applied correctly: identify the **core domain** and model it once, well.

**Separation of Concerns**

| Concern | Scope | Implementation |
|---------|-------|----------------|
| **Schema** | Universal (all tenants) | Single OpenSearch index template |
| **Data** | Tenant-isolated | Filtered by `tenant_id` field |
| **Lexicon** | Tenant-specific | Overlays on shared base dictionary |
| **Configuration** | Tenant-specific | Search weights, ranking strategies |

### 1.2 Architectural Benefits

**Scaling**:
- One index template to optimize (refresh intervals, merge policies, codec settings)
- OpenSearch shards by `tenant_id` as routing key while using same mapping
- Horizontal scaling = add nodes, not manage N schemas

**Cross-Tenant Learning**:
- Analyze which fields are important across ALL tenants (e.g., "brand" critical for nav queries)
- New tenants benefit from global learnings on day 1 (no cold start for schema)
- A/B test schema changes globally, measure impact statistically

**Operational Simplicity**:
- Consistent query patterns
- Unified monitoring dashboards
- Feature rollout (e.g., adding `sustainability_score`) benefits all tenants

**Resource Efficiency**:
- Shared shard allocation
- Consistent merge policies
- One set of index lifecycle management (ILM) rules

---

## 2. Universal Commerce Schema (v1.0)

### 2.1 Core Schema Definition

```json
{
  "settings": {
    "index": {
      "number_of_shards": 12,
      "number_of_replicas": 1,
      "refresh_interval": "5s",
      "routing.allocation.include._tier_preference": "data_hot",
      "codec": "best_compression",

      // Tenant-based routing for efficiency
      "routing_partition_size": 1,

      // k-NN settings for vector search
      "knn": true,
      "knn.algo_param.ef_search": 100
    },

    "analysis": {
      "analyzer": {
        "commerce_standard": {
          "type": "standard",
          "stopwords": "_english_"
        },
        "commerce_keyword": {
          "type": "keyword",
          "lowercase": true
        }
      }
    }
  },

  "mappings": {
    "properties": {

      // ========================================
      // TENANT ISOLATION (CRITICAL)
      // ========================================
      "tenant_id": {
        "type": "keyword",
        "index": true,
        "doc_values": true
      },

      // ========================================
      // UNIVERSAL PRODUCT FIELDS
      // ========================================
      "product_id": {
        "type": "keyword",
        "doc_values": true
      },

      "sku": {
        "type": "keyword",
        "doc_values": true
      },

      "upc": {
        "type": "keyword",
        "index": false,
        "doc_values": false
      },

      "title": {
        "type": "text",
        "analyzer": "commerce_standard",
        "fields": {
          "keyword": {
            "type": "keyword",
            "ignore_above": 256
          },
          "ngram": {
            "type": "text",
            "analyzer": "edge_ngram_analyzer"
          }
        }
      },

      "description": {
        "type": "text",
        "analyzer": "commerce_standard"
      },

      "brand": {
        "type": "keyword",
        "fields": {
          "text": {
            "type": "text",
            "analyzer": "commerce_standard"
          }
        }
      },

      "manufacturer": {
        "type": "keyword"
      },

      "model_number": {
        "type": "keyword"
      },

      // ========================================
      // CATEGORY & TAXONOMY
      // ========================================
      "category": {
        "type": "keyword",
        "doc_values": true
      },

      "category_path": {
        "type": "keyword",
        "doc_values": true
      },

      "category_hierarchy": {
        "type": "object",
        "properties": {
          "level_1": { "type": "keyword" },  // "Electronics"
          "level_2": { "type": "keyword" },  // "Laptops"
          "level_3": { "type": "keyword" },  // "Gaming"
          "level_4": { "type": "keyword" }
        }
      },

      // ========================================
      // PRICING
      // ========================================
      "price": {
        "type": "scaled_float",
        "scaling_factor": 100
      },

      "currency": {
        "type": "keyword",
        "index": false
      },

      "sale_price": {
        "type": "scaled_float",
        "scaling_factor": 100
      },

      "on_sale": {
        "type": "boolean"
      },

      "discount_percentage": {
        "type": "half_float"
      },

      // ========================================
      // AVAILABILITY & INVENTORY
      // ========================================
      "in_stock": {
        "type": "boolean"
      },

      "inventory_count": {
        "type": "integer",
        "index": false
      },

      "availability_status": {
        "type": "keyword"  // in_stock, out_of_stock, backorder, discontinued
      },

      "shipping_weight": {
        "type": "scaled_float",
        "scaling_factor": 100,
        "index": false
      },

      // ========================================
      // SEMANTIC SEARCH (EMBEDDINGS)
      // ========================================
      "embedding": {
        "type": "knn_vector",
        "dimension": 768,
        "method": {
          "name": "hnsw",
          "space_type": "cosinesimil",
          "engine": "nmslib",
          "parameters": {
            "ef_construction": 128,
            "m": 16
          }
        }
      },

      "concepts": {
        "type": "keyword",
        "doc_values": true
      },

      // ========================================
      // CONTENT TYPE
      // ========================================
      "content_type": {
        "type": "keyword"
        // Values: product, guide, article, manual, faq, comparison, review, video
      },

      "content_subtype": {
        "type": "keyword"
        // Values: buying_guide, troubleshooting, how_to, comparison_table, etc.
      },

      // ========================================
      // CRAWLED CONTEXT
      // ========================================
      "crawled_context": {
        "type": "object",
        "properties": {
          "url": {
            "type": "keyword",
            "index": false
          },
          "canonical_url": {
            "type": "keyword"
          },
          "breadcrumbs": {
            "type": "keyword"
          },
          "related_urls": {
            "type": "keyword",
            "index": false
          },
          "depth": {
            "type": "integer"
          },
          "last_crawled": {
            "type": "date"
          },
          "crawl_source": {
            "type": "keyword"  // full_crawl, partial_crawl, event_driven
          }
        }
      },

      // ========================================
      // CATEGORY-SPECIFIC ATTRIBUTES (NESTED)
      // ========================================
      "attributes": {
        "type": "nested",
        "properties": {
          "key": {
            "type": "keyword"
          },
          "value": {
            "type": "keyword",
            "fields": {
              "text": {
                "type": "text",
                "analyzer": "commerce_standard"
              }
            }
          },
          "unit": {
            "type": "keyword"
          },
          "value_numeric": {
            "type": "float"
          }
        }
      },

      // ========================================
      // IMAGES & MEDIA
      // ========================================
      "images": {
        "type": "object",
        "enabled": false
      },

      "videos": {
        "type": "object",
        "enabled": false
      },

      // ========================================
      // RATINGS & REVIEWS (AGGREGATED)
      // ========================================
      "rating_avg": {
        "type": "half_float"
      },

      "rating_count": {
        "type": "integer"
      },

      "review_count": {
        "type": "integer"
      },

      // ========================================
      // POPULARITY & RANKING SIGNALS
      // ========================================
      "view_count": {
        "type": "integer",
        "index": false
      },

      "click_count": {
        "type": "integer",
        "index": false
      },

      "purchase_count": {
        "type": "integer",
        "index": false
      },

      "popularity_score": {
        "type": "half_float"
      },

      // ========================================
      // SEO & METADATA
      // ========================================
      "meta_title": {
        "type": "text",
        "index": false
      },

      "meta_description": {
        "type": "text",
        "index": false
      },

      "keywords": {
        "type": "keyword"
      },

      // ========================================
      // TENANT-SPECIFIC EXTENSIONS
      // ========================================
      "custom_fields": {
        "type": "object",
        "enabled": false
      },

      // ========================================
      // SYSTEM METADATA
      // ========================================
      "created_at": {
        "type": "date"
      },

      "updated_at": {
        "type": "date"
      },

      "indexed_at": {
        "type": "date"
      },

      "version": {
        "type": "integer",
        "index": false
      }
    }
  }
}
```

---

## 3. Field-by-Field Documentation

### 3.1 Tenant Isolation

| Field | Type | Purpose | Indexed | Notes |
|-------|------|---------|---------|-------|
| `tenant_id` | keyword | Tenant isolation boundary | ✅ Yes | **CRITICAL**: All queries MUST filter by this |

**Usage**:
```json
{
  "query": {
    "bool": {
      "filter": [{ "term": { "tenant_id": "electronic-express" }}],
      "must": [{ "match": { "title": "laptop" }}]
    }
  }
}
```

### 3.2 Product Identification

| Field | Type | Purpose | Indexed | Notes |
|-------|------|---------|---------|-------|
| `product_id` | keyword | Unique product identifier (tenant-scoped) | ✅ Yes | Format: `{tenant_id}:{sku}` or UUID |
| `sku` | keyword | Stock Keeping Unit | ✅ Yes | Tenant-specific SKU |
| `upc` | keyword | Universal Product Code | ❌ No | Stored for display only |

### 3.3 Content Fields

| Field | Type | Purpose | Indexed | Boost Weight |
|-------|------|---------|---------|--------------|
| `title` | text | Product/content title | ✅ Yes | 3.0 (highest) |
| `title.keyword` | keyword | Exact match variant | ✅ Yes | N/A |
| `title.ngram` | text | Autocomplete variant | ✅ Yes | N/A |
| `description` | text | Full description | ✅ Yes | 1.5 |
| `brand` | keyword | Brand name (facet) | ✅ Yes | 2.0 (navigational) |
| `brand.text` | text | Brand name (full-text) | ✅ Yes | 2.0 |

### 3.4 Category & Taxonomy

| Field | Type | Purpose | Example |
|-------|------|---------|---------|
| `category` | keyword | Primary category | "laptops" |
| `category_path` | keyword | Full path (array) | ["Electronics", "Laptops", "Gaming"] |
| `category_hierarchy.level_1` | keyword | Top-level category | "Electronics" |
| `category_hierarchy.level_2` | keyword | Second-level category | "Laptops" |

**Query Pattern** (drill-down):
```json
{
  "query": {
    "bool": {
      "filter": [
        { "term": { "tenant_id": "ee" }},
        { "term": { "category_hierarchy.level_1": "Electronics" }},
        { "term": { "category_hierarchy.level_2": "Laptops" }}
      ]
    }
  },
  "aggs": {
    "level_3_facets": {
      "terms": { "field": "category_hierarchy.level_3", "size": 50 }
    }
  }
}
```

### 3.5 Semantic Search

| Field | Type | Dimension | Purpose |
|-------|------|-----------|---------|
| `embedding` | knn_vector | 768 | MPNet multilingual embeddings |
| `concepts` | keyword | N/A | Extracted concepts (hiking, 4K, energy-efficient) |

**Hybrid Query** (BM25 + k-NN):
```json
{
  "query": {
    "bool": {
      "filter": [{ "term": { "tenant_id": "ee" }}],
      "should": [
        {
          "multi_match": {
            "query": "best laptops for students",
            "fields": ["title^3", "description^1.5", "brand^2"],
            "type": "best_fields"
          }
        },
        {
          "knn": {
            "embedding": {
              "vector": [0.23, -0.45, ...],  // 768-dim query embedding
              "k": 50
            }
          }
        }
      ],
      "minimum_should_match": 1
    }
  }
}
```

### 3.6 Category-Specific Attributes

**Nested Structure** (flexible key-value pairs):

```json
// Example: Laptop
{
  "attributes": [
    { "key": "screen_size", "value": "15.6", "unit": "inches", "value_numeric": 15.6 },
    { "key": "processor", "value": "Intel i7-13700H", "unit": null },
    { "key": "ram", "value": "16", "unit": "GB", "value_numeric": 16 },
    { "key": "storage", "value": "512", "unit": "GB SSD", "value_numeric": 512 }
  ]
}

// Example: T-Shirt
{
  "attributes": [
    { "key": "size", "value": "M", "unit": null },
    { "key": "color", "value": "Navy Blue", "unit": null },
    { "key": "material", "value": "100% Polyester", "unit": null }
  ]
}

// Example: TV
{
  "attributes": [
    { "key": "screen_size", "value": "65", "unit": "inches", "value_numeric": 65 },
    { "key": "resolution", "value": "4K", "unit": null },
    { "key": "display_technology", "value": "QLED", "unit": null },
    { "key": "refresh_rate", "value": "120", "unit": "Hz", "value_numeric": 120 }
  ]
}
```

**Query Pattern** (filter by attribute):
```json
{
  "query": {
    "bool": {
      "filter": [
        { "term": { "tenant_id": "ee" }},
        { "term": { "category": "laptops" }},
        {
          "nested": {
            "path": "attributes",
            "query": {
              "bool": {
                "must": [
                  { "term": { "attributes.key": "screen_size" }},
                  { "range": { "attributes.value_numeric": { "gte": 15, "lte": 16 }}}
                ]
              }
            }
          }
        }
      ]
    }
  }
}
```

---

## 4. Index Naming & Lifecycle

### 4.1 Index Naming Convention

```
commerce-products-v1       # Primary product index (current version)
commerce-content-v1        # Articles, guides, FAQs (current version)

commerce-products-v2       # Future version (schema evolution)
commerce-content-v2
```

**Alias Pattern**:
```json
POST /_aliases
{
  "actions": [
    { "add": { "index": "commerce-products-v1", "alias": "commerce-products" }},
    { "add": { "index": "commerce-content-v1", "alias": "commerce-content" }}
  ]
}
```

Queries target alias: `commerce-products`, enabling zero-downtime schema migrations.

### 4.2 Sharding Strategy

**Per-Index Shards**:
- `commerce-products-v1`: 12 shards (anticipating 10M+ documents across all tenants)
- `commerce-content-v1`: 6 shards (lower volume)

**Routing by Tenant**:
```json
POST /commerce-products/_doc?routing=electronic-express
{
  "tenant_id": "electronic-express",
  "product_id": "ee:laptop-001",
  "title": "Dell XPS 15"
}
```

Benefits:
- Queries for single tenant hit only relevant shards
- Tenant data co-located on same shard (faster retrieval)

### 4.3 Index Lifecycle Management (ILM)

**Hot → Warm → Cold → Delete**:

```json
{
  "policy": "commerce-products-lifecycle",
  "phases": {
    "hot": {
      "actions": {
        "rollover": {
          "max_size": "50GB",
          "max_age": "30d"
        }
      }
    },
    "warm": {
      "min_age": "30d",
      "actions": {
        "readonly": {},
        "forcemerge": { "max_num_segments": 1 }
      }
    },
    "cold": {
      "min_age": "90d",
      "actions": {
        "allocate": {
          "include": { "_tier_preference": "data_cold" }
        }
      }
    },
    "delete": {
      "min_age": "365d",
      "actions": {
        "delete": {}
      }
    }
  }
}
```

---

## 5. Schema Evolution & Versioning

### 5.1 Adding New Fields

**Backward-Compatible Additions**:

```json
// Add new field to v1 (existing index)
PUT /commerce-products-v1/_mapping
{
  "properties": {
    "sustainability_score": {
      "type": "half_float"
    },
    "carbon_footprint": {
      "type": "keyword"
    }
  }
}
```

**Non-Breaking**: Existing documents automatically get `null` for new fields.

### 5.2 Breaking Changes (New Version)

**When to create v2**:
- Changing field type (e.g., `price` from `float` to `scaled_float`)
- Removing required fields
- Changing analyzer settings

**Migration Process**:

```bash
# 1. Create v2 index with new schema
PUT /commerce-products-v2
{ "mappings": { ... new schema ... }}

# 2. Reindex from v1 to v2 (background)
POST /_reindex
{
  "source": { "index": "commerce-products-v1" },
  "dest": { "index": "commerce-products-v2" }
}

# 3. Switch alias (zero downtime)
POST /_aliases
{
  "actions": [
    { "remove": { "index": "commerce-products-v1", "alias": "commerce-products" }},
    { "add": { "index": "commerce-products-v2", "alias": "commerce-products" }}
  ]
}

# 4. Verify v2, then delete v1 (after 7 days)
DELETE /commerce-products-v1
```

### 5.3 Feature Flags for New Fields

**Tenant Opt-In**:

```scala
case class TenantFeatureFlags(
  tenantId: TenantId,
  enableSustainabilityScore: Boolean = false,
  enableVideoSearch: Boolean = false
)

object IndexWriter {
  def indexProduct(product: Product, flags: TenantFeatureFlags): Unit = {
    val doc = baseDocument(product)

    if (flags.enableSustainabilityScore) {
      doc.put("sustainability_score", product.sustainabilityScore)
    }

    if (flags.enableVideoSearch && product.videos.nonEmpty) {
      doc.put("videos", product.videos)
    }

    esClient.index(doc)
  }
}
```

---

## 6. Query Patterns & Best Practices

### 6.1 Tenant-Isolated Search

**ALWAYS include tenant filter**:

```json
{
  "query": {
    "bool": {
      "filter": [
        { "term": { "tenant_id": "electronic-express" }}
      ],
      "must": [
        { "match": { "title": "laptop" }}
      ]
    }
  }
}
```

**Compile-Time Safety** (Scala):

```scala
trait TenantIsolatedQuery {
  def tenantId: TenantId

  def toOpenSearchQuery: OpenSearchQuery = {
    OpenSearchQuery(
      bool = BoolQuery(
        filter = List(
          TermQuery("tenant_id", tenantId.value)  // Enforced at compile time
        ),
        must = buildMustClauses()
      )
    )
  }

  protected def buildMustClauses(): List[Query]
}
```

### 6.2 Hybrid Search (BM25 + k-NN)

**Weighted Fusion**:

```json
{
  "query": {
    "bool": {
      "filter": [{ "term": { "tenant_id": "ee" }}],
      "should": [
        {
          "multi_match": {
            "query": "4K gaming laptop",
            "fields": ["title^3", "description^1.5", "brand^2"],
            "type": "best_fields",
            "boost": 0.6
          }
        },
        {
          "script_score": {
            "query": { "match_all": {} },
            "script": {
              "source": "cosineSimilarity(params.query_vector, 'embedding') + 1.0",
              "params": {
                "query_vector": [0.23, -0.45, ...]
              }
            },
            "boost": 0.4
          }
        }
      ]
    }
  }
}
```

### 6.3 Faceted Search

**Multi-Level Faceting**:

```json
{
  "query": {
    "bool": {
      "filter": [
        { "term": { "tenant_id": "ee" }},
        { "term": { "category_hierarchy.level_1": "Electronics" }}
      ]
    }
  },
  "aggs": {
    "brands": {
      "terms": { "field": "brand", "size": 20 }
    },
    "price_ranges": {
      "range": {
        "field": "price",
        "ranges": [
          { "to": 500 },
          { "from": 500, "to": 1000 },
          { "from": 1000, "to": 2000 },
          { "from": 2000 }
        ]
      }
    },
    "attributes": {
      "nested": { "path": "attributes" },
      "aggs": {
        "screen_sizes": {
          "filter": { "term": { "attributes.key": "screen_size" }},
          "aggs": {
            "values": { "terms": { "field": "attributes.value", "size": 10 }}
          }
        }
      }
    }
  }
}
```

---

## 7. Performance Optimization

### 7.1 Index Settings

```json
{
  "settings": {
    "index": {
      "refresh_interval": "5s",           // Balance: near-real-time vs write throughput
      "number_of_replicas": 1,            // HA without excessive storage
      "codec": "best_compression",        // Reduce storage by ~30% (slight CPU cost)
      "max_result_window": 10000,         // Prevent deep pagination abuse
      "max_inner_result_window": 100,     // Limit nested query results

      // Merge policy (reduce segment count)
      "merge.policy.max_merged_segment": "5gb",
      "merge.policy.segments_per_tier": 10,

      // Translog settings
      "translog.durability": "async",     // Higher throughput, slight durability risk
      "translog.sync_interval": "5s"
    }
  }
}
```

### 7.2 Query Optimization

**Filter vs Must**:
- Use `filter` for exact matches (cached, no scoring): `tenant_id`, `category`, `in_stock`
- Use `must` for full-text search (scored): `title`, `description`

**Field Data**:
- Enable `doc_values` for sorting/aggregation fields
- Disable for full-text fields (saves heap)

**Source Filtering**:
```json
{
  "query": { ... },
  "_source": ["product_id", "title", "price", "brand"]
}
```

### 7.3 Caching Strategy

**Request Cache** (query results):
- Enabled by default for `size=0` (aggregation-only queries)
- Use `?request_cache=true` for frequently repeated queries

**Field Data Cache** (doc values):
- Automatically managed by OpenSearch
- Monitor: `GET /_nodes/stats/indices/fielddata`

---

## 8. Monitoring & Observability

### 8.1 Key Metrics

```yaml
metrics:
  - name: index_size_bytes
    type: gauge
    labels: [index_name]
    description: "Total size of index on disk"

  - name: document_count
    type: gauge
    labels: [index_name, tenant_id]
    description: "Number of documents per tenant"

  - name: search_latency_ms
    type: histogram
    labels: [index_name, query_type]
    buckets: [10, 50, 100, 250, 500, 1000]

  - name: indexing_rate
    type: counter
    labels: [index_name, tenant_id]
    description: "Documents indexed per second"

  - name: refresh_time_ms
    type: histogram
    labels: [index_name]

  - name: merge_time_ms
    type: histogram
    labels: [index_name]
```

### 8.2 Alerts

```yaml
alerts:
  - name: HighSearchLatency
    condition: search_latency_ms{quantile="0.95"} > 500
    for: 5m
    action: page on-call

  - name: IndexSizeTooLarge
    condition: index_size_bytes > 100GB
    for: 1h
    action: email platform-team, consider rollover

  - name: LowIndexingRate
    condition: rate(indexing_rate[5m]) < 10
      AND tenant_has_event_integration == true
    for: 15m
    action: email tenant contact, check event source
```

---

## 9. Security & Access Control

### 9.1 Index-Level Permissions

**OpenSearch Security Plugin**:

```yaml
roles:
  tenant_read_only:
    index_permissions:
      - index_patterns: ["commerce-products", "commerce-content"]
        allowed_actions: ["indices:data/read/search", "indices:data/read/get"]
        dls: '{"term": {"tenant_id": "${attr.tenant_id}"}}'  # Document-level security

  tenant_read_write:
    index_permissions:
      - index_patterns: ["commerce-products", "commerce-content"]
        allowed_actions: ["indices:data/write/index", "indices:data/write/update"]
        dls: '{"term": {"tenant_id": "${attr.tenant_id}"}}'

  platform_admin:
    index_permissions:
      - index_patterns: ["commerce-*"]
        allowed_actions: ["indices:*"]
```

### 9.2 Document-Level Security (DLS)

**Automatic tenant filtering** at OpenSearch layer:

```json
// User JWT contains: { "tenant_id": "electronic-express" }

// Query submitted by user:
{
  "query": { "match": { "title": "laptop" }}
}

// OpenSearch rewrites with DLS:
{
  "query": {
    "bool": {
      "must": [{ "match": { "title": "laptop" }}],
      "filter": [{ "term": { "tenant_id": "electronic-express" }}]
    }
  }
}
```

---

## 10. Testing Strategy

### 10.1 Schema Validation Tests

```scala
class SchemaValidationSpec extends AnyFlatSpec {

  test("index template should enforce tenant_id field") {
    val template = OpenSearchClient.getIndexTemplate("commerce-products")

    val tenantIdMapping = template.mappings.properties("tenant_id")
    tenantIdMapping.`type` shouldBe "keyword"
    tenantIdMapping.index shouldBe true
  }

  test("embedding field should have correct dimension") {
    val template = OpenSearchClient.getIndexTemplate("commerce-products")

    val embeddingMapping = template.mappings.properties("embedding")
    embeddingMapping.`type` shouldBe "knn_vector"
    embeddingMapping.dimension shouldBe 768
  }
}
```

### 10.2 Query Performance Tests

```scala
class QueryPerformanceSpec extends AnyFlatSpec {

  test("tenant-filtered search should complete in <100ms at p95") {
    val results = (1 to 100).map { _ =>
      val start = System.currentTimeMillis()

      searchClient.search(
        index = "commerce-products",
        query = TenantIsolatedQuery(
          tenantId = TenantId("ee"),
          queryText = "laptop"
        )
      )

      System.currentTimeMillis() - start
    }

    val p95 = results.sorted.apply(95)
    p95 should be < 100L
  }
}
```

---

## 11. Migration from Per-Tenant Indexes

### 11.1 Pre-Migration Checklist

- [ ] Backup all tenant indexes
- [ ] Create universal schema index (v1)
- [ ] Test reindex script on 1 tenant (pilot)
- [ ] Verify query performance (compare old vs new)
- [ ] Update application code to use tenant_id filter
- [ ] Update monitoring dashboards

### 11.2 Migration Script

```scala
object TenantIndexMigration {

  def migrateTenantsToUniversalSchema(
    tenants: List[TenantId],
    sourceIndexPattern: String = "commerce-products-{tenant}",
    destIndex: String = "commerce-products-v1"
  ): Unit = {

    tenants.foreach { tenant =>
      val sourceIndex = sourceIndexPattern.replace("{tenant}", tenant.value)

      println(s"Migrating $tenant from $sourceIndex to $destIndex")

      esClient.reindex(
        source = sourceIndex,
        dest = destIndex,
        script = s"""
          ctx._source.tenant_id = '${tenant.value}';
          ctx._source.product_id = '${tenant.value}:' + ctx._source.product_id;
        """
      )

      // Verify migration
      val sourceCount = esClient.count(sourceIndex)
      val destCount = esClient.count(destIndex, filter = TermQuery("tenant_id", tenant.value))

      require(sourceCount == destCount, s"Migration failed for $tenant: $sourceCount != $destCount")

      println(s"✓ Migrated $sourceCount documents for $tenant")
    }
  }
}
```

---

## 12. Best Practices & Anti-Patterns

### 12.1 ✅ Best Practices

1. **Always Filter by Tenant**: Every query MUST include `tenant_id` filter (enforce at compile time)
2. **Use Routing**: Specify `?routing={tenant_id}` for single-tenant queries (faster)
3. **Leverage Aliases**: Query via alias (`commerce-products`) not versioned index
4. **Monitor Per-Tenant**: Track document count, query latency, indexing rate per tenant
5. **Use Doc Values**: Enable for all fields used in sorting/aggregation/scripting
6. **Batch Writes**: Use bulk API with 500-1000 docs per batch

### 12.2 ❌ Anti-Patterns

1. **Querying Without Tenant Filter**: Security risk + performance disaster
2. **Deep Pagination**: Don't use `from: 10000` (use search_after instead)
3. **Wildcard Queries on Analyzed Fields**: `title: laptop*` is slow (use ngram field)
4. **Storing Large Blobs**: Don't put full images in `images` field (use URLs)
5. **Too Many Shards**: Don't create per-tenant shards (defeats universal schema purpose)
6. **Modifying Schema Per-Tenant**: If you need tenant-specific fields, use `custom_fields` (stored only)

---

## 13. Future Enhancements

### 13.1 Roadmap

**v1.1** (Q2 2026):
- Add `sustainability_score`, `carbon_footprint` fields
- Add `video_urls` field with video metadata
- Support for AR/3D model references

**v2.0** (Q3 2026):
- Upgrade to OpenSearch 3.x
- Switch to Lucene 10 codec
- Add multi-language support (separate `title_es`, `description_es` fields)

### 13.2 Research Areas

- **Learned Sparse Retrieval**: Explore SPLADE, SPLADEv2 for better sparse vectors
- **Cross-Modal Search**: Image → text search using CLIP embeddings
- **Query Understanding**: Use LLMs to rewrite queries before search

---

## 14. References

- [OpenSearch Mapping Documentation](https://opensearch.org/docs/latest/opensearch/mappings/)
- [k-NN Plugin](https://opensearch.org/docs/latest/search-plugins/knn/)
- [Index Lifecycle Management](https://opensearch.org/docs/latest/im-plugin/ism/)
- [CHARTER.md](../../CHARTER.md): Section 2.8 Data Models
- [Multi-Tenant-Search-Platform-Architecture.md](Multi-Tenant-Search-Platform-Architecture.md): Tenant isolation patterns

---

## 15. Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-01-04 | Author | Initial universal commerce schema definition |

---

**Status**: ✅ **Active** - This SBPF is the authoritative reference for the universal commerce index schema.
