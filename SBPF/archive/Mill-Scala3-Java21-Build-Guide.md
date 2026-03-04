# Mill Build Tool Guide: Scala 3 + Java 21

## Overview

This guide documents how to use the [Mill build tool](https://mill-build.com/) for **Java 21** and **Scala 3** projects in accordance with our ceremony-based **DDD+BDD+TDD** methodology. Mill is our chosen build tool (ADR-056) for its simplicity, performance, and excellent support for monorepo structures with multiple bounded contexts.

**Prerequisites**: Read ADR-056 (Mill supersedes Maven) and POL-027 (JVM language choice) before using this guide.

---

## Why Mill?

### Advantages Over Alternatives

**vs. Maven**:
- ✅ Builds are 2-10x faster (aggressive caching, parallel execution)
- ✅ Configuration in Scala (type-safe, IDE support)
- ✅ Better monorepo support (cross-module dependencies)
- ✅ No XML configuration hell

**vs. Gradle**:
- ✅ Simpler mental model (functional, composable)
- ✅ Faster startup time (no daemon warm-up)
- ✅ More predictable caching
- ✅ Easier to reason about build graph

**vs. sbt**:
- ✅ Cleaner syntax (no DSL magic)
- ✅ Better error messages
- ✅ Faster incremental compilation
- ✅ Easier for Java developers to understand

### Alignment with Our Methodology

Mill fits our ceremony-based approach:
- **Bounded Contexts as Modules**: Each bounded context is a Mill module
- **Fast Feedback Loops**: Sub-second incremental builds support TDD
- **Monorepo-Friendly**: Supports multiple services in one repository
- **Reproducible Builds**: Consistent caching across team members
- **Java + Scala Unified**: One build tool for both languages (POL-027)

---

## Installation

### Prerequisites

- **Java 21** (required for our services per ADR-035)
- **Bash** or **Zsh** shell

### Install Mill

**Option 1: Direct Download (Recommended)**
```bash
# Download Mill 0.11.6 (or latest)
curl -L https://github.com/com-lihaoyi/mill/releases/download/0.11.6/0.11.6 > mill

# Make executable
chmod +x mill

# Move to PATH
sudo mv mill /usr/local/bin/

# Verify installation
mill version
# Output: Mill Build Tool version 0.11.6
```

**Option 2: Project-Local Wrapper (Team Recommendation)**
```bash
# Download millw wrapper script (ensures consistent Mill version across team)
curl -L https://raw.githubusercontent.com/lefou/millw/0.4.11/millw > millw
chmod +x millw

# Team members use ./millw instead of mill
./millw version

# Commit millw to git (team gets consistent Mill version)
git add millw
git commit -m "Add Mill wrapper for version consistency"
```

**Verification**:
```bash
mill version
# Should output: Mill Build Tool version 0.11.6
```

---

## Project Structure

### Monorepo Layout

```
copilot-training/
├── build.sc                    # Root build file (Mill configuration)
├── .mill-version               # Pin Mill version (0.11.6)
├── millw                       # Mill wrapper (team uses this)
├── out/                        # Build outputs (gitignored)
├── services/
│   ├── tenant-management/      # Bounded Context: Tenant (Java)
│   │   ├── src/
│   │   │   ├── main/java/
│   │   │   │   └── com/example/tenant/
│   │   │   │       ├── domain/         # Aggregates, value objects
│   │   │   │       ├── application/    # Pekko actors, use cases
│   │   │   │       └── infrastructure/ # Repositories, adapters
│   │   │   └── test/java/
│   │   │       └── com/example/tenant/
│   │   │           ├── domain/         # Unit tests
│   │   │           └── acceptance/     # BDD tests (Karate)
│   │   └── resources/
│   │       ├── application.conf        # Pekko configuration
│   │       └── db/migration/           # Flyway migrations
│   │
│   ├── billing/                # Bounded Context: Billing (Java)
│   │   └── [same structure as tenant-management]
│   │
│   └── catalog/                # Bounded Context: Catalog (Scala 3)
│       ├── src/
│       │   ├── main/scala/
│       │   │   └── com/example/catalog/
│       │   │       ├── domain/         # ADTs, enums, value objects
│       │   │       ├── application/    # ZIO/Pekko actors
│       │   │       └── infrastructure/
│       │   └── test/scala/
│       │       └── com/example/catalog/
│       │           ├── domain/         # ScalaTest unit tests
│       │           └── acceptance/     # BDD tests (Karate)
│       └── resources/
│
├── shared/                     # Shared libraries
│   └── domain-primitives/      # Shared value objects (Java)
│       └── src/main/java/
│
└── features/                   # BDD scenarios (Gherkin)
    ├── tenant/
    ├── billing/
    └── catalog/
```

---

## build.sc Configuration

### Root Build File

Create `build.sc` in repository root:

```scala
// build.sc
import mill._, scalalib._

// Common configuration for all Java modules
trait CommonJavaModule extends JavaModule {
  def javaVersion = "21"
  
  // Common dependencies (Pekko, logging, etc.)
  def ivyDeps = Agg(
    ivy"org.apache.pekko::pekko-actor-typed:1.0.2",
    ivy"ch.qos.logback:logback-classic:1.4.14",
    ivy"io.opentelemetry:opentelemetry-api:1.32.0"
  )
  
  // Test dependencies (JUnit 5, jqwik, Karate)
  object test extends JavaTests with TestModule.Junit5 {
    def ivyDeps = super.ivyDeps() ++ Agg(
      ivy"org.junit.jupiter:junit-jupiter:5.10.1",
      ivy"org.assertj:assertj-core:3.24.2",
      ivy"net.jqwik:jqwik:1.8.0",
      ivy"com.intuit.karate:karate-junit5:1.4.1",
      ivy"org.testcontainers:postgresql:1.19.3"
    )
  }
}

// Common configuration for all Scala modules
trait CommonScalaModule extends ScalaModule {
  def scalaVersion = "3.3.1"
  
  // Common dependencies
  def ivyDeps = Agg(
    ivy"org.apache.pekko::pekko-actor-typed:1.0.2",
    ivy"ch.qos.logback:logback-classic:1.4.14",
    ivy"io.opentelemetry:opentelemetry-api:1.32.0"
  )
  
  // Test dependencies (ScalaTest, jqwik-scala, Karate)
  object test extends ScalaTests with TestModule.ScalaTest {
    def ivyDeps = super.ivyDeps() ++ Agg(
      ivy"org.scalatest::scalatest:3.2.17",
      ivy"net.jqwik::jqwik-scala:0.8.0",
      ivy"com.intuit.karate:karate-junit5:1.4.1",
      ivy"org.testcontainers:postgresql:1.19.3"
    )
  }
}

// ============================================================================
// Bounded Context: Tenant Management (Java 21)
// ============================================================================
object tenantManagement extends CommonJavaModule {
  def ivyDeps = super.ivyDeps() ++ Agg(
    ivy"io.vertx:vertx-pg-client:4.5.0",         // Reactive PostgreSQL
    ivy"org.flywaydb:flyway-core:10.4.1",        // Database migrations
    ivy"com.fasterxml.jackson.core:jackson-databind:2.16.0"
  )
  
  // Depend on shared domain primitives
  def moduleDeps = Seq(domainPrimitives)
}

// ============================================================================
// Bounded Context: Billing (Java 21)
// ============================================================================
object billing extends CommonJavaModule {
  def ivyDeps = super.ivyDeps() ++ Agg(
    ivy"org.apache.kafka:kafka-clients:3.6.0",   // Kafka producer/consumer
    ivy"io.vertx:vertx-pg-client:4.5.0"
  )
  
  // Billing depends on Tenant (cross-context integration)
  def moduleDeps = Seq(tenantManagement, domainPrimitives)
}

// ============================================================================
// Bounded Context: Catalog (Scala 3)
// ============================================================================
object catalog extends CommonScalaModule {
  def ivyDeps = super.ivyDeps() ++ Agg(
    ivy"dev.zio::zio:2.0.19",                    // ZIO for effect management
    ivy"dev.zio::zio-streams:2.0.19",            // ZIO Streams
    ivy"io.getquill::quill-jdbc-zio:4.8.0",      // ZIO-based DB access
    ivy"org.postgresql:postgresql:42.7.1"
  )
  
  def moduleDeps = Seq(domainPrimitives)
}

// ============================================================================
// Shared: Domain Primitives (Java 21)
// ============================================================================
object domainPrimitives extends CommonJavaModule {
  // Shared value objects: TenantId, Money, etc.
  // No dependencies on other modules (leaf module)
}
```

---

## Common Tasks

### Compile

```bash
# Compile all modules
mill __.compile

# Compile specific module
mill tenantManagement.compile
mill catalog.compile

# Watch mode (recompile on file change)
mill -w tenantManagement.compile
```

### Run Tests

```bash
# Run all tests (unit + integration)
mill __.test

# Run tests for one module
mill tenantManagement.test

# Run tests in watch mode (TDD)
mill -w tenantManagement.test

# Run specific test class
mill tenantManagement.test com.example.tenant.domain.TenantTest

# Run BDD tests (Karate)
mill tenantManagement.test com.example.tenant.acceptance.TenantProvisioningTest
```

### Package JARs

```bash
# Create JAR with dependencies (fat JAR)
mill tenantManagement.assembly

# Output: out/tenantManagement/assembly.dest/out.jar

# Run the JAR
java -jar out/tenantManagement/assembly.dest/out.jar
```

### Clean Build Cache

```bash
# Clean specific module
mill tenantManagement.clean

# Clean all modules
mill clean

# Deep clean (remove entire out/ directory)
rm -rf out/
```

### IDE Integration

```bash
# Generate IntelliJ IDEA project
mill mill.idea.GenIdea/idea

# Open IntelliJ, then File > Open > select copilot-training directory
# IntelliJ recognizes Mill modules automatically
```

---

## Java 21 Module Example

### File Structure

```
tenant-management/
├── src/
│   ├── main/java/com/example/tenant/
│   │   ├── domain/
│   │   │   ├── Tenant.java              # Aggregate root
│   │   │   ├── TenantId.java            # Value object (record)
│   │   │   ├── CompanyName.java         # Value object
│   │   │   └── TenantStatus.java        # Enum (sealed interface)
│   │   ├── application/
│   │   │   ├── TenantActor.java         # Pekko actor
│   │   │   └── ProvisionTenantUseCase.java
│   │   └── infrastructure/
│   │       ├── TenantRepository.java    # Reactive repository
│   │       └── KafkaEventPublisher.java
│   └── test/java/com/example/tenant/
│       ├── domain/
│       │   ├── TenantTest.java          # Unit tests (JUnit 5)
│       │   └── TenantPropertyTest.java  # Property tests (jqwik)
│       └── acceptance/
│           └── TenantProvisioningTest.java  # BDD tests (Karate)
```

### Code Examples

**Value Object (Java 21 Record)**:
```java
// TenantId.java
package com.example.tenant.domain;

import java.util.UUID;

public record TenantId(UUID value) {
    public static TenantId generate() {
        return new TenantId(UUID.randomUUID());
    }
    
    public static TenantId of(String value) {
        return new TenantId(UUID.fromString(value));
    }
}
```

**Aggregate Root (Java 21)**:
```java
// Tenant.java
package com.example.tenant.domain;

public class Tenant {
    private final TenantId id;
    private CompanyName companyName;
    private TenantStatus status;
    
    public Tenant(TenantId id, CompanyName companyName) {
        this.id = id;
        this.companyName = companyName;
        this.status = TenantStatus.PROVISIONING;
    }
    
    public void activate() {
        if (status != TenantStatus.PROVISIONING && status != TenantStatus.SUSPENDED) {
            throw new IllegalStateTransitionException(
                "Cannot activate tenant from status: " + status
            );
        }
        this.status = TenantStatus.ACTIVE;
    }
    
    public TenantId getId() { return id; }
    public TenantStatus getStatus() { return status; }
}
```

**Unit Test (JUnit 5)**:
```java
// TenantTest.java
package com.example.tenant.domain;

import org.junit.jupiter.api.Test;
import static org.assertj.core.api.Assertions.*;

class TenantTest {
    
    @Test
    void shouldActivateTenantFromProvisioningStatus() {
        // Given
        Tenant tenant = new Tenant(
            TenantId.generate(),
            CompanyName.of("Acme Corp")
        );
        
        // When
        tenant.activate();
        
        // Then
        assertThat(tenant.getStatus()).isEqualTo(TenantStatus.ACTIVE);
    }
    
    @Test
    void shouldRejectActivationFromTerminatedStatus() {
        // Given
        Tenant tenant = new Tenant(TenantId.generate(), CompanyName.of("Acme"));
        tenant.terminate();
        
        // When/Then
        assertThatThrownBy(tenant::activate)
            .isInstanceOf(IllegalStateTransitionException.class)
            .hasMessageContaining("Cannot activate");
    }
}
```

**Run Tests**:
```bash
# Run all Tenant tests
mill tenantManagement.test

# Watch mode (TDD)
mill -w tenantManagement.test
```

---

## Scala 3 Module Example

### File Structure

```
catalog/
├── src/
│   ├── main/scala/com/example/catalog/
│   │   ├── domain/
│   │   │   ├── Product.scala            # Aggregate root
│   │   │   ├── ProductId.scala          # Value object (opaque type)
│   │   │   └── ProductStatus.scala      # ADT (enum)
│   │   ├── application/
│   │   │   ├── ProductActor.scala       # Pekko actor
│   │   │   └── SearchProductUseCase.scala
│   │   └── infrastructure/
│   │       └── ProductRepository.scala  # ZIO-based repository
│   └── test/scala/com/example/catalog/
│       ├── domain/
│       │   ├── ProductSpec.scala        # Unit tests (ScalaTest)
│       │   └── ProductPropertySpec.scala # Property tests (jqwik-scala)
│       └── acceptance/
│           └── ProductSearchSpec.scala  # BDD tests (Karate)
```

### Code Examples

**Value Object (Scala 3 Opaque Type)**:
```scala
// ProductId.scala
package com.example.catalog.domain

import java.util.UUID

opaque type ProductId = UUID

object ProductId:
  def apply(value: UUID): ProductId = value
  
  def generate(): ProductId = UUID.randomUUID()
  
  def parse(value: String): Either[String, ProductId] =
    try Right(UUID.fromString(value))
    catch case _: IllegalArgumentException => Left("Invalid UUID")
  
  extension (id: ProductId)
    def value: UUID = id
```

**Aggregate Root (Scala 3 with ADT)**:
```scala
// Product.scala
package com.example.catalog.domain

enum ProductStatus:
  case Draft, Published, Archived

case class Product(
  id: ProductId,
  name: String,
  price: BigDecimal,
  status: ProductStatus
):
  def publish: Either[String, Product] =
    status match
      case ProductStatus.Draft =>
        Right(this.copy(status = ProductStatus.Published))
      case _ =>
        Left(s"Cannot publish product from status: $status")
  
  def archive: Either[String, Product] =
    status match
      case ProductStatus.Published =>
        Right(this.copy(status = ProductStatus.Archived))
      case _ =>
        Left(s"Cannot archive product from status: $status")
```

**Unit Test (ScalaTest)**:
```scala
// ProductSpec.scala
package com.example.catalog.domain

import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers

class ProductSpec extends AnyFunSuite with Matchers:
  
  test("should publish product from Draft status"):
    // Given
    val product = Product(
      ProductId.generate(),
      "Widget",
      BigDecimal(19.99),
      ProductStatus.Draft
    )
    
    // When
    val result = product.publish
    
    // Then
    result shouldBe a[Right[_, _]]
    result.map(_.status) shouldBe Right(ProductStatus.Published)
  
  test("should reject publishing from Published status"):
    // Given
    val product = Product(
      ProductId.generate(),
      "Widget",
      BigDecimal(19.99),
      ProductStatus.Published
    )
    
    // When
    val result = product.publish
    
    // Then
    result shouldBe a[Left[_, _]]
    result.left.map(_ should include("Cannot publish"))
```

**Run Tests**:
```bash
# Run all Catalog tests
mill catalog.test

# Watch mode (TDD)
mill -w catalog.test
```

---

## Cross-Module Dependencies

### Scenario: Billing depends on Tenant

**build.sc**:
```scala
object billing extends CommonJavaModule {
  // Billing can import Tenant domain models
  def moduleDeps = Seq(tenantManagement)
}
```

**Usage in Billing**:
```java
// billing/src/main/java/com/example/billing/Invoice.java
package com.example.billing;

import com.example.tenant.domain.TenantId;  // ✅ Import from tenantManagement module

public record Invoice(
    InvoiceId id,
    TenantId tenantId,  // Reference to Tenant aggregate
    Money amount
) {}
```

**Compile**:
```bash
# Mill detects dependency, compiles tenantManagement first, then billing
mill billing.compile
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Java 21
        uses: actions/setup-java@v4
        with:
          java-version: '21'
          distribution: 'temurin'
      
      - name: Cache Mill
        uses: actions/cache@v3
        with:
          path: ~/.cache/mill
          key: ${{ runner.os }}-mill-${{ hashFiles('build.sc') }}
      
      - name: Install Mill
        run: |
          curl -L https://github.com/com-lihaoyi/mill/releases/download/0.11.6/0.11.6 > mill
          chmod +x mill
          sudo mv mill /usr/local/bin/
      
      - name: Compile All Modules
        run: mill __.compile
      
      - name: Run All Tests
        run: mill __.test
      
      - name: Package JARs
        run: mill __.assembly
      
      - name: Upload Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: jars
          path: out/**/assembly.dest/out.jar
```

---

## Troubleshooting

### Issue: Mill command not found
**Solution**:
```bash
# Install Mill
curl -L https://github.com/com-lihaoyi/mill/releases/download/0.11.6/0.11.6 > mill
chmod +x mill
sudo mv mill /usr/local/bin/
```

### Issue: IntelliJ doesn't recognize project
**Solution**:
```bash
# Regenerate IntelliJ project
mill mill.idea.GenIdea/idea

# Restart IntelliJ
# File > Open > select copilot-training directory
```

### Issue: Incremental build not working
**Solution**:
```bash
# Clean cache and rebuild
mill clean
mill __.compile
```

### Issue: Out of memory during compilation
**Solution**:
```bash
# Increase Mill heap size
export MILL_JVM_OPTS="-Xmx4G"
mill __.compile
```

### Issue: Scala 3 compiler errors (metaprogramming)
**Solution**:
```bash
# Scala 3 metaprogramming (macros) sometimes fails incrementally
# Force full recompilation
mill clean catalog.compile
```

---

## Best Practices

### 1. Pin Mill Version
Create `.mill-version` file:
```
0.11.6
```

Commit to git (ensures team uses same Mill version).

### 2. Use Wrapper Script
Commit `millw` script (auto-downloads correct Mill version):
```bash
curl -L https://raw.githubusercontent.com/lefou/millw/0.4.11/millw > millw
chmod +x millw
git add millw
```

Team uses `./millw` instead of `mill`.

### 3. Organize Modules by Bounded Context
```
build.sc:
- tenantManagement (Java)
- billing (Java)
- catalog (Scala)
```

Each module = one deployable service.

### 4. Share Common Configuration
```scala
// build.sc
trait CommonJavaModule extends JavaModule {
  def javaVersion = "21"
  // Shared deps, test setup
}

object tenantManagement extends CommonJavaModule {
  // Service-specific deps
}
```

### 5. Watch Mode for TDD
```bash
# Continuous testing (re-run on file save)
mill -w tenantManagement.test
```

### 6. Fast Feedback: Compile Before Test
```bash
# Compile catches errors faster than tests
mill -w tenantManagement.compile
```

---

## Related Documentation

- **ADR-056**: Mill Supersedes Maven (why Mill chosen)
- **ADR-057**: Scala 3 for Functional Services (when to use Scala vs Java)
- **ADR-035**: Java 21 LTS Baseline (Java version standard)
- **POL-027**: JVM Language Choice (Java 17+ or Scala 3+ only)
- **HOW-WE-WORK.md**: Ceremony-based SDLC (TDD workflow using Mill)

---

## Further Resources

- **Mill Documentation**: https://mill-build.com/
- **Mill GitHub**: https://github.com/com-lihaoyi/mill
- **Example Repos**: https://github.com/com-lihaoyi/mill/tree/main/example
- **Mill Gitter Chat**: https://gitter.im/lihaoyi/mill
