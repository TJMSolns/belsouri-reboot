# Offline-First Desktop Architecture
## Design Patterns for Caribbean Healthcare Applications

**Purpose**: Architecture patterns for building desktop healthcare applications that operate seamlessly offline for extended periods, with intelligent synchronization when connectivity returns.

**Context**: Caribbean dental practice management systems where internet connectivity is unreliable, expensive, or unavailable during hurricane seasons.

**Key Principle**: The application must function as a **first-class citizen** offline, not as a degraded online application.

---

## 🏗️ Core Architecture Patterns

### Pattern 1: Local-First Data Architecture

**Problem**: Healthcare applications require 100% uptime for patient care, but Caribbean internet connectivity is unreliable.

**Solution**: Treat local storage as the source of truth, with cloud as backup/sync destination.

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Application   │    │   Local Event    │    │   Sync Engine   │
│   Layer         │────│   Store          │────│   (Background)  │
│                 │    │   (Source of     │    │                 │
│                 │    │    Truth)        │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                │                        │
                                ▼                        ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │   Local Cache   │    │   Cloud Backup  │
                       │   (Fast Query)  │    │   (When Online) │
                       └─────────────────┘    └─────────────────┘
```

**Implementation**:
```scala
// Local-first architecture with EventStore
trait LocalFirstStore[T] {
  def save(entity: T): Future[Unit]
  def findById(id: String): Future[Option[T]]
  def query(criteria: QueryCriteria): Future[List[T]]
  def getAllEvents(since: Option[Instant] = None): Stream[DomainEvent]
}

class PatientStore extends LocalFirstStore[Patient] {
  private val eventStore = new LocalEventStore("patients.db")
  private val cache = new LocalCache[Patient]
  
  def save(patient: Patient): Future[Unit] = {
    val event = PatientUpdated(patient.id, patient, Instant.now())
    for {
      _ <- eventStore.append(event)
      _ <- cache.put(patient.id, patient)
    } yield ()
  }
  
  // Query from local cache for speed
  def findById(id: String): Future[Option[Patient]] = {
    cache.get(id).recoverWith {
      case _ => rebuildFromEvents(id)
    }
  }
}
```

### Pattern 2: Event Sourcing for Offline Resilience

**Problem**: Traditional CRUD operations don't handle offline scenarios well - lost updates, inconsistent state.

**Solution**: Event sourcing with immutable event log as single source of truth.

**Benefits for Healthcare**:
- **Complete Audit Trail** - Required for regulatory compliance
- **Conflict Resolution** - Handle concurrent offline edits
- **Point-in-Time Recovery** - Reconstruct patient state at any moment
- **Incremental Sync** - Only sync new events, not full state

**Implementation**:
```scala
sealed trait PatientEvent extends DomainEvent {
  def patientId: PatientId
  def timestamp: Instant
  def userId: UserId
}

case class PatientRegistered(
  patientId: PatientId,
  timestamp: Instant,
  userId: UserId,
  demographics: Demographics
) extends PatientEvent

case class ProcedureCompleted(
  patientId: PatientId, 
  timestamp: Instant,
  userId: UserId,
  procedure: DentalProcedure,
  notes: String,
  attachments: List[FileReference]
) extends PatientEvent

class PatientAggregate(events: List[PatientEvent]) {
  def applyEvent(event: PatientEvent): PatientAggregate = {
    event match {
      case PatientRegistered(id, _, _, demographics) =>
        new PatientAggregate(events :+ event).copy(
          id = id,
          demographics = demographics
        )
      case ProcedureCompleted(_, _, _, procedure, notes, attachments) =>
        new PatientAggregate(events :+ event).copy(
          procedures = procedures :+ CompletedProcedure(procedure, notes, attachments)
        )
    }
  }
  
  // Reconstruct current state from events
  def currentState: Patient = events.foldLeft(Patient.empty)(applyEvent).state
}
```

### Pattern 3: Intelligent Background Synchronization

**Problem**: When connectivity returns, sync must be efficient, conflict-aware, and non-disruptive to ongoing clinical work.

**Solution**: Background sync engine with smart conflict resolution and bandwidth optimization.

**Sync Strategy**:
1. **Event-Based Sync** - Only sync new events since last successful sync
2. **Conflict Detection** - Identify concurrent edits during offline period
3. **Automatic Resolution** - Handle simple conflicts (different fields)
4. **Manual Resolution** - Flag complex conflicts for user review
5. **Bandwidth Optimization** - Compress, batch, prioritize critical data

**Implementation**:
```scala
class BackgroundSyncEngine {
  private val syncQueue = new PersistentQueue[SyncOperation]
  private val conflictResolver = new ConflictResolver
  
  def startBackgroundSync(): Unit = {
    // Run on separate thread, don't block UI
    Future {
      while (true) {
        if (isOnline && hasUnsynced) {
          processSyncQueue()
        }
        Thread.sleep(30.seconds.toMillis)
      }
    }
  }
  
  private def processSyncQueue(): Unit = {
    val batch = syncQueue.takeBatch(maxBatchSize = 100)
    
    batch.groupBy(_.patientId).foreach { case (patientId, operations) =>
      syncPatientData(patientId, operations) match {
        case SyncSuccess => markSynced(operations)
        case SyncConflict(conflicts) => 
          val resolved = conflictResolver.attemptAutoResolve(conflicts)
          val manual = conflicts.filterNot(resolved.contains)
          if (manual.nonEmpty) flagForManualResolution(manual)
        case SyncFailure(error) => 
          requeueWithBackoff(operations, error)
      }
    }
  }
  
  def syncPatientData(patientId: PatientId, operations: List[SyncOperation]): SyncResult = {
    val localEvents = eventStore.getEventsSince(patientId, lastSyncTimestamp)
    val remoteEvents = cloudAPI.getEventsSince(patientId, lastSyncTimestamp)
    
    detectConflicts(localEvents, remoteEvents) match {
      case Nil => 
        // No conflicts, merge and sync
        cloudAPI.uploadEvents(localEvents)
        SyncSuccess
      case conflicts => 
        SyncConflict(conflicts)
    }
  }
}

sealed trait SyncResult
case object SyncSuccess extends SyncResult
case class SyncConflict(conflicts: List[DataConflict]) extends SyncResult
case class SyncFailure(error: Throwable) extends SyncResult
```

### Pattern 4: Local Database Selection and Optimization

**Problem**: Choose appropriate local database technology for offline healthcare applications with specific Caribbean constraints.

**Solution**: Embedded database selection based on use case and performance requirements.

**Database Options Comparison**:

| Database | Use Case | Pros | Cons | Caribbean Suitability |
|----------|----------|------|------|----------------------|
| **SQLite** | Simple relational queries | Lightweight, mature, zero-config | Limited concurrency | ✅ Excellent - single clinic |
| **EventStoreDB** | Event sourcing | Built for events, great performance | Complex setup | ⚠️ Good - if event sourcing |
| **H2 Database** | Java applications | Pure Java, good performance | Java-specific | ✅ Good - Java desktop apps |
| **LevelDB/RocksDB** | Key-value storage | High performance, simple | No SQL queries | ⚠️ Limited - need custom queries |
| **PostgreSQL Embedded** | Complex queries | Full PostgreSQL features | Resource intensive | ❌ Poor - too heavy for old hardware |

**Recommended Architecture for Caribbean Dental Platform**:
```scala
// Hybrid approach: SQLite for queries + Local EventStore for audit trail
class HybridLocalDatabase {
  private val eventStore = new FileBasedEventStore("events/") // Immutable events
  private val queryStore = new SQLiteDatabase("queries.db")   // Optimized for reads
  
  def savePatient(patient: Patient): Future[Unit] = {
    val event = PatientUpdated(patient)
    for {
      _ <- eventStore.append(event)           // Audit trail
      _ <- queryStore.upsert(patient)         // Fast queries
    } yield ()
  }
  
  def findPatient(id: PatientId): Future[Option[Patient]] = {
    queryStore.findById(id) // Fast query path
  }
  
  def getAuditTrail(patientId: PatientId): Future[List[DomainEvent]] = {
    eventStore.getEvents(patientId) // Complete audit trail
  }
}
```

### Pattern 5: File System Management for Patient Data

**Problem**: Manage patient documents, images, and attachments locally with sync capabilities.

**Solution**: Content-addressable storage with differential sync.

**File Storage Strategy**:
```scala
case class FileReference(
  hash: String,           // SHA-256 hash for deduplication
  filename: String,       // Original filename
  mimeType: String,       // File type
  size: Long,             // File size in bytes
  localPath: Option[String], // Local file path (if available)
  syncStatus: SyncStatus     // Upload status
)

class LocalFileManager {
  private val storageRoot = Paths.get(System.getProperty("user.home"), ".dental-app", "files")
  
  def storeFile(inputStream: InputStream, originalName: String): Future[FileReference] = {
    val hash = calculateSHA256(inputStream)
    val localPath = storageRoot.resolve(hash.take(2)).resolve(hash)
    
    // Store file with content-addressable path (deduplication)
    Files.createDirectories(localPath.getParent)
    Files.copy(inputStream, localPath, StandardCopyOption.REPLACE_EXISTING)
    
    FileReference(
      hash = hash,
      filename = originalName,
      mimeType = detectMimeType(originalName),
      size = Files.size(localPath),
      localPath = Some(localPath.toString),
      syncStatus = SyncStatus.PendingUpload
    )
  }
  
  def getFile(reference: FileReference): Future[Option[InputStream]] = {
    reference.localPath match {
      case Some(path) if Files.exists(Paths.get(path)) =>
        Future.successful(Some(Files.newInputStream(Paths.get(path))))
      case _ =>
        // File not available locally, try to download
        downloadFile(reference)
    }
  }
  
  // Efficient sync - only upload new/changed files
  def syncFiles(): Future[SyncResult] = {
    val pendingFiles = fileRepository.findBySyncStatus(SyncStatus.PendingUpload)
    
    pendingFiles.map { file =>
      cloudAPI.uploadFile(file.hash, Files.newInputStream(Paths.get(file.localPath.get)))
        .map(_ => fileRepository.updateSyncStatus(file.hash, SyncStatus.Synced))
    }.sequence
  }
}

sealed trait SyncStatus
object SyncStatus {
  case object PendingUpload extends SyncStatus
  case object Synced extends SyncStatus
  case object DownloadRequired extends SyncStatus
}
```

---

## 🔄 Synchronization Patterns

### Pattern 6: Conflict-Free Replicated Data Types (CRDTs)

**Problem**: Multiple users editing patient data offline can create conflicts that are difficult to resolve.

**Solution**: Use CRDT patterns for conflict-free merging of concurrent edits.

**CRDT Examples for Healthcare**:
```scala
// LWW (Last-Writer-Wins) Register for patient demographics
case class LWWRegister[T](value: T, timestamp: Instant, userId: UserId) {
  def merge(other: LWWRegister[T]): LWWRegister[T] = {
    if (this.timestamp.isAfter(other.timestamp)) this else other
  }
}

// OR-Set for patient allergies (only additions, removals tracked separately)
case class ORSet[T](
  added: Map[T, Set[UserId]], 
  removed: Map[T, Set[UserId]]
) {
  def add(element: T, userId: UserId): ORSet[T] = {
    this.copy(added = added.updated(element, added.getOrElse(element, Set.empty) + userId))
  }
  
  def remove(element: T, userId: UserId): ORSet[T] = {
    this.copy(removed = removed.updated(element, removed.getOrElse(element, Set.empty) + userId))
  }
  
  def elements: Set[T] = {
    added.keySet.filter { element =>
      added(element).nonEmpty && 
      !removed.getOrElse(element, Set.empty).intersect(added(element)).nonEmpty
    }
  }
  
  def merge(other: ORSet[T]): ORSet[T] = {
    ORSet(
      added = mergeMapSets(this.added, other.added),
      removed = mergeMapSets(this.removed, other.removed)
    )
  }
}

// Patient record with CRDT fields
case class Patient(
  id: PatientId,
  name: LWWRegister[String],           // Last writer wins for name changes
  allergies: ORSet[String],            // Add-only set for allergies
  procedures: GSet[ProcedureRecord]    // Grow-only set for completed procedures
) {
  def merge(other: Patient): Patient = {
    require(this.id == other.id, "Cannot merge different patients")
    Patient(
      id = id,
      name = name.merge(other.name),
      allergies = allergies.merge(other.allergies), 
      procedures = procedures.merge(other.procedures)
    )
  }
}
```

### Pattern 7: Optimistic Synchronization with Manual Fallback

**Problem**: Some conflicts cannot be automatically resolved and require human judgment.

**Solution**: Automatic resolution where possible, manual resolution UI for complex cases.

**Implementation**:
```scala
class ConflictResolver {
  def resolveConflicts(localEvents: List[DomainEvent], 
                      remoteEvents: List[DomainEvent]): ConflictResolution = {
    
    val conflicts = detectConflicts(localEvents, remoteEvents)
    val (autoResolved, manualRequired) = conflicts.partition(canAutoResolve)
    
    ConflictResolution(
      autoResolved = autoResolved.map(autoResolve),
      manualRequired = manualRequired
    )
  }
  
  private def canAutoResolve(conflict: DataConflict): Boolean = {
    conflict match {
      case FieldConflict(field, localValue, remoteValue) =>
        // Can auto-resolve if fields don't overlap
        field match {
          case "contactInfo" if localValue != remoteValue => true  // LWW
          case "allergies" => true                                 // OR-Set merge
          case _ => false
        }
      case ProcedureConflict(_, _) => false  // Always require manual resolution
    }
  }
}

case class ConflictResolution(
  autoResolved: List[ResolvedConflict],
  manualRequired: List[DataConflict]
)

// UI for manual conflict resolution
class ConflictResolutionUI {
  def showConflictDialog(conflicts: List[DataConflict]): Future[List[ResolvedConflict]] = {
    // Show side-by-side comparison of local vs remote changes
    // Allow user to choose: Keep Local, Keep Remote, or Custom Merge
    
    conflicts.map { conflict =>
      val resolution = showConflictChoice(conflict)
      ResolvedConflict(conflict, resolution)
    }.sequence
  }
}
```

---

## 🛡️ Offline Licensing and Security

### Pattern 8: Grace Period Licensing

**Problem**: Software licenses must be validated periodically, but internet may be unavailable for extended periods.

**Solution**: Grace period licensing with cryptographic license tokens.

**Implementation**:
```scala
case class OfflineLicense(
  clinicId: String,
  issuedDate: Instant,
  expiryDate: Instant,
  gracePeriodDays: Int,
  signature: String  // Cryptographic signature from licensing server
) {
  def isValidAt(checkTime: Instant): Boolean = {
    val gracePeriodEnd = expiryDate.plus(gracePeriodDays, ChronoUnit.DAYS)
    checkTime.isBefore(gracePeriodEnd) && isSignatureValid
  }
  
  def daysUntilExpiry(currentTime: Instant): Long = {
    ChronoUnit.DAYS.between(currentTime, expiryDate)
  }
  
  def isInGracePeriod(currentTime: Instant): Boolean = {
    currentTime.isAfter(expiryDate) && isValidAt(currentTime)
  }
}

class OfflineLicenseManager {
  def checkLicense(): LicenseStatus = {
    val license = loadStoredLicense()
    val now = Instant.now()
    
    license match {
      case Some(lic) if lic.isValidAt(now) && !lic.isInGracePeriod(now) =>
        LicenseStatus.Valid(lic.daysUntilExpiry(now))
      case Some(lic) if lic.isValidAt(now) && lic.isInGracePeriod(now) =>
        LicenseStatus.GracePeriod(lic.daysUntilExpiry(now))
      case Some(lic) =>
        LicenseStatus.Expired
      case None =>
        LicenseStatus.NotLicensed
    }
  }
  
  // Attempt to refresh license when online
  def refreshLicense(): Future[LicenseStatus] = {
    if (isOnline) {
      licenseServer.requestNewLicense(clinicId)
        .map(storeLicense)
        .map(_ => checkLicense())
    } else {
      Future.successful(checkLicense())
    }
  }
}

sealed trait LicenseStatus
object LicenseStatus {
  case class Valid(daysRemaining: Long) extends LicenseStatus
  case class GracePeriod(daysRemaining: Long) extends LicenseStatus  
  case object Expired extends LicenseStatus
  case object NotLicensed extends LicenseStatus
}
```

---

## 📊 Performance Optimization Patterns

### Pattern 9: Lazy Loading and Caching

**Problem**: Loading complete patient histories can be slow on limited Caribbean hardware.

**Solution**: Lazy loading with intelligent caching strategies.

**Implementation**:
```scala
class PatientDataCache {
  private val recentPatients = new LRUCache[PatientId, Patient](maxSize = 50)
  private val patientSummaries = new Cache[PatientId, PatientSummary]
  
  def getPatient(id: PatientId): Future[Patient] = {
    recentPatients.get(id) match {
      case Some(patient) => Future.successful(patient)  // Cache hit
      case None => 
        loadPatientFromDisk(id).map { patient =>
          recentPatients.put(id, patient)
          patient
        }
    }
  }
  
  def getPatientSummary(id: PatientId): Future[PatientSummary] = {
    patientSummaries.getOrElse(id) {
      // Load only summary data, not full history
      patientRepository.loadSummary(id)
    }
  }
  
  // Preload frequently accessed data
  def preloadRecentPatients(): Future[Unit] = {
    val recentIds = appointmentRepository.getUpcomingAppointments(next7Days)
      .map(_.patientId)
    
    recentIds.foreach(getPatient) // Warm the cache
  }
}

case class PatientSummary(
  id: PatientId,
  name: String,
  dateOfBirth: LocalDate,
  lastVisit: Option[LocalDate],
  nextAppointment: Option[LocalDateTime],
  alertFlags: Set[AlertFlag]  // Allergies, special needs, etc.
)
```

---

## 🔗 Related Patterns

- **Desktop-Healthcare-Data-Security.md** - Local encryption and security patterns
- **Caribbean-Desktop-Resilience-Patterns.md** - Power and hardware failure recovery
- **Local-First-Healthcare-Data-Architecture.md** - Domain modeling for offline systems
- **Cross-Platform-Desktop-Development-Strategies.md** - Technology stack choices

---

**Last Updated**: January 17, 2026  
**Maintained By**: Architect + Desktop Development Team  
**Review Frequency**: When offline requirements change or new sync conflicts discovered  
**Version**: 1.0.0

---

**Key Insight**: True offline-first architecture inverts traditional thinking - instead of "online app with offline mode," build "offline app with online sync." This mindset shift is crucial for Caribbean healthcare applications where connectivity is a luxury, not a requirement.