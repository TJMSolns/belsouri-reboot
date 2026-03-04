# Embedding Strategies for Commerce Search

**Version**: 1.0.0
**Last Updated**: 2026-01-04
**Audience**: ML Engineers, Architects, Developers

---

## Purpose

This document provides guidance on selecting, generating, and optimizing embeddings (dense vector representations) for semantic commerce search. It covers pre-trained model selection, fine-tuning strategies, multilingual support, and engineering best practices for production deployment.

---

## What Are Embeddings?

**Embeddings** are fixed-length vector representations of text (or images) in a high-dimensional space where semantically similar items are close together.

**Example**:
```
"running shoes"      → [0.23, -0.45, 0.67, ..., 0.12]  (768 dimensions)
"athletic footwear"  → [0.21, -0.43, 0.65, ..., 0.14]  (cosine similarity ≈ 0.95)
"laptop computer"    → [0.87, 0.12, -0.34, ..., 0.56]  (cosine similarity ≈ 0.12)
```

**Why embeddings for commerce?**
- **Semantic similarity**: "sneakers" ≈ "trainers" ≈ "running shoes" (same vector neighborhood)
- **Cross-language**: "zapatillas" (Spanish) ≈ "shoes" (English) with multilingual models
- **Concept matching**: "energy efficient washer" finds products even without exact phrase match

---

## Pre-Trained Model Selection

### Recommended Models for Commerce (2026)

| Model | Dimensions | Multilingual | Speed | Use Case |
|-------|------------|--------------|-------|----------|
| **multi-qa-mpnet-base-dot-v1** | 768 | ✅ | Fast | General-purpose semantic search |
| **BAAI/bge-large-en-v1.5** | 1024 | ❌ (English only) | Medium | State-of-the-art English retrieval |
| **intfloat/e5-large-v2** | 1024 | ✅ | Medium | Strong zero-shot cross-language |
| **sentence-transformers/all-MiniLM-L6-v2** | 384 | ❌ | Very Fast | Low-latency applications (mobile) |
| **BAAI/bge-m3** | 1024 | ✅ (100+ languages) | Slow | Extreme multilingual support |

### Selection Criteria

| Criterion | Recommendation | Reasoning |
|-----------|---------------|-----------|
| **Production deployment** | MPNet (768-dim) | Proven, fast, multilingual, good recall |
| **English-only, max accuracy** | BGE-large (1024-dim) | SOTA on BEIR benchmark |
| **Extreme multilingual (50+ languages)** | BGE-m3 or E5 | Zero-shot cross-language retrieval |
| **Low latency (<50ms)** | MiniLM (384-dim) | Sacrifice accuracy for speed |
| **Fine-tuning planned** | E5 or MPNet | Good transfer learning base |

**Recommendation for POC**: Start with **multi-qa-mpnet-base-dot-v1** (proven for commerce, multilingual, fast).

---

## Embedding Generation Pipeline

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     EMBEDDING GENERATION                          │
└─────────────────────────────────────────────────────────────────┘

Input Text (Product/Query)
    │
    ▼
┌──────────────┐
│ Preprocessing│ → Normalize, clean, construct prompt
└──────────────┘
    │
    ▼
┌──────────────┐
│ Tokenization │ → Convert text to token IDs (BERT-style)
└──────────────┘
    │
    ▼
┌──────────────┐
│ Model Forward│ → Transformer encoder (12-24 layers)
│     Pass     │
└──────────────┘
    │
    ▼
┌──────────────┐
│   Pooling    │ → Mean pooling or CLS token
└──────────────┘
    │
    ▼
┌──────────────┐
│ Normalization│ → L2-normalize for cosine similarity
└──────────────┘
    │
    ▼
Embedding Vector [768 or 1024 floats]
```

### Implementation (Python + Sentence Transformers)

```python
from sentence_transformers import SentenceTransformer
import numpy as np

class EmbeddingGenerator:
    def __init__(self, model_name: str = "multi-qa-mpnet-base-dot-v1"):
        self.model = SentenceTransformer(model_name)

    def generate(self, text: str) -> np.ndarray:
        """Generate L2-normalized embedding for input text."""
        embedding = self.model.encode(
            text,
            convert_to_numpy=True,
            normalize_embeddings=True  # L2-normalize for cosine similarity
        )
        return embedding

    def generate_batch(self, texts: List[str], batch_size: int = 32) -> np.ndarray:
        """Generate embeddings for multiple texts efficiently."""
        embeddings = self.model.encode(
            texts,
            batch_size=batch_size,
            convert_to_numpy=True,
            normalize_embeddings=True,
            show_progress_bar=True
        )
        return embeddings

# Example usage
generator = EmbeddingGenerator()

# Single query
query_embedding = generator.generate("best laptops for students")
# Result: [0.123, -0.456, ..., 0.789]  (768 dimensions)

# Batch processing (efficient for indexing)
product_titles = [
    "Dell XPS 15 Laptop",
    "MacBook Pro 16-inch",
    "Lenovo ThinkPad X1 Carbon"
]
product_embeddings = generator.generate_batch(product_titles)
# Result: (3, 768) numpy array
```

---

## Text Preprocessing for Embeddings

### Best Practices

**Do**:
- ✅ Normalize whitespace (`"Best  Laptop"` → `"Best Laptop"`)
- ✅ Remove HTML tags (`"<p>Laptop</p>"` → `"Laptop"`)
- ✅ Truncate to model's max length (512 tokens for BERT-style models)
- ✅ Construct informative prompts (see below)

**Don't**:
- ❌ Remove stopwords (transformers use them for context)
- ❌ Lowercase everything (models are case-sensitive; "IPhone" ≠ "iphone")
- ❌ Stem/lemmatize (transformers handle morphology)
- ❌ Remove punctuation (can convey meaning)

### Prompt Construction for Products

**Problem**: Raw product titles lack context: "XPS 15" doesn't indicate it's a laptop.

**Solution**: Construct rich prompts that include category, brand, attributes.

```python
def construct_product_prompt(product: Product) -> str:
    """
    Construct embedding-friendly prompt for product.

    Example:
    Input: Product(title="XPS 15", category="Laptops", brand="Dell", price=1299)
    Output: "Dell XPS 15. Category: Laptops. A laptop computer for professionals and students."
    """
    parts = []

    # Brand + title (most important)
    if product.brand:
        parts.append(f"{product.brand} {product.title}")
    else:
        parts.append(product.title)

    # Category context
    if product.category:
        parts.append(f"Category: {product.category}")

    # Description (first 200 chars)
    if product.description:
        desc = product.description[:200].strip()
        parts.append(desc)

    # Key attributes
    if product.attributes:
        attr_str = ", ".join(f"{k}: {v}" for k, v in product.attributes.items())
        parts.append(attr_str)

    return ". ".join(parts)

# Example
product = Product(
    title="XPS 15",
    brand="Dell",
    category="Laptops",
    description="High-performance laptop for creative professionals",
    attributes={"Screen Size": "15.6 inch", "RAM": "16GB"}
)

prompt = construct_product_prompt(product)
# Result: "Dell XPS 15. Category: Laptops. High-performance laptop for creative professionals. Screen Size: 15.6 inch, RAM: 16GB"

embedding = generator.generate(prompt)
```

**Trade-off**: Longer prompts → more context but slower encoding and potential truncation.

---

## Multilingual Embeddings

### Zero-Shot Cross-Language Retrieval

**How it works**: Multilingual models (MPNet, E5, BGE-m3) align embeddings across languages during training.

```python
# English query
query_en = "best running shoes"
embedding_en = generator.generate(query_en)

# Spanish query
query_es = "mejores zapatillas para correr"
embedding_es = generator.generate(query_es)

# Cosine similarity (should be > 0.85 for good multilingual model)
similarity = np.dot(embedding_en, embedding_es)
print(similarity)  # 0.92 (very similar!)
```

**Validation test**:
```python
def test_multilingual_alignment():
    """Test that multilingual model aligns English and Spanish queries."""
    model = SentenceTransformer("multi-qa-mpnet-base-dot-v1")

    query_en = "laptop for gaming"
    query_es = "portátil para juegos"

    emb_en = model.encode(query_en, normalize_embeddings=True)
    emb_es = model.encode(query_es, normalize_embeddings=True)

    similarity = np.dot(emb_en, emb_es)
    assert similarity > 0.80, f"Multilingual alignment poor: {similarity}"
```

### Language Detection (Optional)

For language-specific optimizations:

```python
from langdetect import detect

def generate_with_language_hint(text: str, model: SentenceTransformer) -> np.ndarray:
    """Generate embedding with optional language-specific prefix."""
    lang = detect(text)

    # Some models (E5) support language prefixes for better alignment
    if lang == "es":
        text = f"query: {text}"  # E5-style prefix

    return model.encode(text, normalize_embeddings=True)
```

---

## Fine-Tuning for Domain Adaptation

### When to Fine-Tune

| Scenario | Fine-Tune? | Approach |
|----------|-----------|----------|
| **General commerce (electronics, furniture)** | ❌ No | Pre-trained models are sufficient |
| **Highly specialized domain (medical devices, chemicals)** | ✅ Yes | Fine-tune on domain corpus |
| **Poor zero-shot performance (NDCG < 0.60)** | ✅ Yes | Contrastive learning on query-product pairs |
| **Extreme multilingual (20+ languages)** | ✅ Maybe | Fine-tune on multilingual query logs |

### Fine-Tuning Approach: Contrastive Learning

**Data required**: Pairs of `(query, relevant_product, irrelevant_product)`

**Example dataset**:
```python
training_data = [
    {
        "query": "best laptop for students",
        "positive": "Dell Inspiron 15 Student Laptop",
        "negative": "Herman Miller Office Chair"
    },
    {
        "query": "zapatillas para correr",  # Spanish
        "positive": "Nike Air Zoom Pegasus Running Shoes",
        "negative": "Sony WH-1000XM5 Headphones"
    },
    # ... 10K+ examples
]
```

**Training code (Sentence Transformers)**:
```python
from sentence_transformers import SentenceTransformer, InputExample, losses
from torch.utils.data import DataLoader

# Load base model
model = SentenceTransformer("multi-qa-mpnet-base-dot-v1")

# Convert data to InputExample
train_examples = []
for item in training_data:
    train_examples.append(InputExample(
        texts=[item["query"], item["positive"]],
        label=1.0  # Positive pair
    ))
    train_examples.append(InputExample(
        texts=[item["query"], item["negative"]],
        label=0.0  # Negative pair
    ))

# Create DataLoader
train_dataloader = DataLoader(train_examples, shuffle=True, batch_size=16)

# Define loss (contrastive or triplet)
train_loss = losses.CosineSimilarityLoss(model)

# Fine-tune
model.fit(
    train_objectives=[(train_dataloader, train_loss)],
    epochs=3,
    warmup_steps=100,
    output_path="./models/commerce-mpnet-finetuned"
)
```

**Evaluation**: Measure NDCG@10 on held-out validation set before/after fine-tuning.

---

## Engineering Best Practices

### 1. Batching for Throughput

**Problem**: Encoding 100K products one-by-one is slow (hours).

**Solution**: Batch encode in groups of 32-128.

```python
def index_products(products: List[Product], batch_size: int = 64):
    """Efficiently generate embeddings for large product catalogs."""
    generator = EmbeddingGenerator()

    for i in range(0, len(products), batch_size):
        batch = products[i:i + batch_size]
        texts = [construct_product_prompt(p) for p in batch]

        embeddings = generator.generate_batch(texts, batch_size=batch_size)

        # Index into OpenSearch
        bulk_index(batch, embeddings)
```

**Performance**: Batching achieves ~10-50x speedup (GPU-dependent).

---

### 2. GPU Acceleration

**Recommendation**: Use GPU for embedding generation (100x faster than CPU).

```python
import torch

# Check if GPU available
device = "cuda" if torch.cuda.is_available() else "cpu"

model = SentenceTransformer("multi-qa-mpnet-base-dot-v1", device=device)

# Verify model is on GPU
print(model.device)  # cuda:0
```

**Hardware recommendations**:
- **Development**: NVIDIA GTX 1660 or better (6GB VRAM)
- **Production**: NVIDIA A10 or T4 (16-24GB VRAM for large batches)

---

### 3. Caching Embeddings

**Problem**: Re-computing embeddings for same product on every index rebuild is wasteful.

**Solution**: Cache embeddings keyed by content hash.

```python
import hashlib
import redis

class EmbeddingCache:
    def __init__(self, redis_client: redis.Redis):
        self.redis = redis_client
        self.generator = EmbeddingGenerator()

    def get_or_generate(self, text: str) -> np.ndarray:
        """Get embedding from cache or generate if missing."""
        # Hash text to create stable key
        text_hash = hashlib.sha256(text.encode()).hexdigest()
        cache_key = f"embedding:{text_hash}"

        # Try cache first
        cached = self.redis.get(cache_key)
        if cached:
            return np.frombuffer(cached, dtype=np.float32)

        # Cache miss: generate and store
        embedding = self.generator.generate(text)
        self.redis.set(cache_key, embedding.tobytes(), ex=86400 * 30)  # 30-day TTL

        return embedding
```

**Cache invalidation**: When prompt construction logic changes, flush cache.

---

### 4. Quantization for Storage Efficiency

**Problem**: 768-float embeddings × 1M products = 3 GB memory.

**Solution**: Quantize to `float16` (half precision) with minimal accuracy loss.

```python
def quantize_embedding(embedding: np.ndarray) -> np.ndarray:
    """Convert float32 to float16 (50% storage reduction)."""
    return embedding.astype(np.float16)

# Example
embedding_f32 = generator.generate("laptop")  # 768 × 4 bytes = 3072 bytes
embedding_f16 = quantize_embedding(embedding_f32)  # 768 × 2 bytes = 1536 bytes

# Verify minimal accuracy loss
similarity_before = np.dot(embedding_f32, embedding_f32)
similarity_after = np.dot(embedding_f16.astype(np.float32), embedding_f16.astype(np.float32))
print(similarity_before, similarity_after)  # 1.0000, 0.9998 (negligible)
```

**Trade-off**: 50% storage savings, <0.1% accuracy loss (acceptable for most use cases).

---

### 5. Monitoring Embedding Drift

**Problem**: Model updates or prompt changes can shift embeddings, breaking retrieval.

**Solution**: Track embedding statistics over time.

```python
def monitor_embedding_stats(embeddings: np.ndarray):
    """Log embedding statistics for drift detection."""
    mean_norm = np.linalg.norm(embeddings, axis=1).mean()
    std_norm = np.linalg.norm(embeddings, axis=1).std()
    mean_embedding = embeddings.mean(axis=0)

    metrics = {
        "mean_norm": mean_norm,
        "std_norm": std_norm,
        "mean_embedding_hash": hashlib.sha256(mean_embedding.tobytes()).hexdigest()[:8]
    }

    # Log to monitoring system (Prometheus, Datadog)
    log_metrics(metrics)

    return metrics

# Alerting rule
# If mean_norm changes by >10%, trigger alert (possible model drift)
```

---

## Evaluation and Benchmarking

### Intrinsic Evaluation (Embedding Quality)

**Cosine Similarity Tests**:
```python
def test_semantic_similarity():
    """Test that semantically similar phrases have high cosine similarity."""
    model = SentenceTransformer("multi-qa-mpnet-base-dot-v1")

    pairs = [
        ("running shoes", "athletic footwear", 0.80),  # Should be similar
        ("laptop computer", "notebook PC", 0.85),
        ("washing machine", "gaming console", 0.20),   # Should be dissimilar
    ]

    for text1, text2, expected_min_similarity in pairs:
        emb1 = model.encode(text1, normalize_embeddings=True)
        emb2 = model.encode(text2, normalize_embeddings=True)
        similarity = np.dot(emb1, emb2)

        assert similarity >= expected_min_similarity, \
            f"{text1} vs {text2}: similarity={similarity}, expected>={expected_min_similarity}"
```

### Extrinsic Evaluation (Search Quality)

**Measure retrieval metrics** (NDCG, MRR) on labeled query-product pairs:

```python
def evaluate_retrieval(model: SentenceTransformer, test_set: List[QueryProductPair]) -> dict:
    """Evaluate embedding model on retrieval task."""
    ndcg_scores = []
    mrr_scores = []

    for query, relevant_products in test_set:
        query_emb = model.encode(query, normalize_embeddings=True)

        # Retrieve top-k candidates from index
        candidates = index.search_knn(query_emb, k=100)

        # Compute NDCG
        ndcg = compute_ndcg(candidates, relevant_products, k=10)
        ndcg_scores.append(ndcg)

        # Compute MRR
        mrr = compute_mrr(candidates, relevant_products)
        mrr_scores.append(mrr)

    return {
        "ndcg@10": np.mean(ndcg_scores),
        "mrr": np.mean(mrr_scores)
    }
```

---

## Anti-Patterns

### ❌ Using TF-IDF or Word2Vec for Commerce Search
**Problem**: No semantic understanding; misses synonyms and cross-language queries.
**Solution**: Use transformer-based embeddings (BERT, MPNet, E5).

### ❌ Generating Embeddings from Raw Product Titles Only
**Problem**: "XPS 15" lacks context; poor retrieval quality.
**Solution**: Construct rich prompts with category, brand, description.

### ❌ Not Normalizing Embeddings Before Indexing
**Problem**: Cosine similarity assumes normalized vectors; unnormalized = incorrect scores.
**Solution**: Always L2-normalize (`normalize_embeddings=True`).

### ❌ Re-Generating Embeddings on Every Search Query
**Problem**: Adds 50-200ms latency per query.
**Solution**: Pre-compute and cache product embeddings; only generate query embeddings at search time.

### ❌ Using CPU for Large-Scale Embedding Generation
**Problem**: Indexing 100K products takes hours.
**Solution**: Use GPU (10-100x faster).

### ❌ Ignoring Model License for Production Use
**Problem**: Some models prohibit commercial use.
**Solution**: Verify license (e.g., Apache 2.0, MIT) before production deployment.

---

## Model Comparison Table

| Model | License | Dimensions | Languages | Speed (GPU) | BEIR NDCG@10 | Recommended For |
|-------|---------|------------|-----------|-------------|--------------|-----------------|
| **multi-qa-mpnet-base-dot-v1** | Apache 2.0 | 768 | 50+ | Fast | 0.52 | General commerce (multilingual) |
| **BAAI/bge-large-en-v1.5** | MIT | 1024 | English | Medium | 0.54 | English-only, max accuracy |
| **intfloat/e5-large-v2** | MIT | 1024 | 100+ | Medium | 0.53 | Multilingual with high accuracy |
| **all-MiniLM-L6-v2** | Apache 2.0 | 384 | English | Very Fast | 0.42 | Low-latency apps (mobile) |
| **BAAI/bge-m3** | MIT | 1024 | 100+ | Slow | 0.55 | Extreme multilingual |

**Note**: BEIR NDCG@10 is a standardized benchmark; higher = better retrieval quality.

---

## Related SBPF Documents

- **[OpenSearch-Vector-Search-Architecture.md](OpenSearch-Vector-Search-Architecture.md)**: Indexing embeddings with k-NN
- **[Semantic-Search-Intent-Modeling.md](Semantic-Search-Intent-Modeling.md)**: Using embeddings in the three-phase pipeline
- **[Web-Crawling-Best-Practices.md](Web-Crawling-Best-Practices.md)**: Building the corpus to embed

---

## References

- **Sentence Transformers**: [sbert.net](https://www.sbert.net/)
- **MTEB Leaderboard**: Model rankings for embedding tasks ([huggingface.co/spaces/mteb/leaderboard](https://huggingface.co/spaces/mteb/leaderboard))
- **BEIR Benchmark**: Retrieval evaluation dataset ([github.com/beir-cellar/beir](https://github.com/beir-cellar/beir))
- **Multilingual MPNet Paper**: Reimers & Gurevych (2020) - [arXiv:2004.09813](https://arxiv.org/abs/2004.09813)
- **BGE Embeddings**: BAAI (2023) - [GitHub](https://github.com/FlagOpen/FlagEmbedding)

---

**Summary**: Use pre-trained transformer embeddings (MPNet, E5, BGE) for commerce search. Construct rich prompts with category, brand, and attributes. Batch encode on GPU for efficiency. Normalize embeddings for cosine similarity. Cache embeddings to avoid redundant computation. Fine-tune only if zero-shot performance is insufficient. Validate with NDCG and cosine similarity tests.
