# Property-Based Testing Implementation

**Category:** Testing Strategies
**Related:** TDD, FV-PBT.md, ScalaCheck, Domain-Driven Design
**Exhibit:** MDSlides project (29 property-based tests, custom generators)

---

## Overview

Property-Based Testing (PBT) complements example-based testing by **generating hundreds or thousands of test cases automatically** to verify that properties (invariants) hold across a wide range of inputs. Instead of writing specific examples, you describe **what should always be true** about your code.

This document provides **practical implementation guidance** for property-based testing using ScalaCheck in Scala 3, building on the theoretical foundations in [FV-PBT.md](FV-PBT.md).

---

## Example-Based vs Property-Based Testing

### Example-Based Testing (Traditional TDD)

```scala
test("title slide with valid title passes validation") {
  val slide = Slide(SlideId.unsafe(1), "title", Map("title" -> "My Title"))
  val result = Slide.validated(slide.id, slide.templateName, slide.slots)

  result match
    case Right(validSlide) => assertEquals(validSlide.getSlot("title"), Some("My Title"))
    case Left(errors) => fail(s"Expected success, got: ${errors}")
}
```

**Strengths:**
- Easy to understand
- Documents specific scenarios
- Fails with clear examples

**Weaknesses:**
- Only tests cases you thought of
- Misses edge cases
- Doesn't prove invariants hold generally

### Property-Based Testing

```scala
property("all validated title slides have required title slot") {
  forAll(validTitleSlideGen) { slide =>
    Slide.validated(slide.id, slide.templateName, slide.slots) match
      case Right(validSlide) => validSlide.hasSlot("title")
      case Left(_) => true  // Invalid slides can fail, that's OK
  }
}
```

**Strengths:**
- Tests hundreds/thousands of cases automatically
- Finds edge cases you didn't think of
- Proves invariants hold generally
- Shrinks failures to minimal examples

**Weaknesses:**
- Requires writing generators
- Can be harder to understand
- May need custom shrinking logic

---

## When to Use Property-Based Testing

### Use PBT When:

1. **Testing Invariants** - Properties that must always hold
   - "Validated slides never exceed max line count"
   - "Parsing then rendering preserves semantics"
   - "Encryption then decryption returns original"

2. **Testing Transformations** - Functions with clear mathematical properties
   - "reverse(reverse(list)) == list"
   - "sort(sort(list)) == sort(list)" (idempotence)
   - "encode(decode(x)) == x" (round-trip)

3. **Testing Domain Rules** - Business constraints
   - "Price must be positive"
   - "Username must be alphanumeric"
   - "File size must not exceed limit"

4. **Testing Parsers/Serializers** - Round-trip properties
   - "parse(serialize(x)) == x"
   - "fromJson(toJson(x)) == x"

### Use Example-Based Testing When:

1. **Documenting Specific Scenarios** - User stories, bug reproductions
2. **Testing Error Messages** - Exact wording matters
3. **Testing Side Effects** - Database writes, API calls
4. **Regression Tests** - Specific bugs that were found

### Best Practice: Use Both

MDSlides uses **29 property-based tests** + **32 example-based tests** = **61 domain tests**

- **Properties** verify invariants hold across thousands of inputs
- **Examples** document specific scenarios and edge cases

---

## ScalaCheck Basics

### Simple Property

```scala
import org.scalacheck.Prop.forAll
import org.scalacheck.Properties

object StringProperties extends Properties("String"):
  property("reverse twice is identity") = forAll { (s: String) =>
    s.reverse.reverse == s
  }
```

Run with: `mill domain.test`

Output:
```
+ String.reverse twice is identity: OK, passed 100 tests.
```

### With Generators

```scala
import org.scalacheck.Gen

val positiveIntGen: Gen[Int] = Gen.choose(1, 1000)

property("positive numbers are greater than zero") = forAll(positiveIntGen) { n =>
  n > 0
}
```

---

## Building Domain Generators

### Principle: Generators Mirror Domain Structure

If your domain has:
- **Value Objects** → Generator for each
- **Aggregates** → Compose value object generators
- **Constraints** → Filter or constrain generators

### Example: SlideId Generator (MDSlides)

```scala
// Domain: Value object with constraint
opaque type SlideId = Int
object SlideId:
  def apply(id: Int): Either[String, SlideId] =
    if id > 0 then Right(id) else Left("SlideId must be positive")

// Generator: Respects constraint
val slideIdGen: Gen[SlideId] =
  Gen.choose(1, 200).map(SlideId.unsafe)  // Always generates valid IDs
```

### Example: SlotContent Generator (MDSlides)

```scala
// Domain: Text with line/word counts
opaque type SlotContent = String
object SlotContent:
  extension (content: SlotContent)
    def lineCount: Int = content.split("\n").length
    def wordCount: Int = content.split("\\s+").length

// Generator: Realistic text content
val validBodyGen: Gen[String] = Gen.frequency(
  (40, textWithLines(5)),   // Most common: 5 lines
  (30, textWithLines(8)),   // Common: 8 lines
  (20, textWithLines(12)),  // Edge: max 12 lines
  (10, Gen.listOfN(100, Gen.alphaNumStr.suchThat(_.nonEmpty)).map(_.mkString(" ")))  // Edge: 100 words
)

def textWithLines(lines: Int): Gen[String] =
  Gen.listOfN(lines, Gen.alphaNumStr.suchThat(_.nonEmpty).map(_.take(40)))
    .map(_.mkString("\n"))
```

**Key Techniques:**
- `Gen.frequency` - Weight distribution (more common vs edge cases)
- `suchThat` - Filter to meet constraints
- `Gen.listOfN` - Generate specific number of elements
- Composition - Build complex generators from simple ones

---

## Generator Patterns

### Pattern 1: Constrained Values

```scala
// Generate even numbers
val evenGen: Gen[Int] = Gen.choose(0, 100).map(_ * 2)

// Generate non-empty strings
val nonEmptyStringGen: Gen[String] =
  Gen.alphaNumStr.suchThat(_.nonEmpty)

// Generate valid email addresses
val emailGen: Gen[String] = for {
  user <- Gen.alphaNumStr.suchThat(_.length > 0)
  domain <- Gen.oneOf("gmail.com", "example.com", "test.org")
} yield s"$user@$domain"
```

### Pattern 2: Frequency Distribution

```scala
// Generate mostly valid, some invalid
val slideGen: Gen[Slide] = Gen.frequency(
  (80, validSlideGen),    // 80% valid
  (15, invalidSlideGen),  // 15% invalid (missing slots)
  (5, edgeCaseSlideGen)   // 5% edge cases (max limits)
)
```

### Pattern 3: Composition

```scala
// Build aggregates from value objects
val titleSlideGen: Gen[Slide] = for {
  id <- slideIdGen
  title <- validTitleGen
  subtitle <- Gen.option(validSubtitleGen)
  author <- Gen.option(validAuthorGen)
} yield Slide(
  id,
  "title",
  Map("title" -> title) ++
    subtitle.map(s => Map("subtitle" -> s)).getOrElse(Map.empty) ++
    author.map(a => Map("author" -> a)).getOrElse(Map.empty)
)
```

### Pattern 4: Reusable Generators

```scala
object DomainGenerators:
  // Base generators
  val slideIdGen: Gen[SlideId] = ...
  val validTitleGen: Gen[String] = ...
  val validBodyGen: Gen[String] = ...

  // Composite generators
  val validTitleSlideGen: Gen[Slide] = ...
  val validContentSlideGen: Gen[Slide] = ...
  val validSlideDeckGen: Gen[SlideDeck] = ...
```

**File:** `domain/test/src/.../generators/DomainGenerators.scala`

---

## Writing Effective Properties

### Property 1: Invariants

**Pattern:** "Validated X never violates constraint Y"

```scala
property("validated body never exceeds 12 lines") {
  forAll(validContentSlideGen) { slide =>
    Slide.validated(slide.id, slide.templateName, slide.slots) match
      case Right(validSlide) =>
        validSlide.getSlot("body").map(body =>
          SlotContent(body).lineCount <= 12
        ).getOrElse(false)
      case Left(_) => true  // Invalid slides can fail validation
  }
}
```

### Property 2: Round-Trip

**Pattern:** "parse(serialize(x)) == x"

```scala
property("rendering then parsing preserves slide count") {
  forAll(validSlideDeckGen) { deck =>
    val html = HTMLRenderer.renderDeck(deck)
    val parsed = MarkdownParser.parse(html)
    parsed.map(_.slideCount) == Right(deck.slideCount)
  }
}
```

### Property 3: Idempotence

**Pattern:** "f(f(x)) == f(x)"

```scala
property("validating twice is same as validating once") {
  forAll(slideGen) { slide =>
    val once = Slide.validated(slide.id, slide.templateName, slide.slots)
    val twice = once.flatMap(s => Slide.validated(s.id, s.templateName, s.slots))
    once == twice
  }
}
```

### Property 4: Commutativity

**Pattern:** "f(g(x)) == g(f(x))"

```scala
property("line count is same for trimmed or untrimmed text") {
  forAll(Gen.alphaNumStr) { text =>
    SlotContent(text).lineCount == SlotContent(text.trim).lineCount
  }
}
```

### Property 5: Relationships

**Pattern:** "If X then Y"

```scala
property("slides with missing required slots fail validation") {
  forAll(slideWithMissingSlotGen) { slide =>
    Slide.validated(slide.id, slide.templateName, slide.slots).isLeft
  }
}
```

---

## Handling Edge Cases

### Include Edge Cases in Generators

```scala
val bodyGen: Gen[String] = Gen.frequency(
  (40, textWithLines(5)),       // Normal case
  (30, textWithLines(8)),       // Normal case
  (15, textWithLines(12)),      // Boundary: exactly max
  (10, textWithLines(13)),      // Boundary: one over max
  (3, Gen.const("")),           // Edge: empty
  (2, Gen.const("\n\n\n"))      // Edge: only newlines
)
```

### Test Boundaries Explicitly

```scala
property("content slide with exactly 12 lines passes validation") {
  forAll(slideIdGen) { id =>
    val body = List.fill(12)("line").mkString("\n")
    val slide = Slide(id, "content", Map("heading" -> "Test", "body" -> body))
    Slide.validated(slide.id, slide.templateName, slide.slots).isRight
  }
}

property("content slide with 13 lines fails validation") {
  forAll(slideIdGen) { id =>
    val body = List.fill(13)("line").mkString("\n")
    val slide = Slide(id, "content", Map("heading" -> "Test", "body" -> body))
    Slide.validated(slide.id, slide.templateName, slide.slots).isLeft
  }
}
```

---

## Shrinking

When a property fails, ScalaCheck **shrinks** the input to find the **minimal failing case**.

### Default Shrinking

```scala
property("all numbers are less than 100") = forAll { (n: Int) =>
  n < 100
}
```

**Failure:**
```
! all numbers are less than 100: Falsified after 5 passed tests.
> ARG_0: 100
> ARG_0_ORIGINAL: 2147483647
```

ScalaCheck shrunk `2147483647` → `100` (minimal failure).

### Custom Shrinking

```scala
case class Email(user: String, domain: String):
  override def toString = s"$user@$domain"

implicit val emailShrink: Shrink[Email] = Shrink { email =>
  // Shrink user part
  Shrink.shrink(email.user).map(u => Email(u, email.domain)) append
  // Shrink domain part
  Shrink.shrink(email.domain).map(d => Email(email.user, d))
}
```

---

## Integration with MUnit

MDSlides uses **MUnit with ScalaCheck integration**:

```scala
// build.sc
object test extends ScalaTests with TestModule.Munit {
  override def ivyDeps = Agg(
    ivy"org.scalameta::munit:0.7.29",
    ivy"org.scalameta::munit-scalacheck:0.7.29",
    ivy"org.scalacheck::scalacheck:1.17.0"
  )
}
```

### Test Suite Structure

```scala
package com.example.mdslides.domain.properties

import munit.ScalaCheckSuite
import org.scalacheck.Prop.forAll

class SlideProperties extends ScalaCheckSuite:
  import DomainGenerators.*

  property("all validated slides have required slots") {
    forAll(validTitleSlideGen) { slide =>
      Slide.validated(slide.id, slide.templateName, slide.slots) match
        case Right(validSlide) => validSlide.hasSlot("title")
        case Left(_) => true
    }
  }
```

Run tests:
```bash
mill domain.test
```

Output:
```
+ SlideProperties.all validated slides have required slots: OK, passed 100 tests.
```

---

## Organization Patterns

### File Structure (MDSlides)

```
domain/
└── test/
    └── src/
        └── com/example/mdslides/domain/
            ├── generators/
            │   └── DomainGenerators.scala     # Reusable generators
            ├── properties/
            │   ├── SlideProperties.scala       # Property tests for Slide
            │   ├── ContentSlideProperties.scala
            │   └── SlideDeckProperties.scala
            └── SlideSpec.scala                 # Example-based tests
```

### Naming Conventions

- **Generators:** `domain/test/.../generators/DomainGenerators.scala`
- **Properties:** `domain/test/.../properties/EntityProperties.scala`
- **Examples:** `domain/test/.../EntitySpec.scala`

### Test Counts (MDSlides v0.1.0)

- **29 property-based tests** (in `properties/` files)
- **32 example-based tests** (in `*Spec.scala` files)
- **Total: 61 domain tests**

---

## Common Patterns from MDSlides

### Pattern 1: Valid vs Invalid Generators

```scala
object DomainGenerators:
  // Generates only valid slides
  val validContentSlideGen: Gen[Slide] = for {
    id <- slideIdGen
    heading <- validHeadingGen  // Max 80 chars, single line
    body <- validBodyGen        // Max 12 lines, 150 words
  } yield Slide(id, "content", Map("heading" -> heading, "body" -> body))

  // Generates invalid slides (for negative testing)
  val invalidContentSlideGen: Gen[Slide] = Gen.oneOf(
    contentSlideWithMissingHeadingGen,
    contentSlideWithMissingBodyGen,
    contentSlideWithTooManyLinesGen,
    contentSlideWithTooManyWordsGen
  )
```

### Pattern 2: Frequency-Weighted Edge Cases

```scala
val bodyGen: Gen[String] = Gen.frequency(
  (40, textWithLines(5)),    // 40% normal case
  (30, textWithLines(8)),    // 30% normal case
  (20, textWithLines(12)),   // 20% boundary (exactly max)
  (10, textWithLines(100))   // 10% edge case (way over)
)
```

### Pattern 3: Composition and Reuse

```scala
// Base generators
val validTitleGen: Gen[String] = ...
val validSubtitleGen: Gen[String] = ...

// Composed generator
val validTitleSlideGen: Gen[Slide] = for {
  id <- slideIdGen
  title <- validTitleGen
  subtitle <- Gen.option(validSubtitleGen)  // Optional
} yield Slide(id, "title", Map(...))
```

---

## Debugging Failed Properties

### 1. Enable Verbose Output

```scala
property("my property").verbose {
  forAll(myGen) { x =>
    // test
  }
}
```

### 2. Print Intermediate Values

```scala
property("my property") {
  forAll(myGen) { x =>
    val intermediate = compute(x)
    println(s"Input: $x, Intermediate: $intermediate")
    assert(intermediate < 100)
  }
}
```

### 3. Use Smaller Generators

```scala
// Instead of Gen.choose(1, 1000000)
val smallGen = Gen.choose(1, 100)  // Easier to debug
```

### 4. Reproduce Specific Failure

```scala
test("reproduce specific failure") {
  val failingInput = ...  // From shrunk output
  assert(myProperty(failingInput))
}
```

---

## Performance Considerations

### Test Count Configuration

```scala
property("my expensive property") {
  forAll(Gen.choose(1, 100).withMinSuccessfulTests(10)) { n =>
    // Only run 10 tests instead of default 100
    expensiveComputation(n)
  }
}
```

### Generator Performance

**Slow:**
```scala
val badGen: Gen[String] = Gen.alphaNumStr.suchThat(s =>
  s.length > 10 && s.contains("test") && s.startsWith("a")
)  // May retry many times
```

**Fast:**
```scala
val goodGen: Gen[String] = for {
  prefix <- Gen.const("a")
  middle <- Gen.const("test")
  suffix <- Gen.alphaNumStr.map(_.take(6))
} yield s"$prefix$middle$suffix"  // Always succeeds
```

---

## Best Practices Summary

1. **Start with Simple Properties** - "validated X never exceeds limit Y"
2. **Build Generators Incrementally** - Start with basic, compose into complex
3. **Use Frequency Weighting** - Most common cases + edge cases
4. **Test Invariants, Not Specifics** - Properties should be general
5. **Combine with Example Tests** - Use both property-based and example-based
6. **Organize Generators** - Separate file, reuse across tests
7. **Keep Properties Fast** - Aim for <1s per property suite
8. **Shrink Failures** - Let ScalaCheck find minimal failing cases
9. **Document Generators** - Explain constraints and distributions
10. **Review Generated Values** - Occasionally print to verify distribution

---

## Conclusion

Property-Based Testing is a powerful complement to example-based testing, automatically generating thousands of test cases to verify invariants. By building domain-specific generators and writing properties that describe "what should always be true," you can catch edge cases that manual testing would miss.

**Key Takeaways:**
- PBT verifies **invariants hold generally**, not just for specific examples
- Generators should **mirror domain structure** (value objects → aggregates)
- Use **frequency weighting** to balance common cases and edge cases
- **Combine PBT with example-based tests** for comprehensive coverage
- **Organize generators** in a reusable module
- **Shrinking finds minimal failures** automatically

**Start Today:** Pick one domain invariant and write your first property test!

---

## References

- [FV-PBT.md](FV-PBT.md) - Theoretical foundations
- [ScalaCheck User Guide](https://github.com/typelevel/scalacheck/blob/main/doc/UserGuide.md)
- [MDSlides Domain Generators](../../domain/test/src/com/example/mdslides/domain/generators/)
- [MDSlides Property Tests](../../domain/test/src/com/example/mdslides/domain/properties/)

---

**Document Status:** Living Document (v1.0, 2024-12-21)
**Next Review:** After v0.2.0 release
**Maintainer:** Development Team
