# OpenSearch and Vector Search Architecture for Commerce

**Version**: 1.0.0
**Last Updated**: 2026-01-04
**Audience**: Architects, DevOps, Developers

---

## Purpose

This document provides architectural guidance for deploying and configuring OpenSearch with k-NN (k-Nearest Neighbors) vector search capabilities to support semantic commerce search. It covers index design, query patterns, performance optimization, and hybrid retrieval strategies that combine sparse (BM25) and dense (vector) search.

---

## Why OpenSearch for Commerce Search?

| Capability | Benefit for Commerce |
|------------|----------------------|
| **BM25 Lexical Search** | Handles exact matches, product codes, brand names |
| **k-NN Vector Search** | Enables semantic similarity, cross-language retrieval |
| **Hybrid Search** | Combines precision of lexical with recall of semantic |
| **Attribute Filtering** | Pre-filter by category, price, brand before vector search |
| **Open Source** | No licensing costs; community-driven; AWS-independent |
| **Mature Ecosystem** | Battle-tested at scale (derived from Elasticsearch 7.10) |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      OPENSEARCH CLUSTER                           │
└─────────────────────────────────────────────────────────────────┘

┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│   Ingest     │      │   Storage    │      │   Search     │
│   Pipeline   │─────▶│   Indices    │─────▶│   API        │
│              │      │              │      │              │
│ • Transform  │      │ • Shards     │      │ • Query DSL  │
│ • Embed      │      │ • Replicas   │      │ • Aggregat   │
│ • Enrich     │      │ • k-NN index │      │ • Scoring    │
└──────────────┘      └──────────────┘      └──────────────┘


┌─────────────────────────────────────────────────────────────────┐
│                      INDEX STRUCTURE                              │
└─────────────────────────────────────────────────────────────────┘

commerce_items_v1  (Alias: commerce_items)
│
├─ Mappings
│  ├─ id: keyword
│  ├─ type: keyword (product | guide | article | manual)
│  ├─ title: text (BM25 analyzed)
│  ├─ description: text (BM25 analyzed)
│  ├─ embedding: knn_vector[768] (dense semantic)
│  ├─ category: keyword (filter)
│  ├─ brand: keyword (filter)
│  ├─ attributes: nested (facets)
│  └─ crawled_context: object (breadcrumbs, depth)
│
├─ Settings
│  ├─ shards: 3
│  ├─ replicas: 1
│  └─ knn: true (enable k-NN plugin)
│
└─ k-NN Configuration
   ├─ algorithm: HNSW (Hierarchical Navigable Small World)
   ├─ space_type: cosinesimil (cosine similarity)
   ├─ engine: nmslib (fast approximate NN)
   ├─ ef_construction: 512 (build-time accuracy)
   └─ m: 16 (edges per node in graph)
```

---

## Index Design

### Mapping Schema (JSON)

```json
{
  "settings": {
    "number_of_shards": 3,
    "number_of_replicas": 1,
    "index.knn": true,
    "analysis": {
      "analyzer": {
        "commerce_analyzer": {
          "type": "custom",
          "tokenizer": "standard",
          "filter": ["lowercase", "stop", "synonym"]
        }
      },
      "filter": {
        "synonym": {
          "type": "synonym",
          "synonyms": [
            "sneakers, trainers, running shoes, athletic shoes",
            "laptop, notebook, portable computer",
            "tv, television, display"
          ]
        }
      }
    }
  },
  "mappings": {
    "properties": {
      "id": {
        "type": "keyword"
      },
      "type": {
        "type": "keyword"
      },
      "title": {
        "type": "text",
        "analyzer": "commerce_analyzer",
        "fields": {
          "raw": {
            "type": "keyword"
          }
        }
      },
      "description": {
        "type": "text",
        "analyzer": "commerce_analyzer"
      },
      "category": {
        "type": "keyword"
      },
      "brand": {
        "type": "keyword"
      },
      "price": {
        "type": "float"
      },
      "embedding": {
        "type": "knn_vector",
        "dimension": 768,
        "method": {
          "name": "hnsw",
          "space_type": "cosinesimil",
          "engine": "nmslib",
          "parameters": {
            "ef_construction": 512,
            "m": 16
          }
        }
      },
      "concepts": {
        "type": "keyword"
      },
      "attributes": {
        "type": "nested",
        "properties": {
          "key": {
            "type": "keyword"
          },
          "value": {
            "type": "keyword"
          }
        }
      },
      "crawled_context": {
        "type": "object",
        "properties": {
          "breadcrumbs": {
            "type": "keyword"
          },
          "depth": {
            "type": "integer"
          },
          "related_links": {
            "type": "keyword"
          }
        }
      },
      "url": {
        "type": "keyword"
      },
      "created_at": {
        "type": "date"
      }
    }
  }
}
```

---

## k-NN Configuration Deep Dive

### HNSW Algorithm Parameters

| Parameter | Recommended | Trade-off | Tuning Guidance |
|-----------|-------------|-----------|-----------------|
| **ef_construction** | 512 | Build time vs accuracy | Higher = more accurate index, slower indexing (100-1000) |
| **m** | 16 | Memory vs accuracy | Higher = better accuracy, more memory (5-50) |
| **ef_search** | 256 | Query latency vs recall | Higher = better recall, slower queries (100-500) |

**Formula for memory estimation**:
```
Memory (GB) ≈ (num_vectors × dimension × 4 bytes × m) / (1024^3)

Example:
1M vectors × 768 dim × 4 bytes × 16 / (1024^3) ≈ 46 GB
```

### Space Types

| Space Type | Formula | Use Case |
|------------|---------|----------|
| **cosinesimil** | `1 - (A·B / |A||B|)` | Semantic similarity (text embeddings) |
| **l2** | `sqrt(Σ(A-B)²)` | Euclidean distance (image embeddings) |
| **inner_product** | `A·B` | When vectors are pre-normalized |

**Recommendation for commerce**: Use `cosinesimil` for text embeddings (MPNet, BERT).

---

## Hybrid Search: Combining BM25 and k-NN

### Why Hybrid?

| Search Type | Strengths | Weaknesses |
|-------------|-----------|------------|
| **BM25 (Sparse)** | Exact matches, brand names, SKUs | Misses synonyms, cross-language |
| **k-NN (Dense)** | Semantic similarity, concepts | Can miss exact matches |
| **Hybrid** | Best of both worlds | Requires score fusion |

### Query Pattern

```json
{
  "query": {
    "bool": {
      "should": [
        {
          "multi_match": {
            "query": "best laptops for students",
            "fields": ["title^2", "description"],
            "type": "best_fields",
            "boost": 1.0
          }
        },
        {
          "knn": {
            "embedding": {
              "vector": [0.123, 0.456, ..., 0.789],
              "k": 100,
              "boost": 2.0
            }
          }
        }
      ],
      "filter": [
        {
          "term": { "category": "laptops" }
        },
        {
          "range": {
            "price": { "lte": 800 }
          }
        }
      ],
      "minimum_should_match": 1
    }
  },
  "size": 20,
  "_source": ["id", "title", "type", "price", "url"]
}
```

### Score Fusion Strategies

**1. Linear Combination** (default above)
```
final_score = (bm25_score × 1.0) + (knn_score × 2.0)
```

**2. Rank Fusion (Reciprocal Rank Fusion - RRF)**
```scala
def reciprocalRankFusion(bm25Results: List[Result], knnResults: List[Result], k: Int = 60): List[Result] = {
  val bm25Scores = bm25Results.zipWithIndex.map { case (result, rank) => result.id -> (1.0 / (k + rank + 1)) }.toMap
  val knnScores = knnResults.zipWithIndex.map { case (result, rank) => result.id -> (1.0 / (k + rank + 1)) }.toMap

  val allIds = (bm25Scores.keys ++ knnScores.keys).toSet
  val fused = allIds.map { id =>
    id -> (bm25Scores.getOrElse(id, 0.0) + knnScores.getOrElse(id, 0.0))
  }.toList.sortBy(-_._2)

  fused.map { case (id, score) => Result(id, score) }
}
```

**3. Learned Fusion (LambdaMART, RankNet)**
- Train ML model on click-through data to learn optimal weighting
- Out of scope for POC; use fixed weights initially

---

## Pre-Filtering for Performance

**Problem**: k-NN over 1M+ vectors is slow.

**Solution**: Apply **filters before k-NN** using OpenSearch's filter context.

### Efficient Filter + k-NN Pattern

```json
{
  "query": {
    "bool": {
      "must": [
        {
          "knn": {
            "embedding": {
              "vector": [...],
              "k": 50
            }
          }
        }
      ],
      "filter": [
        {
          "term": { "type": "product" }
        },
        {
          "terms": { "category": ["laptops", "notebooks"] }
        },
        {
          "range": { "price": { "gte": 500, "lte": 1500 } }
        }
      ]
    }
  }
}
```

**Performance impact**: Reduces k-NN search space from 1M vectors to ~50K (10x speedup).

---

## Index Alias Pattern (Zero-Downtime Updates)

**Problem**: Re-indexing with new embeddings requires downtime.

**Solution**: Use **aliases** for atomic switchover.

### Workflow

```bash
# 1. Create new index with improved embeddings
PUT /commerce_items_v2
{ "mappings": {...}, "settings": {...} }

# 2. Bulk index new data
POST /_bulk
{ "index": { "_index": "commerce_items_v2", "_id": "1" } }
{ "title": "...", "embedding": [...] }

# 3. Test new index
GET /commerce_items_v2/_search
{ "query": {...} }

# 4. Atomic switchover
POST /_aliases
{
  "actions": [
    { "remove": { "index": "commerce_items_v1", "alias": "commerce_items" } },
    { "add": { "index": "commerce_items_v2", "alias": "commerce_items" } }
  ]
}

# 5. Delete old index (after validation)
DELETE /commerce_items_v1
```

**Application code**: Always query `commerce_items` (alias), never versioned index.

---

## Performance Optimization

### 1. Shard Sizing

**Rule of thumb**: Each shard should be 20-50 GB.

```
num_shards = total_index_size_GB / 30
```

Example:
- 100 GB index → 3-4 shards
- 1 TB index → 30-50 shards

**Over-sharding**: Too many shards = excessive overhead.
**Under-sharding**: Too few shards = poor parallelism.

### 2. Replica Tuning

| Replicas | Read Throughput | Write Throughput | Failure Tolerance |
|----------|-----------------|------------------|-------------------|
| 0 | Baseline | Fastest | None (data loss risk) |
| 1 | 2x | Moderate | 1 node failure |
| 2 | 3x | Slow | 2 node failures |

**Recommendation**: Start with 1 replica; increase for high-traffic production.

### 3. Force Merge (Post-Indexing)

After bulk indexing completes, merge segments to optimize read performance.

```bash
POST /commerce_items/_forcemerge?max_num_segments=1
```

**Warning**: Force merge is expensive; run during off-peak hours.

### 4. k-NN Query Caching

Enable request cache for repeated queries:

```bash
GET /commerce_items/_search?request_cache=true
{
  "query": { "knn": {...} }
}
```

**Cache invalidation**: Automatic when index is updated.

---

## Observability and Monitoring

### Key Metrics (Prometheus + OpenSearch Exporter)

| Metric | Alert Threshold | Action |
|--------|-----------------|--------|
| **Query Latency (P95)** | > 500ms | Optimize queries, add replicas, or scale cluster |
| **Indexing Rate** | < 1000 docs/sec | Check CPU, disk I/O, or increase shards |
| **JVM Heap Usage** | > 75% | Increase heap size or add nodes |
| **Disk Usage** | > 85% | Delete old indices or add storage |
| **k-NN Recall@100** | < 0.90 | Increase `ef_search` or `ef_construction` |

### OpenSearch Performance Analyzer

Enable built-in performance analyzer:

```bash
curl -X POST "localhost:9200/_plugins/_performanceanalyzer/cluster/config" -H 'Content-Type: application/json' -d'
{
  "enabled": true
}
'
```

View metrics:
```bash
curl "localhost:9600/_plugins/_performanceanalyzer/metrics?metrics=CPU_Utilization,IO_TotThroughput&agg=avg"
```

---

## Disaster Recovery and Backups

### Snapshot to S3 (Recommended)

```bash
# Register snapshot repository
PUT /_snapshot/commerce_backups
{
  "type": "s3",
  "settings": {
    "bucket": "my-opensearch-backups",
    "region": "us-east-1",
    "base_path": "snapshots"
  }
}

# Create snapshot
PUT /_snapshot/commerce_backups/snapshot_2026-01-04
{
  "indices": "commerce_items_*",
  "ignore_unavailable": true,
  "include_global_state": false
}

# Restore snapshot
POST /_snapshot/commerce_backups/snapshot_2026-01-04/_restore
{
  "indices": "commerce_items_v1",
  "rename_pattern": "(.+)",
  "rename_replacement": "restored_$1"
}
```

**Frequency**: Daily snapshots; retain for 30 days.

---

## Security Best Practices

### 1. Enable Security Plugin

OpenSearch includes free security plugin (TLS, RBAC, audit logs).

```yaml
# opensearch.yml
plugins.security.ssl.http.enabled: true
plugins.security.ssl.transport.enabled: true
plugins.security.authcz.admin_dn:
  - "CN=admin,OU=ops,O=company,C=US"
```

### 2. Role-Based Access Control (RBAC)

```json
{
  "read_only_user": {
    "cluster_permissions": [],
    "index_permissions": [
      {
        "index_patterns": ["commerce_items*"],
        "allowed_actions": ["indices:data/read/*"]
      }
    ]
  }
}
```

### 3. API Key Authentication

Prefer API keys over basic auth for applications:

```bash
POST /_security/api_key
{
  "name": "commerce_search_app",
  "role_descriptors": {
    "commerce_search_role": {
      "cluster": ["cluster:monitor/main"],
      "index": [
        {
          "names": ["commerce_items"],
          "privileges": ["read"]
        }
      ]
    }
  }
}
```

---

## Docker Deployment (Development)

See [docker-compose.yml](../../../docker-compose.yml) for full config.

**Quick start**:
```yaml
version: '3.8'
services:
  opensearch:
    image: opensearchproject/opensearch:2.11.0
    environment:
      - discovery.type=single-node
      - OPENSEARCH_JAVA_OPTS=-Xms2g -Xmx2g
      - plugins.security.disabled=true  # Disable for local dev
    ports:
      - "9200:9200"
      - "9600:9600"
    volumes:
      - opensearch_data:/usr/share/opensearch/data

volumes:
  opensearch_data:
```

---

## Testing Strategies

### 1. Unit Tests (Index Mapping Validation)

```scala
test("index mapping should include knn_vector field") {
  val mapping = IndexMapping.load("commerce_items")
  mapping.fields should contain key "embedding"
  mapping.fields("embedding").`type` shouldEqual "knn_vector"
  mapping.fields("embedding").dimension shouldEqual 768
}
```

### 2. Integration Tests (Query Correctness)

```scala
test("hybrid search should return both BM25 and k-NN results") {
  val query = HybridQuery(
    text = "gaming laptop",
    embedding = EmbeddingGenerator.generate("gaming laptop")
  )

  val results = OpenSearchClient.search("commerce_items", query)

  results.hits should not be empty
  results.hits.exists(_.matchedBy == "bm25") shouldBe true
  results.hits.exists(_.matchedBy == "knn") shouldBe true
}
```

### 3. Performance Tests (Latency Benchmark)

```scala
test("P95 latency should be under 500ms for k-NN queries") {
  val queries = TestDataGenerator.randomQueries(1000)

  val latencies = queries.map { query =>
    val start = System.nanoTime()
    OpenSearchClient.search("commerce_items", query)
    (System.nanoTime() - start) / 1e6  // Convert to ms
  }.sorted

  val p95 = latencies((latencies.length * 0.95).toInt)
  p95 should be < 500.0
}
```

---

## Anti-Patterns

### ❌ Using k-NN Without Pre-Filtering
**Problem**: Scanning 1M+ vectors is slow.
**Solution**: Apply filters (category, price) before k-NN.

### ❌ Over-Sharding Small Indices
**Problem**: 10 shards for 1 GB index = excessive overhead.
**Solution**: Use 1-2 shards for indices < 10 GB.

### ❌ Ignoring Force Merge After Bulk Indexing
**Problem**: Many small segments = slow reads.
**Solution**: Run `_forcemerge` post-indexing.

### ❌ Storing Embeddings as Arrays of Floats (Without k-NN)
**Problem**: Cannot use HNSW acceleration.
**Solution**: Use `knn_vector` type with proper configuration.

### ❌ Not Using Aliases for Index Management
**Problem**: Re-indexing causes downtime.
**Solution**: Always use aliases; atomic switchover.

---

## Related SBPF Documents

- **[Web-Crawling-Best-Practices.md](Web-Crawling-Best-Practices.md)**: How to build the corpus for indexing
- **[Semantic-Search-Intent-Modeling.md](Semantic-Search-Intent-Modeling.md)**: Query preprocessing and intent classification
- **[Embedding-Strategies-Commerce.md](Embedding-Strategies-Commerce.md)**: Generating embeddings for the `embedding` field

---

## References

- **OpenSearch k-NN Plugin**: [opensearch.org/docs/latest/search-plugins/knn/](https://opensearch.org/docs/latest/search-plugins/knn/)
- **HNSW Paper**: Malkov & Yashunin (2018) - [arXiv:1603.09320](https://arxiv.org/abs/1603.09320)
- **Reciprocal Rank Fusion**: Cormack et al. (2009) - ACM SIGIR
- **OpenSearch Best Practices**: [AWS OpenSearch Service Best Practices](https://docs.aws.amazon.com/opensearch-service/latest/developerguide/bp.html)

---

**Summary**: OpenSearch with k-NN provides production-ready semantic search. Use hybrid queries (BM25 + k-NN), pre-filter to optimize performance, and leverage aliases for zero-downtime re-indexing. Monitor P95 latency, heap usage, and recall metrics. Test with integration and performance suites.
