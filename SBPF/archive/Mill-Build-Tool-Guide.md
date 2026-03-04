# Mill Build Tool Guide

## Overview

This guide documents how to use the [Mill build tool](https://mill-build.com/) for Java/Scala projects in accordance with our ceremony-based DDD+BDD+TDD methodology. Mill is our chosen build tool for its simplicity, performance, and excellent support for monorepo structures with multiple bounded contexts.

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

---

## Installation

### Prerequisites

- **Java 11+** (we use Java 21+)
- **Bash** or **Zsh** shell

### Install Mill

**Option 1: Direct Download (Recommended)**
```bash
# Download latest Mill launcher
curl -L https://github.com/com-lihaoyi/mill/releases/download/0.11.6/0.11.6 > mill

# Make executable
chmod +x mill

# Move to PATH
sudo mv mill /usr/local/bin/
```

**Option 2: Via Coursier**
```bash
cs install mill
```

**Option 3: Project-Local Wrapper (Recommended for Teams)**
```bash
# Download millw wrapper script
curl -L https://raw.githubusercontent.com/lefou/millw/0.4.11/millw > millw
chmod +x millw

# Team members use ./millw instead of mill
./millw version
```

**Verify Installation**:
```bash
mill version
# Should output: Mill Build Tool version 0.11.6
```

---

## Project Structure

### Monorepo Layout

```
copilot-training/
├── build.sc                    # Root build file
├── .mill-version              # Pin Mill version
├── out/                       # Build outputs (gitignored)
├── services/
│   ├── tenant-management/
│   │   ├── src/
│   │   │   ├── main/java/
│   │   │   └── test/java/
│   │   └── resources/
│   ├── billing/
│   │   ├── src/
│   │   │   ├── main/java/
│   │   │   └── test/java/
│   │   └── resources/
│   └── order-fulfillment/
│       ├── src/
│       │   ├── main/java/
│       │   └── test/java/
│       └── resources/
├── shared/
│   ├── domain-kernel/         # Shared value objects
│   └── event-contracts/       # Event schemas
└── features/                  # BDD scenarios (Karate)
```

### Bounded Context = Mill Module

Each bounded context maps to a Mill module:
- **tenant-management** → `services.tenantManagement`
- **billing** → `services.billing`
- **order-fulfillment** → `services.orderFulfillment`

---

## Basic Build File

### Root `build.sc`

```scala
import mill._
import mill.scalalib._
import mill.javalib._

// Define Java version
val javaVersion = "21"

// Define dependency versions
object Versions {
  val pekko = "1.0.2"
  val pekkoHttp = "1.0.1"
  val kafka = "3.6.1"
  val postgres = "42.7.1"
  val karate = "1.4.1"
  val junit = "5.10.1"
  val assertj = "3.24.2"
  val jqwik = "1.8.2"
  val logback = "1.4.14"
  val opentelemetry = "1.34.1"
}

// Base module for all services
trait BaseJavaModule extends JavaModule {
  def javaVersion = T { "21" }
  
  // Common dependencies
  def commonDeps = Agg(
    ivy"ch.qos.logback:logback-classic:${Versions.logback}",
    ivy"io.opentelemetry:opentelemetry-api:${Versions.opentelemetry}",
  )
  
  // Common test dependencies
  def commonTestDeps = Agg(
    ivy"org.junit.jupiter:junit-jupiter-api:${Versions.junit}",
    ivy"org.junit.jupiter:junit-jupiter-engine:${Versions.junit}",
    ivy"org.assertj:assertj-core:${Versions.assertj}",
    ivy"net.jqwik:jqwik:${Versions.jqwik}",
  )
  
  override def ivyDeps = commonDeps
  override def testIvyDeps = commonTestDeps
  
  // Use JUnit 5
  override def testFramework = "com.github.sbt.junit.JupiterFramework"
}

// Shared modules
object shared extends Module {
  object domainKernel extends BaseJavaModule {
    override def ivyDeps = super.ivyDeps() ++ Agg(
      // Value objects, domain primitives
    )
  }
  
  object eventContracts extends BaseJavaModule {
    override def moduleDeps = Seq(domainKernel)
    override def ivyDeps = super.ivyDeps() ++ Agg(
      ivy"com.fasterxml.jackson.core:jackson-databind:2.16.1",
    )
  }
}

// Service modules
object services extends Module {
  
  // Tenant Management Service
  object tenantManagement extends BaseJavaModule {
    override def moduleDeps = Seq(
      shared.domainKernel,
      shared.eventContracts,
    )
    
    override def ivyDeps = super.ivyDeps() ++ Agg(
      // Pekko
      ivy"org.apache.pekko::pekko-actor-typed:${Versions.pekko}",
      ivy"org.apache.pekko::pekko-stream:${Versions.pekko}",
      ivy"org.apache.pekko::pekko-cluster-typed:${Versions.pekko}",
      ivy"org.apache.pekko::pekko-persistence-typed:${Versions.pekko}",
      
      // Kafka
      ivy"org.apache.kafka:kafka-clients:${Versions.kafka}",
      
      // Reactive Postgres
      ivy"io.vertx:vertx-pg-client:4.5.1",
      ivy"io.vertx:vertx-sql-client:4.5.1",
    )
    
    override def testIvyDeps = super.testIvyDeps() ++ Agg(
      ivy"org.apache.pekko::pekko-actor-testkit-typed:${Versions.pekko}",
    )
  }
  
  // Billing Service
  object billing extends BaseJavaModule {
    override def moduleDeps = Seq(
      shared.domainKernel,
      shared.eventContracts,
    )
    
    override def ivyDeps = super.ivyDeps() ++ Agg(
      ivy"org.apache.pekko::pekko-actor-typed:${Versions.pekko}",
      ivy"org.apache.pekko::pekko-stream:${Versions.pekko}",
      ivy"org.apache.kafka:kafka-clients:${Versions.kafka}",
      ivy"io.vertx:vertx-pg-client:4.5.1",
    )
  }
  
  // Order Fulfillment Service
  object orderFulfillment extends BaseJavaModule {
    override def moduleDeps = Seq(
      shared.domainKernel,
      shared.eventContracts,
    )
    
    override def ivyDeps = super.ivyDeps() ++ Agg(
      ivy"org.apache.pekko::pekko-actor-typed:${Versions.pekko}",
      ivy"org.apache.pekko::pekko-stream:${Versions.pekko}",
      ivy"org.apache.kafka:kafka-clients:${Versions.kafka}",
      ivy"io.vertx:vertx-pg-client:4.5.1",
    )
  }
}

// BDD Testing Module
object features extends JavaModule {
  def javaVersion = T { "21" }
  
  override def ivyDeps = Agg(
    ivy"com.intuit.karate:karate-junit5:${Versions.karate}",
  )
  
  override def testFramework = "com.github.sbt.junit.JupiterFramework"
  
  // Reference all services for integration testing
  override def moduleDeps = Seq(
    services.tenantManagement,
    services.billing,
    services.orderFulfillment,
  )
}
```

---

## Common Tasks

### Building

**Compile All Modules**:
```bash
mill __.compile
```

**Compile Specific Module**:
```bash
mill services.tenantManagement.compile
```

**Compile with Dependencies**:
```bash
# Automatically compiles dependencies (e.g., shared.domainKernel)
mill services.tenantManagement.compile
```

**Clean All**:
```bash
mill clean
```

**Clean Specific Module**:
```bash
mill services.tenantManagement.clean
```

---

### Testing

**Run All Tests**:
```bash
mill __.test
```

**Run Tests for Specific Module**:
```bash
mill services.tenantManagement.test
```

**Run Single Test Class**:
```bash
mill services.tenantManagement.test "com.example.tenant.TenantAggregateTest"
```

**Run Tests with Pattern**:
```bash
mill services.tenantManagement.test "com.example.tenant.*"
```

**Watch Mode (TDD)**:
```bash
# Recompile and retest on file changes
mill --watch services.tenantManagement.test
```

**Run BDD Scenarios (Karate)**:
```bash
mill features.test
```

**Run Single BDD Feature**:
```bash
mill features.test "features/tenant-provisioning.feature"
```

---

### Running Services

**Run Service**:
```bash
mill services.tenantManagement.run
```

**Run with Arguments**:
```bash
mill services.tenantManagement.run --config prod.conf
```

**Background Run**:
```bash
mill services.tenantManagement.run &
```

---

### Dependency Management

**Show Dependency Tree**:
```bash
mill show services.tenantManagement.ivyDeps
```

**Resolve Dependencies**:
```bash
mill services.tenantManagement.resolvedIvyDeps
```

**Update Dependencies**:
```bash
# Mill automatically fetches latest matching versions
mill services.tenantManagement.compile
```

---

### Packaging

**Create JAR**:
```bash
mill services.tenantManagement.jar
```

**Create Executable JAR (with dependencies)**:
```bash
mill services.tenantManagement.assembly
```

**Output Location**:
```bash
# JAR location
out/services/tenantManagement/jar.dest/out.jar

# Assembly JAR location
out/services/tenantManagement/assembly.dest/out.jar
```

**Run Packaged JAR**:
```bash
java -jar out/services/tenantManagement/assembly.dest/out.jar
```

---

### Docker Integration

**Custom Assembly Task**:
```scala
object services extends Module {
  object tenantManagement extends BaseJavaModule {
    // ... existing config ...
    
    // Custom task to copy assembly to docker context
    def dockerPrep() = T.command {
      val assemblyPath = assembly().path
      os.copy(assemblyPath, os.pwd / "docker" / "tenant-management" / "app.jar", replaceExisting = true)
      println(s"Copied assembly to docker/tenant-management/app.jar")
    }
  }
}
```

**Usage**:
```bash
# Build and prepare for Docker
mill services.tenantManagement.dockerPrep

# Build Docker image
docker build -t tenant-management:latest docker/tenant-management/
```

---

## Advanced Patterns

### Cross-Module Dependencies

**Problem**: Service A depends on Service B's domain model (Anti-Corruption Layer pattern).

**Solution**: Extract shared domain to `shared/` module:

```scala
object shared extends Module {
  object tenantDomainKernel extends BaseJavaModule {
    // Tenant value objects, events
  }
}

object services extends Module {
  object tenantManagement extends BaseJavaModule {
    override def moduleDeps = Seq(shared.tenantDomainKernel)
  }
  
  object billing extends BaseJavaModule {
    // Billing depends on Tenant events (ACL)
    override def moduleDeps = Seq(shared.tenantDomainKernel)
  }
}
```

**Best Practice**: Only share:
- Value objects
- Domain events (published language)
- DTOs for integration contracts

**Never share**: Aggregates, repositories, application services (breaks bounded context isolation).

---

### Custom Tasks

**Problem**: Need to generate code from OpenAPI spec.

**Solution**: Define custom task:

```scala
object services extends Module {
  object tenantManagement extends BaseJavaModule {
    
    // Custom task to generate API client
    def generateApiClient() = T.command {
      val specPath = millSourcePath / "src" / "main" / "resources" / "openapi.yaml"
      val outputDir = millSourcePath / "src" / "generated" / "java"
      
      os.proc(
        "openapi-generator-cli", "generate",
        "-i", specPath,
        "-g", "java",
        "-o", outputDir,
        "--library", "native"
      ).call(stdout = os.Inherit)
      
      println(s"Generated API client in $outputDir")
    }
    
    // Run before compilation
    override def generatedSources = T {
      generateApiClient()
      super.generatedSources() ++ Seq(PathRef(millSourcePath / "src" / "generated" / "java"))
    }
  }
}
```

---

### Multi-Environment Configuration

**Problem**: Different configs for dev/test/prod.

**Solution**: Use Mill targets:

```scala
object services extends Module {
  object tenantManagement extends BaseJavaModule {
    
    // Define environment configs
    def devConfig = T.source { millSourcePath / "config" / "dev.conf" }
    def testConfig = T.source { millSourcePath / "config" / "test.conf" }
    def prodConfig = T.source { millSourcePath / "config" / "prod.conf" }
    
    // Run with specific config
    def runDev() = T.command {
      run(devConfig().path.toString)
    }
    
    def runProd() = T.command {
      run(prodConfig().path.toString)
    }
  }
}
```

**Usage**:
```bash
mill services.tenantManagement.runDev
mill services.tenantManagement.runProd
```

---

### Property-Based Testing with Jqwik

**Build Configuration**:
```scala
trait BaseJavaModule extends JavaModule {
  override def testIvyDeps = super.testIvyDeps() ++ Agg(
    ivy"net.jqwik:jqwik:1.8.2",
    ivy"net.jqwik:jqwik-engine:1.8.2",
  )
  
  // Ensure Jqwik engine is discovered
  override def testFramework = "com.github.sbt.junit.JupiterFramework"
}
```

**Run Property Tests**:
```bash
mill services.tenantManagement.test
```

---

### Code Coverage

**Add JaCoCo Plugin**:
```scala
import $ivy.`com.lihaoyi::mill-contrib-jmh:0.11.6`
import mill.contrib.jmh._

object services extends Module {
  object tenantManagement extends BaseJavaModule with JacocoModule {
    // ... existing config ...
  }
}
```

**Generate Coverage Report**:
```bash
mill services.tenantManagement.test
mill services.tenantManagement.jacocoReport

# Open report
open out/services/tenantManagement/jacocoReport.dest/html/index.html
```

---

## TDD Workflow with Mill

### Red-Green-Refactor Cycle

**1. Write Failing Test** (Red):
```bash
# Create test file
vim services/tenant-management/src/test/java/com/example/tenant/TenantAggregateTest.java

# Run test (should fail)
mill services.tenantManagement.test "com.example.tenant.TenantAggregateTest"
```

**2. Implement Minimum Code** (Green):
```bash
# Create implementation
vim services/tenant-management/src/main/java/com/example/tenant/Tenant.java

# Run test (should pass)
mill services.tenantManagement.test "com.example.tenant.TenantAggregateTest"
```

**3. Refactor**:
```bash
# Refactor implementation
vim services/tenant-management/src/main/java/com/example/tenant/Tenant.java

# Run test (should still pass)
mill services.tenantManagement.test "com.example.tenant.TenantAggregateTest"
```

**4. Watch Mode for Continuous Feedback**:
```bash
# Automatically rerun tests on file save
mill --watch services.tenantManagement.test
```

---

## BDD Workflow with Mill

### Running Karate Scenarios

**1. Write BDD Scenario** (Given/When/Then):
```bash
vim features/tenant-provisioning.feature
```

**2. Run Scenario** (Red):
```bash
mill features.test "features/tenant-provisioning.feature"
# Should fail - no implementation yet
```

**3. Implement with TDD**:
```bash
# Follow TDD cycle above
mill --watch services.tenantManagement.test
```

**4. Run BDD Scenario Again** (Green):
```bash
mill features.test "features/tenant-provisioning.feature"
# Should pass - implementation complete
```

---

## Performance Optimization

### Caching

Mill aggressively caches build outputs. Cache is stored in `out/`:

**Cache Location**:
```
out/
├── services/
│   ├── tenantManagement/
│   │   ├── compile.dest/        # Compiled classes (cached)
│   │   ├── test.dest/           # Test results (cached)
│   │   └── jar.dest/            # JAR artifacts (cached)
```

**Cache Invalidation**:
- Source files change → recompile
- Dependencies change → re-resolve and recompile
- Configuration change → rebuild affected modules

**Force Rebuild**:
```bash
mill clean services.tenantManagement
mill services.tenantManagement.compile
```

---

### Parallel Execution

Mill automatically parallelizes independent tasks:

```bash
# Compiles all services in parallel (if independent)
mill __.compile
```

**Control Parallelism**:
```bash
# Limit to 4 parallel tasks
mill --jobs 4 __.compile
```

---

### Incremental Compilation

Mill uses Zinc incremental compiler:
- Only recompiles changed files and dependents
- Sub-second recompilation for small changes

**Example**:
```bash
# Initial compilation: 30 seconds
mill services.tenantManagement.compile

# Change one file
vim services/tenant-management/src/main/java/com/example/tenant/Tenant.java

# Incremental compilation: 2 seconds
mill services.tenantManagement.compile
```

---

## Troubleshooting

### Issue: Mill Not Found

**Symptom**:
```bash
mill: command not found
```

**Solution**:
```bash
# Check PATH
echo $PATH

# Reinstall Mill
curl -L https://github.com/com-lihaoyi/mill/releases/download/0.11.6/0.11.6 > mill
chmod +x mill
sudo mv mill /usr/local/bin/
```

---

### Issue: Dependency Resolution Fails

**Symptom**:
```
Error resolving dependencies: Could not find artifact ...
```

**Solution 1: Check Maven Central availability**:
```bash
# Search Maven Central
open "https://search.maven.org/artifact/org.apache.pekko/pekko-actor-typed_2.13/1.0.2/jar"
```

**Solution 2: Add custom repository**:
```scala
override def repositoriesTask = T.task {
  super.repositoriesTask() ++ Seq(
    MavenRepository("https://repo.akka.io/maven/"),
  )
}
```

---

### Issue: Tests Not Running

**Symptom**:
```bash
mill services.tenantManagement.test
# No tests executed
```

**Solution 1: Check test framework**:
```scala
override def testFramework = "com.github.sbt.junit.JupiterFramework"
```

**Solution 2: Check test dependencies**:
```scala
override def testIvyDeps = Agg(
  ivy"org.junit.jupiter:junit-jupiter-api:5.10.1",
  ivy"org.junit.jupiter:junit-jupiter-engine:5.10.1",
)
```

**Solution 3: Verify test location**:
```bash
# Tests must be in src/test/java/
ls services/tenant-management/src/test/java/
```

---

### Issue: Out of Memory

**Symptom**:
```
java.lang.OutOfMemoryError: Java heap space
```

**Solution 1: Increase Mill heap**:
```bash
export JAVA_OPTS="-Xmx4G -Xms1G"
mill services.tenantManagement.compile
```

**Solution 2: Set in `.mill-jvm-opts`**:
```bash
echo "-Xmx4G" > .mill-jvm-opts
echo "-Xms1G" >> .mill-jvm-opts
```

---

### Issue: Slow Compilation

**Symptom**: First compilation takes several minutes.

**Solution 1: Warm up cache**:
```bash
# Run once to cache dependencies
mill __.compile
```

**Solution 2: Use local Ivy cache**:
```bash
# Mill uses ~/.ivy2/cache by default
# Ensure cache is not corrupted
rm -rf ~/.ivy2/cache
mill __.compile
```

---

## Best Practices

### 1. Pin Mill Version

**Problem**: Different team members use different Mill versions.

**Solution**: Create `.mill-version`:
```bash
echo "0.11.6" > .mill-version
```

Team members run:
```bash
mill version
# Automatically downloads correct version
```

---

### 2. Use Millw Wrapper

**Problem**: New team members must install Mill.

**Solution**: Commit `millw` wrapper:
```bash
curl -L https://raw.githubusercontent.com/lefou/millw/0.4.11/millw > millw
chmod +x millw
git add millw
git commit -m "Add Mill wrapper"
```

Team members run:
```bash
./millw compile  # No installation needed
```

---

### 3. Organize Modules by Bounded Context

**Good** (Domain-aligned):
```scala
object services extends Module {
  object tenantManagement extends BaseJavaModule { ... }
  object billing extends BaseJavaModule { ... }
  object orderFulfillment extends BaseJavaModule { ... }
}
```

**Bad** (Technical layers):
```scala
object api extends BaseJavaModule { ... }
object domain extends BaseJavaModule { ... }
object infrastructure extends BaseJavaModule { ... }
```

---

### 4. Share Minimal Domain Artifacts

**Only share**:
```scala
object shared extends Module {
  object domainKernel extends BaseJavaModule {
    // Value objects (TenantId, CompanyName)
    // Domain events (TenantActivated)
    // DTOs for integration contracts
  }
}
```

**Never share**:
- Aggregates
- Repositories
- Application services
- Domain logic

---

### 5. Use Watch Mode for TDD

**Always run in watch mode during TDD**:
```bash
mill --watch services.tenantManagement.test
```

This gives sub-second feedback on test failures.

---

### 6. Separate BDD Scenarios

**Keep BDD scenarios in separate module**:
```scala
object features extends JavaModule {
  def javaVersion = T { "21" }
  override def ivyDeps = Agg(
    ivy"com.intuit.karate:karate-junit5:1.4.1",
  )
  override def moduleDeps = Seq(
    services.tenantManagement,
    services.billing,
  )
}
```

**Benefits**:
- Clear separation of unit tests (TDD) vs. BDD scenarios
- Can run independently: `mill features.test`
- Easier to manage cross-service scenarios

---

### 7. Version Dependencies Centrally

**Use `Versions` object**:
```scala
object Versions {
  val pekko = "1.0.2"
  val kafka = "3.6.1"
  val junit = "5.10.1"
}

trait BaseJavaModule extends JavaModule {
  override def ivyDeps = Agg(
    ivy"org.apache.pekko::pekko-actor-typed:${Versions.pekko}",
  )
}
```

**Benefits**:
- Single source of truth
- Easy to upgrade all modules
- Prevents version conflicts

---

### 8. Clean Build Before Commits

**Before committing**:
```bash
# Clean and rebuild everything
mill clean
mill __.compile
mill __.test

# If all pass, commit
git commit -m "Implement tenant activation"
```

---

### 9. Use Custom Tasks for Codegen

**Example: Generate event schemas from Avro**:
```scala
object shared extends Module {
  object eventContracts extends BaseJavaModule {
    
    def generateAvro() = T.command {
      val schemaDir = millSourcePath / "src" / "main" / "avro"
      val outputDir = millSourcePath / "src" / "generated" / "java"
      
      os.proc(
        "avro-tools", "compile", "schema",
        schemaDir.toString,
        outputDir.toString
      ).call()
    }
    
    override def generatedSources = T {
      generateAvro()
      super.generatedSources() ++ Seq(PathRef(millSourcePath / "src" / "generated" / "java"))
    }
  }
}
```

---

### 10. Document Build Commands

**Create `BUILD.md` in each service**:
```markdown
# Tenant Management Service - Build Guide

## Compile
```bash
mill services.tenantManagement.compile
```

## Test
```bash
mill services.tenantManagement.test
```

## Run
```bash
mill services.tenantManagement.run
```

## Package
```bash
mill services.tenantManagement.assembly
```
```

---

## Integration with Ceremonies

### Discovery Phase

**Event Storming → Ubiquitous Language → Domain Modeling → Context Mapping**

After Context Mapping, create Mill modules:
```bash
# Create module for new bounded context
mkdir -p services/new-context/src/{main,test}/java
```

Update `build.sc`:
```scala
object services extends Module {
  object newContext extends BaseJavaModule {
    override def moduleDeps = Seq(shared.domainKernel)
  }
}
```

---

### Specification Phase

**Three Amigos → Example Mapping → Acceptance Criteria Review**

Write BDD scenarios in `features/`:
```bash
vim features/new-feature.feature
```

Run BDD scenario (Red):
```bash
mill features.test "features/new-feature.feature"
```

---

### Implementation Phase

**Test-First Pairing → Red-Green-Refactor → Property-Based Testing**

Use watch mode for TDD:
```bash
mill --watch services.newContext.test
```

Run property-based tests:
```bash
mill services.newContext.test "com.example.newcontext.*PropertyTest"
```

---

### Integration Phase

**Scenario-to-Test Decomposition → Living Documentation Sync → Cross-Boundary Integration Testing**

Run cross-boundary tests:
```bash
mill features.test
```

Update documentation:
```bash
# Update SERVICE-CHARTER with build instructions
vim doc/services/new-context/SERVICE-CHARTER.md
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Mill CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Java 21
      uses: actions/setup-java@v4
      with:
        java-version: '21'
        distribution: 'temurin'
    
    - name: Cache Mill
      uses: actions/cache@v3
      with:
        path: |
          ~/.cache/mill
          ~/.ivy2/cache
        key: ${{ runner.os }}-mill-${{ hashFiles('build.sc') }}
    
    - name: Install Mill
      run: |
        curl -L https://github.com/com-lihaoyi/mill/releases/download/0.11.6/0.11.6 > mill
        chmod +x mill
        sudo mv mill /usr/local/bin/
    
    - name: Compile
      run: mill __.compile
    
    - name: Run Tests
      run: mill __.test
    
    - name: Run BDD Scenarios
      run: mill features.test
    
    - name: Package
      run: mill services.tenantManagement.assembly
    
    - name: Upload Artifacts
      uses: actions/upload-artifact@v3
      with:
        name: tenant-management-jar
        path: out/services/tenantManagement/assembly.dest/out.jar
```

---

### GitLab CI Example

```yaml
image: eclipse-temurin:21-jdk

stages:
  - build
  - test
  - package

cache:
  paths:
    - .ivy2/cache
    - out/

before_script:
  - curl -L https://github.com/com-lihaoyi/mill/releases/download/0.11.6/0.11.6 > mill
  - chmod +x mill

compile:
  stage: build
  script:
    - ./mill __.compile

unit-test:
  stage: test
  script:
    - ./mill __.test

bdd-test:
  stage: test
  script:
    - ./mill features.test

package:
  stage: package
  script:
    - ./mill services.tenantManagement.assembly
  artifacts:
    paths:
      - out/services/tenantManagement/assembly.dest/out.jar
```

---

## Migration from Maven/Gradle

### From Maven

**Maven `pom.xml`**:
```xml
<dependencies>
  <dependency>
    <groupId>org.apache.pekko</groupId>
    <artifactId>pekko-actor-typed_2.13</artifactId>
    <version>1.0.2</version>
  </dependency>
</dependencies>
```

**Mill `build.sc`**:
```scala
override def ivyDeps = Agg(
  ivy"org.apache.pekko::pekko-actor-typed:1.0.2",
)
```

**Key Differences**:
- `::` for Scala libraries (cross-version)
- `:` for Java libraries
- No `<scope>test</scope>` (use `testIvyDeps` instead)

---

### From Gradle

**Gradle `build.gradle`**:
```groovy
dependencies {
  implementation 'org.apache.pekko:pekko-actor-typed_2.13:1.0.2'
  testImplementation 'org.junit.jupiter:junit-jupiter-api:5.10.1'
}
```

**Mill `build.sc`**:
```scala
override def ivyDeps = Agg(
  ivy"org.apache.pekko::pekko-actor-typed:1.0.2",
)

override def testIvyDeps = Agg(
  ivy"org.junit.jupiter:junit-jupiter-api:5.10.1",
)
```

---

## Reference

### Common Mill Commands

| Command | Description |
|---------|-------------|
| `mill __.compile` | Compile all modules |
| `mill services.tenantManagement.compile` | Compile specific module |
| `mill __.test` | Run all tests |
| `mill services.tenantManagement.test` | Run tests for specific module |
| `mill --watch services.tenantManagement.test` | Watch mode (TDD) |
| `mill services.tenantManagement.run` | Run service |
| `mill services.tenantManagement.jar` | Create JAR |
| `mill services.tenantManagement.assembly` | Create executable JAR (fat JAR) |
| `mill clean` | Clean all outputs |
| `mill services.tenantManagement.clean` | Clean specific module |
| `mill show services.tenantManagement.ivyDeps` | Show dependencies |
| `mill resolve _` | List all targets |
| `mill version` | Show Mill version |

---

### Module Traits

| Trait | Purpose |
|-------|---------|
| `JavaModule` | Base Java module |
| `ScalaModule` | Scala module (if needed) |
| `TestModule` | Test module |
| `PublishModule` | Publishable module (Maven/Ivy) |

---

### Ivy Dependency Format

| Format | Example | Use Case |
|--------|---------|----------|
| `ivy"group:artifact:version"` | `ivy"org.postgresql:postgresql:42.7.1"` | Java library |
| `ivy"group::artifact:version"` | `ivy"org.apache.pekko::pekko-actor-typed:1.0.2"` | Scala library (cross-version) |
| `ivy"group:::artifact:version"` | `ivy"com.example:::my-app:1.0.0"` | Full cross-version |

---

## FAQ

### Q: Should I use Mill or Maven?

**A**: Use Mill for our projects. Benefits:
- 2-10x faster builds
- Better monorepo support (multiple bounded contexts)
- Type-safe configuration (Scala)
- Simpler mental model

---

### Q: How do I handle private Maven repositories?

**A**: Add custom repository:
```scala
override def repositoriesTask = T.task {
  super.repositoriesTask() ++ Seq(
    MavenRepository(
      "https://private.repo.com/maven",
      authentication = Some(Authentication("user", "pass"))
    ),
  )
}
```

---

### Q: Can I use Mill with Spring Boot?

**A**: No. We **do not use Spring Boot**. Use:
- Pekko actors for business logic
- Pekko HTTP for REST APIs
- Reactive Postgres for persistence

See `doc/exhibits/Java-Enterprise-Best-Practices-2.md` for rationale.

---

### Q: How do I debug Mill builds?

**A**: Use `--debug` flag:
```bash
mill --debug services.tenantManagement.compile
```

---

### Q: How do I publish artifacts?

**A**: Extend `PublishModule`:
```scala
object services extends Module {
  object tenantManagement extends BaseJavaModule with PublishModule {
    def publishVersion = "1.0.0"
    def pomSettings = PomSettings(
      description = "Tenant Management Service",
      organization = "com.example",
      url = "https://github.com/example/copilot-training",
      licenses = Seq(License.MIT),
      versionControl = VersionControl.github("example", "copilot-training"),
      developers = Seq(Developer("dev", "Developer", "https://github.com/dev"))
    )
  }
}
```

Publish:
```bash
mill services.tenantManagement.publish --sonatypeUri https://oss.sonatype.org/service/local \
  --sonatypeSnapshotUri https://oss.sonatype.org/content/repositories/snapshots \
  --credentials user:pass \
  --gpgArgs --passphrase=secret
```

---

## Related Documentation

- **Methodology**: `doc/internal/reference/SBPF/Blending-DDD-BDD-TDD.md`
- **Java Best Practices**: `doc/exhibits/Java-Enterprise-Best-Practices-2.md`
- **Team Playbook**: `HOW-WE-WORK.md`
- **Template Guide**: `doc/internal/reference/templates/CHARTER-CANVAS-GUIDE.md`

---

## External Resources

- **Mill Documentation**: https://mill-build.com/
- **Mill GitHub**: https://github.com/com-lihaoyi/mill
- **Mill Gitter Chat**: https://gitter.im/lihaoyi/mill
- **Pekko Documentation**: https://pekko.apache.org/docs/

---

## Summary

Mill is our build tool of choice for:
- **Fast incremental builds** (supports TDD red-green-refactor)
- **Monorepo structure** (multiple bounded contexts in one repository)
- **Type-safe configuration** (Scala, IDE support)
- **Simple mental model** (functional, composable tasks)

**Key Principles**:
1. **One bounded context = one Mill module**
2. **Share minimal domain artifacts** (value objects, events)
3. **Use watch mode for TDD** (`--watch` flag)
4. **Pin Mill version** (`.mill-version` file)
5. **Use millw wrapper** (no installation required for team)

**TDD Workflow**:
```bash
# Watch mode for rapid feedback
mill --watch services.tenantManagement.test

# Write test → Run → Implement → Run → Refactor → Run
```

**BDD Workflow**:
```bash
# Run BDD scenarios
mill features.test

# Implement with TDD, then rerun BDD
mill features.test
```

For questions, see the **Mill Gitter Chat** or reference **Mill Documentation**.
