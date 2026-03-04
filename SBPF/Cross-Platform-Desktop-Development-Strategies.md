# Cross-Platform Desktop Development Strategies
## Technology Stack Decisions for Windows/Mac/Linux Healthcare Applications

**Purpose**: Evaluate and recommend technology stacks for developing cross-platform desktop healthcare applications, with specific focus on Windows-first, Mac-second, Linux-third deployment strategy.

**Context**: Caribbean dental practice platform requiring native desktop performance, offline-first operation, and deployment across diverse hardware configurations.

**Key Constraint**: Must work on aging Windows hardware common in Caribbean clinics while providing modern UX on newer Mac/Linux systems.

---

## 🎯 Technology Stack Evaluation Matrix

### Evaluation Criteria for Caribbean Healthcare Context

| Criteria | Weight | Rationale |
|----------|--------|-----------|
| **Windows Performance** | 🔥🔥🔥 | Primary market, often aging hardware |
| **Offline Capability** | 🔥🔥🔥 | Must work without internet for days/weeks |
| **Development Velocity** | 🔥🔥 | Small team, limited resources |
| **Mac/Linux Portability** | 🔥🔥 | Secondary markets, but important for expansion |
| **Medical Device Integration** | 🔥🔥 | Intraoral cameras, X-ray systems, printers |
| **Deployment Complexity** | 🔥 | Limited IT support in target market |
| **Memory/CPU Efficiency** | 🔥🔥 | Old hardware constraints |
| **Security & Compliance** | 🔥🔥🔥 | Healthcare data protection requirements |

---

## 🛠️ Technology Stack Options Analysis

### Option 1: Java/Scala + JavaFX (Recommended for Caribbean Context)

**Architecture**:
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   JavaFX UI     │    │   Business       │    │   Local         │
│   (Cross-       │────│   Logic          │────│   Database      │
│   Platform)     │    │   (Scala 3)      │    │   (H2/SQLite)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

**Pros**:
- ✅ **Excellent Windows performance** - JVM optimized for Windows
- ✅ **True cross-platform** - Write once, run anywhere
- ✅ **Mature ecosystem** - Libraries for medical device integration
- ✅ **Strong offline support** - No web dependencies
- ✅ **Good hardware compatibility** - Runs on older JVMs
- ✅ **Enterprise security** - Java security model well-understood
- ✅ **Database integration** - Native support for H2, SQLite, PostgreSQL
- ✅ **GraalVM Native Image** - Compile to native executable (60-80MB, <1s startup)

**Cons**:
- ❌ **Larger deployment size (JVM)** - JVM + application (300-400MB) - **SOLVED with GraalVM Native Image (60-80MB)**
- ❌ **Slower startup time (JVM)** - JVM initialization overhead (3-5s) - **SOLVED with Native Image (<1s)**
- ❌ **Modern UI challenges** - JavaFX less modern than web technologies
- ❌ **Limited native integration** - Requires JNI for some OS features
- ❌ **Longer build times (Native Image)** - Native compilation adds 2-5 minutes per platform

**Caribbean Suitability**: ⭐⭐⭐⭐⭐ (Excellent - **especially with GraalVM Native Image for bandwidth-constrained deployments**)

**Implementation Example**:
```scala
// Main Application Architecture with GraalVM Native Image Support
class DentalPracticeApp extends Application {
  
  override def start(primaryStage: Stage): Unit = {
    // Initialize offline-first architecture
    val eventStore = new H2EventStore("dental_practice.db")
    val patientService = new PatientService(eventStore)
    val syncService = new BackgroundSyncService(eventStore)
    
    // Load main UI
    val fxmlLoader = new FXMLLoader(getClass.getResource("/fxml/main-window.fxml"))
    val mainController = new MainController(patientService, syncService)
    fxmlLoader.setController(mainController)
    
    val scene = new Scene(fxmlLoader.load())
    primaryStage.setTitle("Caribbean Dental Practice Management")
    primaryStage.setScene(scene)
    primaryStage.show()
    
    // Start background services
    syncService.startBackgroundSync()
  }
}

// GraalVM Native Image build configuration (Mill)
// build.sc
object caribbeanDental extends ScalaModule {
  def scalaVersion = "3.3.1"
  
  def ivyDeps = Agg(
    ivy"org.openjfx:javafx-controls:21.0.1",
    ivy"org.openjfx:javafx-fxml:21.0.1",
    ivy"com.h2database:h2:2.2.224",
    ivy"org.apache.pekko::pekko-actor-typed:1.0.2",
    ivy"net.java.dev.jna:jna:5.14.0",
    ivy"net.java.dev.jna:jna-platform:5.14.0"
  )
  
  // Native Image compilation for each platform
  def nativeImageWindows = T {
    buildNativeImage("windows", "caribbean-dental.exe")
  }
  
  def nativeImageMacOS = T {
    buildNativeImage("macos", "caribbean-dental")
  }
  
  def nativeImageLinux = T {
    buildNativeImage("linux", "caribbean-dental")
  }
  
  private def buildNativeImage(platform: String, outputName: String) = T {
    val jar = assembly().path
    
    os.proc(
      "native-image",
      "--no-fallback",
      "-H:+ReportExceptionStackTraces",
      "--enable-http",
      "--enable-https",
      "--initialize-at-build-time=scala,pekko,javafx",
      "--initialize-at-run-time=com.sun.javafx.application.PlatformImpl",
      s"-H:ReflectionConfigurationFiles=${graalvmConfig().path}/reflect-config.json",
      s"-H:ResourceConfigurationFiles=${graalvmConfig().path}/resource-config.json",
      s"-H:JNIConfigurationFiles=${graalvmConfig().path}/jni-config.json",
      "-H:+AddAllCharsets",
      "-H:IncludeResources=.*\\.properties$",
      "-H:IncludeResources=.*\\.fxml$",
      "-H:IncludeResources=.*\\.css$",
      s"-jar", jar,
      outputName
    ).call(cwd = T.dest)
    
    PathRef(T.dest / outputName)
  }
  
  def graalvmConfig = T.source { millSourcePath / "graalvm" }
}
```

**GraalVM Reflection Configuration** (`graalvm/reflect-config.json`):
```json
[
  {
    "name": "javafx.application.Application",
    "allDeclaredMethods": true,
    "allDeclaredConstructors": true
  },
  {
    "name": "javafx.fxml.FXMLLoader",
    "allDeclaredMethods": true
  },
  {
    "name": "com.sun.jna.Native",
    "allDeclaredMethods": true,
    "allDeclaredConstructors": true
  },
  {
    "name": "com.sun.jna.Structure",
    "allDeclaredMethods": true
  }
]
```

**Deployment Size Comparison**:
- **With JVM**: 300-400 MB (JRE + application)
- **GraalVM Native Image**: 60-80 MB (single executable)
- **Savings**: ~75% reduction - critical for Caribbean bandwidth constraints

// Native integration via JNI when needed
class WindowsIntegration {
  @native def connectToIntraoralCamera(): Boolean
  @native def printToLabelPrinter(data: Array[Byte]): Boolean
  
  // Load native library
  System.loadLibrary("dental_native_windows")
}

// Deployment configuration
// build.sbt
lazy val dental = project.in(file("."))
  .enablePlugins(JavaAppPackaging, JlinkPlugin)
  .settings(
    name := "dental-practice-mgmt",
    scalaVersion := "3.3.1",
    libraryDependencies ++= Seq(
      "org.openjfx" % "javafx-controls" % "17.0.2",
      "org.openjfx" % "javafx-fxml" % "17.0.2",
      "com.h2database" % "h2" % "2.2.224",
      "com.typesafe.akka" %% "akka-actor-typed" % "2.8.5"
    ),
    
    // Windows MSI generation
    Windows / packageBin := {
      val msi = (Windows / packageBin).value
      // Sign MSI for Windows SmartScreen bypass
      Process(s"signtool sign /f certificate.pfx ${msi.getAbsolutePath}").!
      msi
    }
  )
```

### Option 2: Electron + TypeScript/React

**Architecture**:
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   React UI      │    │   Main Process   │    │   SQLite/       │
│   (Renderer)    │────│   (Node.js)      │────│   File System   │
│                 │    │                  │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

**Pros**:
- ✅ **Rapid development** - Web technologies, large talent pool
- ✅ **Modern UI** - Rich component libraries, responsive design
- ✅ **Cross-platform by default** - Chromium + Node.js everywhere
- ✅ **Great developer experience** - Hot reload, debugging tools
- ✅ **Offline support** - Service workers, local storage

**Cons**:
- ❌ **High memory usage** - Chromium overhead (200MB+ base)
- ❌ **Poor performance on old hardware** - Caribbean constraint
- ❌ **Large deployment size** - 300MB+ for basic app
- ❌ **Security complexity** - Browser security model in desktop context
- ❌ **Native integration complexity** - Requires node-ffi or custom modules

**Caribbean Suitability**: ⭐⭐⭐ (Fair - memory constraints problematic)

**Implementation Example**:
```typescript
// Main process (Node.js)
import { app, BrowserWindow, ipcMain } from 'electron';
import { PatientDatabase } from './database/patient-db';
import { SyncService } from './sync/sync-service';

class DentalApp {
  private mainWindow: BrowserWindow;
  private patientDb: PatientDatabase;
  private syncService: SyncService;

  constructor() {
    this.patientDb = new PatientDatabase('./data/patients.db');
    this.syncService = new SyncService(this.patientDb);
  }

  createWindow(): void {
    this.mainWindow = new BrowserWindow({
      width: 1200,
      height: 800,
      webPreferences: {
        nodeIntegration: false,  // Security best practice
        contextIsolation: true,
        preload: path.join(__dirname, 'preload.js')
      }
    });

    this.mainWindow.loadFile('dist/index.html');
    this.setupIPCHandlers();
  }

  private setupIPCHandlers(): void {
    ipcMain.handle('get-patient', async (event, patientId: string) => {
      return await this.patientDb.findById(patientId);
    });

    ipcMain.handle('save-patient', async (event, patient: Patient) => {
      return await this.patientDb.save(patient);
    });
  }
}

// Renderer process (React)
import React, { useState, useEffect } from 'react';
import { PatientForm } from './components/PatientForm';

const App: React.FC = () => {
  const [patients, setPatients] = useState<Patient[]>([]);
  const [syncStatus, setSyncStatus] = useState<SyncStatus>('offline');

  useEffect(() => {
    // Load initial patient data
    window.electronAPI.getAllPatients().then(setPatients);
    
    // Monitor sync status
    window.electronAPI.onSyncStatusChanged(setSyncStatus);
  }, []);

  return (
    <div className="app">
      <header>
        <h1>Dental Practice Management</h1>
        <SyncIndicator status={syncStatus} />
      </header>
      <main>
        <PatientList patients={patients} />
        <PatientForm />
      </main>
    </div>
  );
};
```

### Option 3: .NET MAUI (Windows-First Alternative)

**Architecture**:
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   MAUI UI       │    │   Business       │    │   SQLite/       │
│   (Platform     │────│   Logic          │────│   SQL Server    │
│   Specific)     │    │   (.NET 8)       │    │   LocalDB       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

**Pros**:
- ✅ **Excellent Windows integration** - Native Windows APIs, performance
- ✅ **Enterprise-grade tooling** - Visual Studio, debugging, profiling
- ✅ **Strong database support** - Entity Framework, SQL Server LocalDB
- ✅ **Good hardware efficiency** - Native compilation options
- ✅ **Healthcare ecosystem** - Many medical device SDKs support .NET

**Cons**:
- ❌ **Limited Mac/Linux support** - MAUI Mac support newer, Linux experimental
- ❌ **Microsoft ecosystem lock-in** - Licensing, deployment dependencies
- ❌ **Development team skills** - May require .NET expertise
- ❌ **Deployment complexity** - .NET runtime requirements on target machines

**Caribbean Suitability**: ⭐⭐⭐⭐ (Good for Windows-heavy approach)

**Implementation Example**:
```csharp
// Main Application
namespace DentalPractice.MAUI
{
    public partial class MainPage : ContentPage
    {
        private readonly IPatientService patientService;
        private readonly ISyncService syncService;

        public MainPage(IPatientService patientService, ISyncService syncService)
        {
            InitializeComponent();
            this.patientService = patientService;
            this.syncService = syncService;
        }

        protected override async void OnAppearing()
        {
            base.OnAppearing();
            await LoadPatients();
            
            // Start background sync
            _ = Task.Run(async () => await syncService.StartBackgroundSync());
        }

        private async Task LoadPatients()
        {
            var patients = await patientService.GetAllPatientsAsync();
            PatientCollectionView.ItemsSource = patients;
        }
    }

    // Business Logic - Shared across platforms
    public class PatientService : IPatientService
    {
        private readonly IEventStore eventStore;
        private readonly ILogger<PatientService> logger;

        public PatientService(IEventStore eventStore, ILogger<PatientService> logger)
        {
            this.eventStore = eventStore;
            this.logger = logger;
        }

        public async Task<Patient?> GetPatientAsync(Guid patientId)
        {
            var events = await eventStore.GetEventsAsync(patientId);
            return Patient.FromEvents(events);
        }

        public async Task SavePatientAsync(Patient patient)
        {
            var events = patient.GetUncommittedEvents();
            await eventStore.SaveEventsAsync(patient.Id, events);
            patient.MarkEventsAsCommitted();
        }
    }

    // Platform-specific database implementation
    public class WindowsEventStore : IEventStore
    {
        private readonly string connectionString;

        public WindowsEventStore(IConfiguration configuration)
        {
            // Use SQL Server LocalDB for Windows
            connectionString = configuration.GetConnectionString("LocalDB");
        }

        public async Task SaveEventsAsync(Guid aggregateId, IEnumerable<IDomainEvent> events)
        {
            using var connection = new SqlConnection(connectionString);
            // ... SQL Server implementation
        }
    }
}

// Dependency injection setup
public static class MauiProgram
{
    public static MauiApp CreateMauiApp()
    {
        var builder = MauiApp.CreateBuilder();
        builder
            .UseMauiApp<App>()
            .ConfigureFonts(fonts => fonts.AddFont("OpenSans-Regular.ttf", "OpenSansRegular"));

        // Register platform-specific services
        builder.Services.AddSingleton<IEventStore, WindowsEventStore>();
        builder.Services.AddSingleton<IPatientService, PatientService>();
        builder.Services.AddSingleton<ISyncService, SyncService>();

        return builder.Build();
    }
}
```

### Option 4: Tauri + Rust + React

**Architecture**:
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   React UI      │    │   Rust Backend   │    │   SQLite/       │
│   (WebView)     │────│   (Native)       │────│   RocksDB       │
│                 │    │                  │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

**Pros**:
- ✅ **Small binary size** - Rust efficiency, no Chromium bundled
- ✅ **Excellent performance** - Rust backend, system WebView
- ✅ **Modern security model** - Rust memory safety
- ✅ **Cross-platform** - Single codebase for all platforms
- ✅ **Fast development** - Web frontend, native backend

**Cons**:
- ❌ **Newer technology** - Smaller ecosystem, fewer resources
- ❌ **WebView dependencies** - Different WebView versions across platforms
- ❌ **Learning curve** - Rust expertise required for backend
- ❌ **Limited medical device SDKs** - Most are C/C++/C#, not Rust

**Caribbean Suitability**: ⭐⭐⭐ (Promising but newer technology risk)

---

## 🎯 Recommended Technology Stack Decision Matrix

### Phase 1: Windows-First Deployment

**Recommendation**: **Java/Scala + JavaFX**

**Rationale**:
1. **Proven Windows performance** on aging hardware
2. **Strong offline capabilities** with local H2 database
3. **Excellent medical device integration** via JNI
4. **Cross-platform ready** for Mac/Linux expansion
5. **Team expertise alignment** with existing Scala/Mill build system

**Architecture**:
```scala
// Project structure
dental-practice-desktop/
├── build.sbt                           // SBT build with JavaFX
├── src/
│   ├── main/
│   │   ├── scala/
│   │   │   ├── com/caribbeandental/
│   │   │   │   ├── ui/                 // JavaFX controllers
│   │   │   │   ├── domain/             // Business logic (DDD)
│   │   │   │   ├── infrastructure/     // Database, file system
│   │   │   │   └── integration/        // Medical device APIs
│   │   │   └── resources/
│   │   │       ├── fxml/               // JavaFX layouts
│   │   │       └── css/                // Styling
│   │   └── native/
│   │       └── windows/                // JNI libraries for Windows
│   └── test/
└── deployment/
    ├── windows/                        // MSI packaging
    ├── mac/                            // DMG packaging (Phase 2)
    └── linux/                          // AppImage/DEB (Phase 3)
```

### Phase 2: Mac Expansion Strategy

**Options for Mac deployment**:

1. **Continue JavaFX** (Recommended for consistency)
   - Same codebase, different packaging
   - DMG distribution via `sbt-native-packager`
   - Mac App Store submission possible with some effort

2. **Native Mac App** (If JavaFX limitations found)
   - SwiftUI + Core Data for native Mac experience  
   - Shared business logic via Scala Native or JNI bridge
   - Better Mac integration (Touch Bar, Services, etc.)

### Phase 3: Linux Strategy

**Recommended approach**:
1. **JavaFX Linux** - Same codebase as Windows/Mac
2. **AppImage packaging** - Single file deployment
3. **Flatpak distribution** - Linux app store integration
4. **Debian/RPM packages** - Traditional Linux package managers

---

## 🚀 Deployment Strategy Per Platform

### Windows Deployment (Phase 1)

**Target Distribution**:
- **MSI Installer** - Group Policy deployment in enterprise
- **Portable EXE** - USB stick deployment for remote clinics
- **Windows Store** - Future consideration for easier updates

**Implementation**:
```sbt
// build.sbt Windows configuration
lazy val windowsSettings = Seq(
  Windows / packageName := "Caribbean-Dental-Practice",
  Windows / packageSummary := "Offline-first dental practice management",
  Windows / packageDescription := "Electronic health records for Caribbean dental practices",
  
  // MSI generation with WiX
  Windows / wixProductId := java.util.UUID.randomUUID().toString,
  Windows / wixProductUpgradeId := "12345678-1234-1234-1234-123456789012",
  
  // Code signing for Windows SmartScreen
  Windows / packageBin := {
    val msi = (Windows / packageBin).value
    val signTool = "C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.19041.0\\x64\\signtool.exe"
    Process(s"${signTool} sign /f certificate.pfx /p ${certificatePassword} ${msi.getAbsolutePath}").!
    msi
  }
)

// JLink for smaller JVM bundling
jlinkModules := Seq("java.base", "java.desktop", "java.sql", "javafx.controls", "javafx.fxml")
jlinkOptions := Seq("--strip-debug", "--compress", "2", "--no-header-files", "--no-man-pages")
```

**Deployment Size Optimization**:
- **Base JVM**: ~50MB (JLink custom runtime)
- **Application JAR**: ~20MB  
- **Native libraries**: ~10MB
- **Total**: ~80MB (vs 300MB+ for Electron)

### Mac Deployment (Phase 2)

**Target Distribution**:
- **DMG Image** - Standard Mac distribution
- **Mac App Store** - Wider reach, automatic updates
- **Homebrew Cask** - Developer-friendly distribution

**Implementation**:
```sbt
// build.sbt Mac configuration  
lazy val macSettings = Seq(
  Universal / packageName := "Caribbean-Dental-Practice",
  
  // DMG generation
  Mac / packageBin := {
    val dmg = (Mac / packageBin).value
    // Code sign for Gatekeeper
    Process(s"codesign --force --sign '${developerID}' ${dmg.getAbsolutePath}").!
    // Notarize for macOS 10.15+
    Process(s"xcrun notarytool submit ${dmg.getAbsolutePath} --keychain-profile 'AC_PASSWORD'").!
    dmg
  }
)
```

### Linux Deployment (Phase 3)

**Target Distribution**:
- **AppImage** - Universal Linux distribution
- **Flatpak** - Sandboxed app store distribution
- **Snap Package** - Ubuntu software center
- **Traditional packages** - .deb, .rpm for system integration

**Implementation**:
```sbt
// build.sbt Linux configuration
lazy val linuxSettings = Seq(
  Linux / packageName := "caribbean-dental-practice",
  Linux / packageSummary := "Offline-first dental practice management",
  
  // AppImage generation
  Linux / packageBin := {
    val appDir = (Linux / target).value / "AppDir"
    val appImage = (Linux / target).value / s"${(Linux / packageName).value}.AppImage"
    
    // Create AppImage structure
    IO.createDirectory(appDir / "usr" / "bin")
    IO.copyFile((Linux / stage).value / "bin" / (Linux / packageName).value, 
                appDir / "usr" / "bin" / (Linux / packageName).value)
    
    // Generate AppImage
    Process(s"appimagetool ${appDir.getAbsolutePath} ${appImage.getAbsolutePath}").!
    appImage
  }
)
```

---

## 🔧 Development Environment Setup

### IDE and Tooling Recommendations

**Primary Development Environment**:
```bash
# Required tools for cross-platform development
java --version                    # OpenJDK 17+ (LTS version)
scala --version                   # Scala 3.3.1
sbt --version                     # SBT 1.9+
mill --version                    # Mill 0.11+ (for build validation)

# Platform-specific packaging tools
# Windows
choco install wixtoolset         # WiX for MSI generation
choco install windows-sdk-10     # For code signing

# Mac (when targeting Mac)
xcode-select --install           # Xcode command line tools
brew install create-dmg          # DMG creation

# Linux (when targeting Linux)  
sudo apt install appimagetool    # AppImage creation
sudo apt install flatpak-builder # Flatpak packaging
```

**VS Code Configuration**:
```json
// .vscode/settings.json
{
  "scala.servers": ["metals"],
  "java.home": "/usr/lib/jvm/java-17-openjdk",
  "files.watcherExclude": {
    "**/target/**": true,
    "**/.bloop/**": true
  },
  "metals.enableSemanticHighlighting": true,
  "metals.bloopSbtAlreadyInstalled": true
}

// .vscode/launch.json - Debug configurations
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "scala",
      "request": "launch",  
      "name": "Launch Dental App",
      "mainClass": "com.caribbeandental.DentalPracticeApp",
      "args": [],
      "jvmOptions": ["-Djava.awt.headless=false"]
    }
  ]
}
```

---

## 📊 Performance Benchmarking

### Target Performance Metrics

| Metric | Windows (Primary) | Mac (Secondary) | Linux (Tertiary) |
|--------|------------------|-----------------|-------------------|
| **Startup Time** | < 3 seconds | < 2 seconds | < 3 seconds |
| **Memory Usage** | < 512MB | < 400MB | < 400MB |
| **Patient Load Time** | < 1 second | < 0.5 seconds | < 1 second |
| **Search Response** | < 200ms | < 100ms | < 200ms |
| **Sync Performance** | 1000 events/min | 2000 events/min | 1000 events/min |

### Performance Testing Strategy

```scala
// Performance test suite
class DesktopPerformanceTest extends AnyFunSuite {
  
  test("application startup time under 3 seconds on Windows") {
    val startTime = System.currentTimeMillis()
    
    // Launch application
    val app = new DentalPracticeApp()
    app.start(new Stage())
    
    val startupTime = System.currentTimeMillis() - startTime
    assert(startupTime < 3000, s"Startup took ${startupTime}ms, expected < 3000ms")
  }
  
  test("patient search response under 200ms") {
    val patientService = new PatientService(testEventStore)
    val searchTerm = "John"
    
    val startTime = System.nanoTime()
    val results = patientService.searchPatients(searchTerm)
    val responseTime = (System.nanoTime() - startTime) / 1_000_000 // Convert to ms
    
    assert(responseTime < 200, s"Search took ${responseTime}ms, expected < 200ms")
    assert(results.nonEmpty, "Search should return results")
  }
  
  test("memory usage under 512MB after loading 1000 patients") {
    val runtime = Runtime.getRuntime
    val patientService = new PatientService(testEventStore)
    
    // Load 1000 patients
    (1 to 1000).foreach { i =>
      patientService.loadPatient(PatientId(s"patient-$i"))
    }
    
    val memoryUsed = (runtime.totalMemory() - runtime.freeMemory()) / 1024 / 1024 // MB
    assert(memoryUsed < 512, s"Memory usage: ${memoryUsed}MB, expected < 512MB")
  }
}
```

---

## 🔗 Related Patterns

- **Offline-First-Desktop-Architecture.md** - Core offline-first patterns
- **Windows-Desktop-Healthcare-Patterns.md** - Windows-specific implementation
- **MacOS-Desktop-Healthcare-Patterns.md** - Mac-specific considerations  
- **Linux-Desktop-Healthcare-Patterns.md** - Linux deployment strategies
- **Desktop-Application-Performance-Patterns.md** - Performance optimization techniques

---

**Last Updated**: January 17, 2026  
**Maintained By**: Architect + Platform Team  
**Review Frequency**: Before each platform expansion (Windows → Mac → Linux)  
**Version**: 1.0.0

---

**Key Decision**: Java/Scala + JavaFX provides the optimal balance of performance, cross-platform compatibility, and development velocity for the Caribbean dental practice platform. This choice supports the Windows-first strategy while maintaining flexibility for Mac/Linux expansion.