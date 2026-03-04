# Anticorruption Layer Patterns

**Category:** Domain-Driven Design, Architecture Patterns
**Related:** DDD-Principles.md, Bounded Contexts, Pure Functional Domain
**Exhibit:** MDSlides project (Flexmark, Scalatags, Circe adapters)

---

## Overview

An **Anticorruption Layer** (ACL) is a translation layer that sits between your domain model and external systems (libraries, APIs, databases). It prevents external concepts from "corrupting" your domain model by translating between external representations and your ubiquitous language.

The ACL pattern is critical for maintaining a **pure functional domain** where business logic remains isolated from infrastructure concerns and external library dependencies.

---

## The Problem: Domain Contamination

### Without an Anticorruption Layer

```scala
// ❌ BAD: Domain contaminated with external library types
package com.example.mdslides.domain

import com.vladsch.flexmark.ast.Document  // External library type!

case class Slide(
  id: SlideId,
  templateName: String,
  flexmarkDocument: Document  // ❌ Domain knows about Flexmark
)

object Slide:
  def parse(markdown: String): Slide =
    val parser = Parser.builder().build()  // ❌ Domain does parsing
    val document = parser.parse(markdown)
    // ... extract slots from Flexmark AST
```

**Problems:**
1. **Domain depends on external library** (Flexmark)
2. **Domain logic mixed with parsing logic**
3. **Changing markdown library requires domain changes**
4. **Cannot test domain without external library**
5. **Ubiquitous language polluted** with technical terms

### With an Anticorruption Layer

```scala
// ✅ GOOD: Pure domain model
package com.example.mdslides.domain

case class Slide(
  id: SlideId,
  templateName: String,
  slots: Map[String, FormattedContent]  // Domain types only
)

case class FormattedContent(
  textSpans: List[TextSpan],
  links: List[Link],
  codeBlocks: List[CodeBlock]
)

// ✅ GOOD: ACL in infrastructure layer
package com.example.mdslides.infrastructure.parser

import com.vladsch.flexmark.ast.Document

object FlexmarkAdapter:
  def parseToSlides(markdown: String): Either[String, List[Slide]] =
    val parser = createFlexmarkParser()
    val document = parser.parse(markdown)
    mapFlexmarkToSlides(document)  // Translate Flexmark AST → Domain types

  private def mapFlexmarkToSlides(doc: Document): Either[String, List[Slide]] =
    // Map external Flexmark types → Internal domain types
    ???
```

**Benefits:**
1. **Domain is pure** - No external dependencies
2. **Separation of concerns** - Parsing in infrastructure, validation in domain
3. **Swappable implementations** - Can replace Flexmark without domain changes
4. **Testable domain** - No need for Flexmark in domain tests
5. **Preserved ubiquitous language** - Domain speaks business language

---

## When to Use an Anticorruption Layer

### Use ACL When Integrating With:

1. **External Libraries** (Flexmark, Jackson, AWS SDK)
2. **Third-Party APIs** (Stripe, Twilio, GitHub API)
3. **Legacy Systems** (Old database schema, monolith APIs)
4. **Different Bounded Contexts** (Other microservices)
5. **Infrastructure Services** (Message queues, object storage)

### Don't Use ACL For:

1. **Standard library** (List, Map, String) - These are universal
2. **Pure utility libraries** (Apache Commons, Cats) - If purely functional
3. **Your own bounded contexts** - If you control the model

### Rule of Thumb

Ask: **"If I replaced this external system, would my domain model need to change?"**
- **Yes** → Need ACL
- **No** → ACL not needed

---

## ACL Patterns

### Pattern 1: Adapter

**Use Case:** Wrap external library API with domain-friendly interface

**Example:** Flexmark Adapter (MDSlides)

```scala
// External library types (in infrastructure layer)
import com.vladsch.flexmark.ast._
import com.vladsch.flexmark.parser.Parser

// Domain types (in domain layer)
case class FormattedContent(
  textSpans: List[TextSpan],
  links: List[Link],
  codeBlocks: List[CodeBlock],
  images: List[ImageReference]
)

// ACL: Adapter in infrastructure layer
object FlexmarkAdapter:
  private val parser: Parser = Parser.builder()
    .extensions(java.util.Arrays.asList(
      TablesExtension.create(),
      StrikethroughExtension.create()
    ))
    .build()

  def parseToFormattedContent(markdown: String): Either[String, FormattedContent] =
    try
      val document = parser.parse(markdown)
      Right(mapToFormattedContent(document))
    catch
      case e: Exception => Left(s"Parse error: ${e.getMessage}")

  private def mapToFormattedContent(node: Node): FormattedContent =
    // Walk Flexmark AST, build FormattedContent
    val textSpans = extractTextSpans(node)
    val links = extractLinks(node)
    val codeBlocks = extractCodeBlocks(node)
    val images = extractImages(node)
    FormattedContent(textSpans, links, codeBlocks, images)

  private def extractTextSpans(node: Node): List[TextSpan] =
    // Map Flexmark StrongEmphasis → TextSpan(text, bold = true)
    // Map Flexmark Emphasis → TextSpan(text, italic = true)
    ???

  private def extractLinks(node: Node): List[Link] =
    // Map Flexmark Link → domain Link(url, text)
    ???
```

**Key Points:**
- External types never escape infrastructure layer
- Domain types are independent of Flexmark
- Can swap Flexmark for CommonMark without domain changes

---

### Pattern 2: Translator

**Use Case:** Convert between external and domain data structures

**Example:** Theme JSON Translator (MDSlides)

```scala
// External representation (Circe JSON)
import io.circe.{Decoder, Encoder}
import io.circe.generic.semiauto._

// Domain model
case class Theme(
  name: ThemeName,
  background: Background,
  colors: Colors,
  fonts: Fonts
)

case class Background(
  color: Color,
  image: Option[ImageUrl],
  opacity: Opacity
)

// ACL: JSON translator
object ThemeJsonAdapter:
  // External JSON structure
  private case class ThemeJson(
    name: String,
    background: BackgroundJson,
    colors: Map[String, String],
    fonts: Map[String, String]
  )

  private case class BackgroundJson(
    color: String,
    image: Option[String],
    opacity: Option[Double]
  )

  implicit val decoder: Decoder[ThemeJson] = deriveDecoder
  implicit val encoder: Encoder[ThemeJson] = deriveEncoder

  // Translate JSON → Domain
  def fromJson(json: String): Either[String, Theme] =
    import io.circe.parser._

    for {
      themeJson <- decode[ThemeJson](json).left.map(_.getMessage)
      theme <- translateToDomain(themeJson)
    } yield theme

  private def translateToDomain(json: ThemeJson): Either[String, Theme] =
    for {
      name <- ThemeName(json.name)
      background <- translateBackground(json.background)
      colors <- translateColors(json.colors)
      fonts <- translateFonts(json.fonts)
    } yield Theme(name, background, colors, fonts)

  private def translateBackground(json: BackgroundJson): Either[String, Background] =
    for {
      color <- Color.parse(json.background.color)
      image <- json.background.image.traverse(ImageUrl.parse)
      opacity <- Opacity(json.background.opacity.getOrElse(1.0))
    } yield Background(color, image, opacity)
```

**Key Points:**
- JSON structure separate from domain structure
- Validation happens during translation
- Domain types enforce invariants (Color, Opacity)

---

### Pattern 3: Facade

**Use Case:** Simplify complex external API for domain use

**Example:** HTML Renderer Facade (MDSlides)

```scala
// External library: Scalatags (complex API)
import scalatags.Text.all._

// Domain model
case class SlideDeck(slides: NonEmptyList[Slide])

// ACL: Facade
object HTMLRendererFacade:
  def renderDeck(deck: SlideDeck, theme: Theme): String =
    val slides = deck.slides.toList.zipWithIndex.map { case (slide, index) =>
      renderSlide(slide, index, deck.slideCount, theme)
    }

    val html = buildDocument(slides, theme)
    "<!DOCTYPE html>\n" + html.render

  private def buildDocument(slides: List[ConcreteHtmlTag[String]], theme: Theme): ConcreteHtmlTag[String] =
    html(
      buildHead(theme),
      body(
        div(cls := "slides")(slides),
        buildControls(slides.length),
        buildJavaScript()
      )
    )

  private def renderSlide(slide: Slide, index: Int, total: Int, theme: Theme): ConcreteHtmlTag[String] =
    slide.templateName match
      case "title" => renderTitleSlide(slide, index, total, theme)
      case "content" => renderContentSlide(slide, index, total, theme)
      case _ => div("Unknown template")

  private def renderTitleSlide(slide: Slide, index: Int, total: Int, theme: Theme): ConcreteHtmlTag[String] =
    // Extract domain data, render with Scalatags
    val title = slide.getSlot("title").getOrElse("")
    val subtitle = slide.getSlot("subtitle")
    val author = slide.getSlot("author")

    div(cls := "slide title-slide", attr("data-index") := index.toString)(
      h1(title),
      subtitle.map(s => h2(s)),
      author.map(a => p(cls := "author")(a))
    )
```

**Key Points:**
- Domain calls simple `renderDeck(deck, theme)` - no Scalatags knowledge needed
- Facade handles Scalatags complexity
- Theme application encapsulated

---

### Pattern 4: Gateway

**Use Case:** Interface to external services (APIs, databases)

**Example:** File System Gateway (MDSlides)

```scala
// Domain interface (in domain layer - abstract)
trait FileGateway:
  def readFile(path: FilePath): Either[String, FileContent]
  def writeFile(path: FilePath, content: FileContent): Either[String, Unit]

// Domain types
opaque type FilePath = String
opaque type FileContent = String

// ACL: Implementation in infrastructure layer
class FileSystemGateway extends FileGateway:
  import java.nio.file.{Files, Paths}

  override def readFile(path: FilePath): Either[String, FileContent] =
    try
      val content = Files.readString(Paths.get(path))
      Right(FileContent(content))
    catch
      case e: Exception => Left(s"Failed to read file: ${e.getMessage}")

  override def writeFile(path: FilePath, content: FileContent): Either[String, Unit] =
    try
      Files.writeString(Paths.get(path), content)
      Right(())
    catch
      case e: Exception => Left(s"Failed to write file: ${e.getMessage}")
```

**Key Points:**
- Domain defines interface (FileGateway)
- Infrastructure provides implementation
- Java NIO types never leak to domain
- Easy to mock for testing

---

## MDSlides ACL Examples

### ACL 1: Flexmark Markdown Parser

**External Library:** Flexmark (com.vladsch.flexmark)
**Location:** `infrastructure/parser/FlexmarkAdapter.scala`

**Domain Types Protected:**
- `FormattedContent`
- `TextSpan`
- `Link`
- `CodeBlock`
- `ImageReference`

**Translation:**
```
Flexmark Types          →  Domain Types
─────────────────────      ─────────────────
Document                →  FormattedContent
StrongEmphasis          →  TextSpan(bold = true)
Emphasis                →  TextSpan(italic = true)
Link                    →  Link(url, text)
FencedCodeBlock         →  CodeBlock(code, language)
Image                   →  ImageReference(url, altText)
```

---

### ACL 2: Scalatags HTML Generator

**External Library:** Scalatags (com.lihaoyi:scalatags)
**Location:** `infrastructure/renderer/HTMLRenderer.scala`

**Domain Types Protected:**
- `Slide`
- `SlideDeck`
- `Theme`

**Translation:**
```
Domain Types            →  Scalatags Types
────────────────           ─────────────────
Slide                   →  div(cls := "slide")
SlideDeck               →  html(...) document
Theme.colors.text       →  color: #333333 (CSS)
Background.image        →  background-image: url(...)
```

---

### ACL 3: Circe JSON Parser (Themes)

**External Library:** Circe (io.circe)
**Location:** `infrastructure/theme/ThemeLoader.scala`

**Domain Types Protected:**
- `Theme`
- `ThemeName`
- `Color`
- `ImageUrl`
- `Opacity`

**Translation:**
```
JSON                    →  Domain Types
────────────────           ─────────────────
{"name": "dark"}        →  ThemeName("dark")
{"color": "#1e1e1e"}    →  Color(0x1e1e1e)
{"opacity": 0.5}        →  Opacity(0.5)  // validated 0.0-1.0
```

---

## Testing Strategies

### Test Domain Without ACL

```scala
// Domain test - No external libraries needed
class SlideSpec extends munit.FunSuite:
  test("validate title slide with valid title") {
    val slide = Slide(SlideId.unsafe(1), "title", Map("title" -> "My Title"))
    val result = Slide.validated(slide.id, slide.templateName, slide.slots)

    assert(result.isRight)
  }
```

**No Flexmark, no Scalatags, no Circe - pure domain test**

### Test ACL Separately

```scala
// Infrastructure test - Test ACL translation
class FlexmarkAdapterSpec extends munit.FunSuite:
  test("parse bold text to TextSpan with bold=true") {
    val markdown = "**bold text**"
    val result = FlexmarkAdapter.parseToFormattedContent(markdown)

    result match
      case Right(content) =>
        assert(content.textSpans.exists(span => span.text == "bold text" && span.bold))
      case Left(error) => fail(s"Parse failed: $error")
  }
```

**Tests ACL translation logic in isolation**

---

## Common Pitfalls

### Pitfall 1: Leaking External Types

```scala
// ❌ BAD: External type leaks to domain
case class Slide(
  id: SlideId,
  content: com.vladsch.flexmark.ast.Document  // External type!
)
```

**Fix:** Translate to domain type in ACL

```scala
// ✅ GOOD: Domain type only
case class Slide(
  id: SlideId,
  content: FormattedContent  // Domain type
)
```

---

### Pitfall 2: Domain Calling External Library

```scala
// ❌ BAD: Domain imports external library
package com.example.mdslides.domain

import com.vladsch.flexmark.parser.Parser  // External import!

object Slide:
  def parse(markdown: String): Slide =
    val parser = Parser.builder().build()  // Domain does parsing!
    ???
```

**Fix:** Move to infrastructure layer

```scala
// ✅ GOOD: Infrastructure does parsing
package com.example.mdslides.infrastructure.parser

import com.vladsch.flexmark.parser.Parser

object MarkdownParser:
  def parse(markdown: String): Either[String, SlideDeck] =
    val parser = Parser.builder().build()
    // Parse and translate to domain types
    ???
```

---

### Pitfall 3: Incomplete Translation

```scala
// ❌ BAD: Partial translation
def translateLink(flexmarkLink: Link): domain.Link =
  domain.Link(
    url = flexmarkLink.getUrl.toString,
    text = flexmarkLink.getText.toString,
    flexmarkNode = flexmarkLink  // ❌ External type leaks!
  )
```

**Fix:** Complete translation

```scala
// ✅ GOOD: Complete translation
def translateLink(flexmarkLink: Link): domain.Link =
  domain.Link(
    url = flexmarkLink.getUrl.toString,
    text = flexmarkLink.getText.toString
    // No external types
  )
```

---

### Pitfall 4: Tight Coupling Through Shared Enums

```scala
// ❌ BAD: Domain uses external enum
import com.vladsch.flexmark.ast.NodeType

case class Slide(
  id: SlideId,
  nodeType: NodeType  // External enum!
)
```

**Fix:** Define domain enum

```scala
// ✅ GOOD: Domain enum
enum SlideType:
  case Title, Content, Section

case class Slide(
  id: SlideId,
  slideType: SlideType  // Domain enum
)

// ACL translates
def translateNodeType(nodeType: NodeType): SlideType =
  nodeType match
    case NodeType.TITLE => SlideType.Title
    case NodeType.CONTENT => SlideType.Content
    case _ => SlideType.Content
```

---

## Benefits of Anticorruption Layers

### 1. Domain Independence

**Without ACL:**
```scala
// Domain depends on 3 external libraries
import com.vladsch.flexmark._
import com.lihaoyi.scalatags._
import io.circe._
```

**With ACL:**
```scala
// Domain depends only on Cats (functional primitives)
import cats.data.NonEmptyList
```

---

### 2. Testability

**Without ACL:**
```scala
// Must set up Flexmark parser to test domain
test("validate slide") {
  val parser = Parser.builder().build()
  val doc = parser.parse("...")
  val slide = Slide.fromFlexmark(doc)  // Domain depends on Flexmark
  assert(Slide.validated(slide).isRight)
}
```

**With ACL:**
```scala
// Test domain directly
test("validate slide") {
  val slide = Slide(SlideId.unsafe(1), "title", Map("title" -> "Test"))
  assert(Slide.validated(slide.id, slide.templateName, slide.slots).isRight)
}
```

---

### 3. Flexibility

**Scenario:** Replace Flexmark with CommonMark

**Without ACL:**
- Change domain model (big impact)
- Change all domain tests
- Change all domain logic

**With ACL:**
- Change infrastructure adapter only
- Domain unchanged
- Domain tests unchanged

---

### 4. Clear Boundaries

ACL makes **bounded context boundaries explicit**:

```
┌─────────────────────────────────────┐
│         Domain Layer                │
│   (Pure, No External Dependencies)  │
│  - Slide, SlideDeck, Theme          │
│  - FormattedContent, Link, etc.     │
└─────────────────────────────────────┘
              ▲
              │ ACL (Translation)
              │
┌─────────────────────────────────────┐
│      Infrastructure Layer           │
│    (Adapters, External Systems)     │
│  - FlexmarkAdapter                  │
│  - ScalatagsRenderer                │
│  - CirceThemeLoader                 │
└─────────────────────────────────────┘
```

---

## Conclusion

The Anticorruption Layer is a critical pattern for maintaining domain purity in DDD applications. By translating between external representations and domain types, the ACL:

- **Protects domain** from external library changes
- **Enables testing** without infrastructure setup
- **Preserves ubiquitous language** in the domain
- **Allows flexibility** to swap implementations

**Key Principles:**
1. **External types never leak to domain**
2. **Translation happens in infrastructure layer**
3. **Domain defines interfaces, infrastructure implements**
4. **Test domain and ACL separately**
5. **Use adapters, translators, facades, and gateways**

**Start Today:** Identify one external library dependency in your domain and extract it behind an ACL!

---

## References

- [Domain-Driven Design (Eric Evans)](https://www.domainlanguage.com/ddd/) - Original ACL definition
- [DDD-Principles.md](DDD-Principles.md) - Bounded contexts and domain isolation
- [ADR-007: Anticorruption Layer](../../governance/ADR/ADR-007-anticorruption-layer.md) - MDSlides decision
- [MDSlides Infrastructure Layer](../../infrastructure/src/) - Real-world ACL examples

---

**Document Status:** Living Document (v1.0, 2024-12-21)
**Next Review:** After v0.2.0 Flexmark integration complete
**Maintainer:** Development Team
