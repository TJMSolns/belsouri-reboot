# SBPF-003: Testing Standards

**Standard**: Testing Conventions and Best Practices
**Version**: 1.0.0
**Status**: Active
**Last Updated**: 2026-01-06
**Owner**: Technical Lead

---

## 📋 Purpose

Define consistent naming conventions and best practices for unit tests, property-based tests, integration tests, and BDD scenarios in the open-commerce-search project.

**Goals**:
- Ensure test readability and maintainability
- Standardize naming across all test types
- Provide clear examples for developers
- Enable automated enforcement via ScalaFmt (where possible)

---

## 🎯 Test Naming Conventions

### 1. Unit Tests (MUnit FunSuite)

**Convention**: Use `test("ClassName should behavior")` format

**Pattern**:
```scala
test("ClassName should verb expected behavior") {
  // Arrange
  val instance = ClassName.create(...)

  // Act
  val result = instance.method(...)

  // Assert
  assert(result == expectedValue)
}
```

**Examples**:

✅ **GOOD**:
```scala
test("BetaParams should reject alpha < 1.0") {
  val result = BetaParams.create(alpha = 0.5, beta = 2.0)
  assert(result.isLeft, "Should reject alpha < 1.0")
}

test("ConvergenceTracker should add weight snapshots to sliding window") {
  val tracker = ConvergenceTracker.create(context)
  val tracker2 = tracker.addSnapshot(weights)
  assertEquals(tracker2.queryCount, 1)
}

test("ConfigurationEnsemble should emit AdaptationConverged event when convergence detected") {
  // ...
}
```

❌ **AVOID**:
```scala
test("test_alpha_validation") { ... }  // Non-descriptive, snake_case
test("BetaParams.create throws for invalid alpha") { ... }  // Use "should reject", not "throws"
test("Adding snapshots") { ... }  // Missing "should", unclear subject
```

**Rationale**:
- **Readability**: "should" clearly expresses expected behavior
- **Documentation**: Test names serve as living specification
- **Searchability**: Consistent format enables grep/search across codebase

---

### 2. Property-Based Tests (ScalaCheck)

**Convention**: Use `property("ClassName.method satisfies invariant")` format

**Pattern**:
```scala
property("ClassName.method satisfies invariant") {
  forAll(generator) { input =>
    val result = ClassName.method(input)
    invariant(result) // Boolean assertion
  }
}
```

**Examples**:

✅ **GOOD**:
```scala
property("BetaParams.update always increases alpha (for reward > 0)") {
  forAll(Gen.choose(0.01, 1.0)) { reward =>
    val params = BetaParams.default
    val updated = params.update(reward)
    updated.alpha > params.alpha
  }
}

property("ConvergenceTracker.addSnapshot is immutable") {
  forAll(weightsGen) { weights =>
    val tracker1 = ConvergenceTracker.create(context)
    val tracker2 = tracker1.addSnapshot(weights)
    (tracker1.queryCount == 0) && (tracker2.queryCount == 1)
  }
}

property("StepBandit.selectMethod returns method from availableMethods") {
  forAll(methodListGen) { methods =>
    val bandit = StepBandit.create("PREPROCESSING", methods)
    given Random = new Random(42)
    val (selected, _) = bandit.selectMethod(context, 0.05)
    methods.contains(selected)
  }
}
```

❌ **AVOID**:
```scala
property("alpha increases") { ... }  // Missing class/method context
property("BetaParams should update alpha") { ... }  // Use "satisfies", not "should" for properties
property("test update immutability") { ... }  // Non-descriptive
```

**Rationale**:
- **Clarity**: Properties express invariants, not behaviors
- **Distinction**: "satisfies" distinguishes properties from unit tests
- **Precision**: Include class/method name for traceability

---

### 3. Integration Tests

**Convention**: Use `test("System A successfully interacts with System B for use case")` format

**Pattern**:
```scala
test("System A successfully interacts with System B for use case") {
  // Setup
  val systemA = ...
  val systemB = ...

  // Execute integration
  val result = systemA.callSystemB(...)

  // Verify contract
  assert(result matches contract)
}
```

**Examples**:

✅ **GOOD**:
```scala
test("Search Context successfully queries ConfigurationEnsemble for configuration") {
  val ensemble = ConfigurationEnsemble.create(...)
  val (config, updatedEnsemble) = ensemble.selectConfiguration(context)

  // Verify configuration returned with three selected methods
  assert(preprocessingMethods.contains(config.preprocessor))
  assert(retrievalMethods.contains(config.retriever))
  assert(rankingMethods.contains(config.ranker))
}

test("ConfigurationEnsemble emits WeightsUpdated events consumed by Analytics Context") {
  val ensemble = ConfigurationEnsemble.create(...)
  val updatedEnsemble = ensemble.updateWeights(context, config, reward)

  val events = updatedEnsemble.uncommittedEvents.collect { case e: WeightsUpdated => e }
  assertEquals(events.length, 3, "Should emit 3 WeightsUpdated events")
}
```

❌ **AVOID**:
```scala
test("integration test") { ... }  // Non-descriptive
test("ConfigurationEnsemble works") { ... }  // Vague, no context boundary specified
```

**Rationale**:
- **Context Boundaries**: Explicitly names interacting systems
- **Use Case Clarity**: Describes integration scenario
- **ACL Validation**: Tests anti-corruption layer contracts

---

### 4. BDD Scenarios (Cucumber/Gherkin)

**Convention**: Use `Scenario: Actor performs action to achieve outcome` format

**Pattern**:
```gherkin
@tag @priority
Scenario: Actor performs action to achieve outcome
  Given initial state
  When action occurs
  Then expected outcome
  And additional verification
```

**Examples**:

✅ **GOOD**:
```gherkin
@P0 @convergence @detection
Scenario: Detect convergence when weights stabilize
  Given a ConfigurationEnsemble for tenant "electronicexpress"
  And context is (ShopIntent, RegisteredUser, Mobile)
  And 100 queries processed with stable weights
  When I check convergence status
  Then convergence should be TRUE
  And an "AdaptationConverged" event should be published

@P1 @thompson-sampling @exploration
Scenario: Epsilon-exploration selects random method with 5% probability
  Given a ConfigurationEnsemble with epsilon = 0.05
  When I select configurations 1000 times
  Then approximately 5% should be random selections
  And approximately 95% should be Thompson Sampling selections
```

❌ **AVOID**:
```gherkin
Scenario: Test convergence  # Non-descriptive
Scenario: ConfigurationEnsemble works correctly  # Vague
```

**Rationale**:
- **Business Language**: Uses domain terms from ubiquitous language
- **Actor-Centric**: Describes who/what performs action
- **Outcome-Focused**: Emphasizes business value, not implementation

---

## 🛠️ ScalaFmt Enforcement

**Status**: Partial (naming conventions cannot be fully enforced by ScalaFmt)

**What ScalaFmt CAN Enforce**:
- Code formatting within tests (indentation, spacing)
- Import organization
- Line length limits

**What ScalaFmt CANNOT Enforce**:
- Test name conventions (requires linter or code review)
- Test structure (Arrange-Act-Assert)
- Property vs unit test distinction

**Recommendation**: Use code review checklist to validate naming conventions.

---

## 📊 Code Review Checklist

### For Unit Tests
- [ ] Test name follows `test("ClassName should behavior")` format
- [ ] Test name is descriptive (> 5 words, clear intent)
- [ ] Uses "should" for behavior specification
- [ ] Includes class name for traceability

### For Property-Based Tests
- [ ] Property name follows `property("ClassName.method satisfies invariant")` format
- [ ] Uses "satisfies" or "always" for invariant expression
- [ ] Includes class and method name
- [ ] Clearly states invariant being tested

### For Integration Tests
- [ ] Test name specifies both interacting systems/contexts
- [ ] Describes integration use case
- [ ] Uses "successfully" for positive scenarios
- [ ] Validates ACL contract (no internal details leaked)

### For BDD Scenarios
- [ ] Scenario name follows `Actor performs action to achieve outcome` format
- [ ] Uses ubiquitous language terms
- [ ] Tagged appropriately (@P0, @P1, @feature-name)
- [ ] Given-When-Then structure maintained

---

## 📚 Examples from Codebase

### Unit Test Examples (Current Codebase)

**ConvergenceTrackerSpec.scala**:
```scala
test("ConvergenceTracker should be created with empty snapshot window") { ... }
test("ConvergenceTracker should add weight snapshots to sliding window") { ... }
test("ConvergenceTracker should detect convergence when variance < 0.1 for 100+ queries") { ... }
```

**ConfigurationEnsembleSpec.scala**:
```scala
test("ConfigurationEnsemble should be created with tenant ID and available methods") { ... }
test("ConfigurationEnsemble should update weights with equal reward splitting (ADR-009)") { ... }
test("ConfigurationEnsemble should emit AdaptationConverged event when convergence detected") { ... }
```

### Property-Based Test Examples (Current Codebase)

**ConvergenceTrackerProperties.scala**:
```scala
property("ConvergenceTracker.addSnapshot is immutable") { ... }
property("ConvergenceTracker.queryCount is monotonically increasing up to windowSize") { ... }
property("ConvergenceTracker.variance is always non-negative") { ... }
```

**BetaParamsProperties.scala**:
```scala
property("BetaParams.update always increases alpha (for reward > 0)") { ... }
property("BetaParams.alpha + BetaParams.beta is capped at 1000.0") { ... }
```

---

## 🔄 Migration Guide

### For Existing Tests

**If test follows convention**: ✅ No action needed

**If test violates convention**: Update test name during next edit (no bulk rename)

**Priority**:
1. **High**: Update when editing test functionality
2. **Medium**: Update when refactoring test file
3. **Low**: Bulk rename during major version bump (optional)

**Example Migration**:
```scala
// BEFORE
test("test_convergence_detection") { ... }

// AFTER (during next edit)
test("ConvergenceTracker should detect convergence when variance < 0.1 for 100+ queries") { ... }
```

---

## 📝 Version History

### Version 1.0.0 (2026-01-06)
- Initial standard definition
- Unit test naming: "should" convention
- Property test naming: "satisfies invariant" convention
- Integration test naming: "System A interacts with System B" convention
- BDD scenario naming: "Actor performs action" convention
- Code review checklist
- Examples from current codebase

---

## 🔗 References

- **PROJ-003-04**: Test Naming Standardization Story
- **MUnit Documentation**: https://scalameta.org/munit/
- **ScalaCheck Documentation**: https://scalacheck.org/
- **Cucumber/Gherkin**: https://cucumber.io/docs/gherkin/
- **BDD Scenarios**: [features/adaptive-ensemble.feature](../../features/adaptive-ensemble.feature)

---

**Standard Owner**: Technical Lead
**Review Cycle**: Annually (January)
**Next Review**: 2027-01-06
