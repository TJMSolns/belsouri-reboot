# Local-First Healthcare Data Architecture
## Advanced Local Data Patterns for Caribbean Dental Practice Platforms

**Purpose**: Advanced architectural patterns for local-first healthcare data management, focusing on data consistency, synchronization, conflict resolution, and offline capabilities for Caribbean dental practices with intermittent connectivity.

**Context**: Building on Offline-First-Desktop-Architecture.md with deeper focus on healthcare-specific data patterns, ACID compliance for medical data, and complex multi-user scenarios in small Caribbean practices.

**Key Principle**: **Medical data integrity above all else** - Ensure no patient data loss or corruption even during power failures, network outages, or hardware failures common in Caribbean environments.

---

## 🏥 Healthcare Data Modeling Patterns

### Pattern 1: Medical Event Sourcing with Clinical Context

**Problem**: Medical records must maintain complete audit trails, support concurrent access by multiple practitioners, and never lose critical patient information, even during infrastructure failures.

**Solution**: Domain-specific event sourcing with medical event semantics and guaranteed persistence.

**Clinical Event Architecture**:
```scala
// Base medical event with clinical context
sealed trait MedicalEvent {
  def eventId: EventId
  def patientId: PatientId
  def practitionerId: PractitionerId
  def clinicalContext: ClinicalContext
  def timestamp: MedicalTimestamp
  def signatures: List[DigitalSignature]
}

case class MedicalTimestamp(
  recordedAt: Instant,
  clinicalDate: LocalDate, // When treatment actually occurred
  timezone: ZoneId
) {
  // Handle Caribbean timezone complexities
  def toUTC: Instant = recordedAt
  def toClinicalTime: LocalDateTime = clinicalDate.atStartOfDay(timezone)
}

case class ClinicalContext(
  appointmentId: Option[AppointmentId],
  treatmentPhase: TreatmentPhase,
  supervisionLevel: SupervisionLevel,
  location: ClinicalLocation
)

sealed trait TreatmentPhase
object TreatmentPhase {
  case object Consultation extends TreatmentPhase
  case object Diagnosis extends TreatmentPhase
  case object TreatmentPlanning extends TreatmentPhase
  case object Treatment extends TreatmentPhase
  case object Follow up extends TreatmentPhase
  case object Emergency extends TreatmentPhase
}

sealed trait SupervisionLevel
object SupervisionLevel {
  case object DentistDirect extends SupervisionLevel // Dentist performing
  case object DentistSupervised extends SupervisionLevel // Under direct supervision
  case object DentistReviewed extends SupervisionLevel // Reviewed by dentist
  case object IndependentHygienist extends SupervisionLevel // Hygienist independent practice
}

// Specific medical events for dental practice
case class PatientAdmissionEvent(
  eventId: EventId,
  patientId: PatientId,
  practitionerId: PractitionerId,
  clinicalContext: ClinicalContext,
  timestamp: MedicalTimestamp,
  signatures: List[DigitalSignature],
  demographics: PatientDemographics,
  medicalHistory: InitialMedicalHistory,
  consentForms: List[ConsentForm]
) extends MedicalEvent

case class ClinicalExaminationEvent(
  eventId: EventId,
  patientId: PatientId,
  practitionerId: PractitionerId,
  clinicalContext: ClinicalContext,
  timestamp: MedicalTimestamp,
  signatures: List[DigitalSignature],
  examinationFindings: ExaminationFindings,
  vitalSigns: Option[VitalSigns],
  painAssessment: Option[PainAssessment]
) extends MedicalEvent

case class DiagnosisEvent(
  eventId: EventId,
  patientId: PatientId,
  practitionerId: PractitionerId,
  clinicalContext: ClinicalContext,
  timestamp: MedicalTimestamp,
  signatures: List[DigitalSignature],
  primaryDiagnosis: DiagnosisCode,
  secondaryDiagnoses: List[DiagnosisCode],
  differentialDiagnoses: List[DiagnosisCode],
  clinicalReasoning: String,
  supportingEvidence: List[ClinicalEvidence]
) extends MedicalEvent

case class TreatmentEvent(
  eventId: EventId,
  patientId: PatientId,
  practitionerId: PractitionerId,
  clinicalContext: ClinicalContext,
  timestamp: MedicalTimestamp,
  signatures: List[DigitalSignature],
  procedure: ProcedureCode,
  toothSurfaces: List[ToothSurface],
  materials: List[MaterialUsed],
  complications: Option[TreatmentComplication],
  outcome: TreatmentOutcome
) extends MedicalEvent

// Medical event store with ACID guarantees
class MedicalEventStore {
  
  private val eventStorage = new ACIDEventStorage()
  private val eventIndex = new MedicalEventIndex()
  private val integrityChecker = new MedicalIntegrityChecker()
  
  def appendEvent(event: MedicalEvent): Either[MedicalEventError, EventAppendResult] = {
    // Validate medical event integrity
    integrityChecker.validateEvent(event) match {
      case Left(validationError) =>
        Left(MedicalEventError.ValidationFailed(validationError))
      case Right(_) =>
        // Use atomic transaction for ACID compliance
        eventStorage.atomicTransaction { transaction =>
          for {
            // Verify no conflicts with existing events
            _ <- checkEventConflicts(event, transaction)
            
            // Append event to storage
            appendResult <- transaction.appendEvent(event)
            
            // Update indexes
            _ <- transaction.updateIndex(eventIndex, event)
            
            // Verify integrity after write
            _ <- transaction.verifyIntegrity(event)
            
          } yield appendResult
        }
    }
  }
  
  def getPatientEventStream(
    patientId: PatientId,
    fromTimestamp: Option[MedicalTimestamp] = None
  ): EventStream[MedicalEvent] = {
    
    val query = MedicalEventQuery(
      patientId = Some(patientId),
      fromTimestamp = fromTimestamp,
      ordering = EventOrdering.ChronologicalAscending
    )
    
    eventStorage.queryEvents(query)
  }
  
  def replayPatientHistory(patientId: PatientId): PatientMedicalRecord = {
    val events = getPatientEventStream(patientId).toList
    
    // Replay events to reconstruct current state
    events.foldLeft(PatientMedicalRecord.empty(patientId)) { (record, event) =>
      applyEventToRecord(record, event)
    }
  }
  
  private def applyEventToRecord(
    record: PatientMedicalRecord,
    event: MedicalEvent
  ): PatientMedicalRecord = {
    event match {
      case admission: PatientAdmissionEvent =>
        record.copy(
          demographics = Some(admission.demographics),
          medicalHistory = Some(admission.medicalHistory),
          consentForms = record.consentForms ++ admission.consentForms
        )
        
      case examination: ClinicalExaminationEvent =>
        record.copy(
          examinations = record.examinations :+ examination.examinationFindings,
          lastVitalSigns = examination.vitalSigns.orElse(record.lastVitalSigns)
        )
        
      case diagnosis: DiagnosisEvent =>
        record.copy(
          currentDiagnoses = record.currentDiagnoses :+ diagnosis.primaryDiagnosis,
          diagnosticHistory = record.diagnosticHistory :+ diagnosis
        )
        
      case treatment: TreatmentEvent =>
        record.copy(
          treatments = record.treatments :+ treatment,
          dentalChart = updateDentalChart(record.dentalChart, treatment)
        )
    }
  }
  
  private def checkEventConflicts(
    newEvent: MedicalEvent,
    transaction: EventTransaction
  ): Either[MedicalEventError, Unit] = {
    
    // Check for temporal conflicts (e.g., overlapping appointments)
    val conflictingEvents = transaction.findConflictingEvents(newEvent)
    
    if (conflictingEvents.nonEmpty) {
      Left(MedicalEventError.ConflictDetected(conflictingEvents))
    } else {
      Right(())
    }
  }
}
```

### Pattern 2: Clinical Decision Support Data Structures

**Problem**: Dental practices need real-time clinical decision support, drug interaction checking, and treatment recommendations, all working offline with locally stored medical knowledge.

**Solution**: Local clinical knowledge graph with inference engine and decision trees.

**Implementation**:
```scala
// Clinical knowledge graph for offline decision support
class ClinicalKnowledgeGraph {
  
  private val drugDatabase = new LocalDrugDatabase()
  private val diagnosisEngine = new DiagnosisInferenceEngine()
  private val treatmentProtocols = new TreatmentProtocolEngine()
  
  def checkDrugInteractions(
    proposedMedication: Medication,
    patientMedications: List[Medication],
    patientConditions: List[MedicalCondition]
  ): DrugInteractionResult = {
    
    val interactions = mutable.ListBuffer[DrugInteraction]()
    
    // Check drug-drug interactions
    patientMedications.foreach { existing =>
      drugDatabase.getInteraction(proposedMedication.activeIngredient, existing.activeIngredient) match {
        case Some(interaction) =>
          interactions += interaction
        case None =>
          // No direct interaction found
      }
    }
    
    // Check drug-condition interactions
    patientConditions.foreach { condition =>
      drugDatabase.getContraindication(proposedMedication.activeIngredient, condition) match {
        case Some(contraindication) =>
          interactions += DrugInteraction.Contraindication(contraindication)
        case None =>
          // No contraindication
      }
    }
    
    // Assess overall interaction severity
    val maxSeverity = if (interactions.nonEmpty) {
      interactions.map(_.severity).maxBy(_.level)
    } else {
      InteractionSeverity.None
    }
    
    DrugInteractionResult(
      medication = proposedMedication,
      interactions = interactions.toList,
      overallSeverity = maxSeverity,
      recommendations = generateRecommendations(interactions.toList)
    )
  }
  
  def suggestDifferentialDiagnoses(
    symptoms: List[Symptom],
    physicalFindings: List[PhysicalFinding],
    patientHistory: PatientMedicalHistory
  ): List[DifferentialDiagnosis] = {
    
    // Use Bayesian inference for diagnosis suggestion
    val possibleDiagnoses = diagnosisEngine.findMatchingDiagnoses(symptoms, physicalFindings)
    
    val scoredDiagnoses = possibleDiagnoses.map { diagnosis =>
      val likelihood = calculateDiagnosisLikelihood(
        diagnosis = diagnosis,
        symptoms = symptoms,
        findings = physicalFindings,
        history = patientHistory
      )
      
      DifferentialDiagnosis(
        diagnosis = diagnosis,
        likelihood = likelihood,
        supportingEvidence = findSupportingEvidence(diagnosis, symptoms, physicalFindings),
        contradictingEvidence = findContradictingEvidence(diagnosis, symptoms, physicalFindings)
      )
    }
    
    // Sort by likelihood and return top candidates
    scoredDiagnoses
      .sortBy(_.likelihood.probability)
      .reverse
      .take(10) // Top 10 differential diagnoses
  }
  
  def recommendTreatmentPlan(
    diagnosis: Diagnosis,
    patientFactors: PatientFactors,
    practiceCapabilities: PracticeCapabilities
  ): TreatmentPlanRecommendation = {
    
    val standardProtocol = treatmentProtocols.getStandardProtocol(diagnosis)
    
    // Customize based on patient factors
    val customizedProtocol = customizeProtocolForPatient(standardProtocol, patientFactors)
    
    // Adjust based on practice capabilities
    val feasibleProtocol = adjustForPracticeCapabilities(customizedProtocol, practiceCapabilities)
    
    TreatmentPlanRecommendation(
      diagnosis = diagnosis,
      recommendedTreatment = feasibleProtocol,
      alternativeOptions = findAlternativeTreatments(diagnosis, patientFactors),
      riskAssessment = assessTreatmentRisks(feasibleProtocol, patientFactors),
      followUpSchedule = generateFollowUpSchedule(feasibleProtocol)
    )
  }
  
  private def calculateDiagnosisLikelihood(
    diagnosis: Diagnosis,
    symptoms: List[Symptom],
    findings: List[PhysicalFinding],
    history: PatientMedicalHistory
  ): DiagnosisLikelihood = {
    
    // Bayesian calculation
    val priorProbability = diagnosisEngine.getPriorProbability(diagnosis, history)
    
    val symptomLikelihoods = symptoms.map { symptom =>
      diagnosisEngine.getSymptomLikelihood(symptom, diagnosis)
    }
    
    val findingLikelihoods = findings.map { finding =>
      diagnosisEngine.getFindingLikelihood(finding, diagnosis)
    }
    
    // Combine likelihoods using Bayesian formula
    val allLikelihoods = symptomLikelihoods ++ findingLikelihoods
    val combinedLikelihood = allLikelihoods.foldLeft(priorProbability) { (prob, likelihood) =>
      bayesianUpdate(prob, likelihood)
    }
    
    DiagnosisLikelihood(
      probability = combinedLikelihood,
      confidence = calculateConfidence(allLikelihoods),
      evidenceStrength = assessEvidenceStrength(symptoms, findings, diagnosis)
    )
  }
}

// Offline clinical reference database
class LocalClinicalDatabase {
  
  private val icd10Codes = new ICD10Database()
  private val cptCodes = new CPTDatabase()
  private val dentalCodes = new CDTDatabase() // Current Dental Terminology
  private val drugDatabase = new RxNormDatabase()
  
  def loadClinicalDatabase(): Unit = {
    // Load compressed clinical databases for offline use
    loadICD10Database()
    loadCPTDatabase()
    loadDentalCodes()
    loadDrugDatabase()
    loadClinicalGuidelines()
  }
  
  private def loadDentalCodes(): Unit = {
    // Load Current Dental Terminology (CDT) codes
    val cdtData = resourceLoader.loadCompressedResource("clinical-db/cdt-codes.gz")
    
    val procedures = cdtData.procedures.map { proc =>
      DentalProcedure(
        code = proc.code,
        description = proc.description,
        category = proc.category,
        complexity = proc.complexity,
        estimatedTime = proc.estimatedTime,
        prerequisites = proc.prerequisites,
        contraindications = proc.contraindications,
        materials = proc.materials
      )
    }
    
    cptCodes.loadProcedures(procedures)
  }
  
  def searchProcedures(
    query: String,
    specialty: Specialty = Specialty.GeneralDentistry
  ): List[ProcedureMatch] = {
    
    val searchTerms = tokenizeSearchQuery(query)
    
    val matches = cptCodes.searchProcedures(searchTerms, specialty)
    
    matches.map { procedure =>
      val relevanceScore = calculateRelevanceScore(procedure, searchTerms)
      
      ProcedureMatch(
        procedure = procedure,
        relevanceScore = relevanceScore,
        matchedTerms = findMatchedTerms(procedure, searchTerms)
      )
    }.sortBy(_.relevanceScore).reverse
  }
  
  def validateDiagnosisCode(code: String): DiagnosisCodeValidation = {
    icd10Codes.findCode(code) match {
      case Some(diagnosis) =>
        DiagnosisCodeValidation.Valid(diagnosis)
      case None =>
        // Try to find similar codes
        val similarCodes = icd10Codes.findSimilar(code, maxDistance = 2)
        DiagnosisCodeValidation.Invalid(similarCodes)
    }
  }
}
```

### Pattern 3: Multi-User Conflict Resolution for Clinical Data

**Problem**: Multiple practitioners may simultaneously update patient records, creating data conflicts that could compromise patient safety. Critical medical information must never be lost due to merge conflicts.

**Solution**: Medical-aware operational transformation with conflict resolution policies prioritizing patient safety.

**Implementation**:
```scala
// Medical conflict resolution system
class MedicalConflictResolver {
  
  sealed trait ConflictResolutionStrategy
  object ConflictResolutionStrategy {
    case object SafetyFirst extends ConflictResolutionStrategy // Preserve all safety-critical data
    case object ChronologicalOrder extends ConflictResolutionStrategy // Latest timestamp wins
    case object PractitionerHierarchy extends ConflictResolutionStrategy // Higher authority wins
    case object ManualReview extends ConflictResolutionStrategy // Require human resolution
  }
  
  def resolveConflicts(
    localChanges: List[MedicalChange],
    remoteChanges: List[MedicalChange]
  ): ConflictResolutionResult = {
    
    val conflicts = detectConflicts(localChanges, remoteChanges)
    
    val resolutions = conflicts.map { conflict =>
      resolveConflict(conflict)
    }
    
    ConflictResolutionResult(
      resolvedChanges = mergeChanges(localChanges, remoteChanges, resolutions),
      manualReviewRequired = resolutions.filter(_.requiresManualReview),
      safetyWarnings = resolutions.filter(_.hasSafetyImplications).map(_.warning)
    )
  }
  
  private def resolveConflict(conflict: MedicalConflict): ConflictResolution = {
    conflict match {
      case DiagnosisConflict(local, remote) =>
        resolveDiagnosisConflict(local, remote)
      case MedicationConflict(local, remote) =>
        resolveMedicationConflict(local, remote)
      case AllergyConflict(local, remote) =>
        resolveAllergyConflict(local, remote)
      case VitalSignsConflict(local, remote) =>
        resolveVitalSignsConflict(local, remote)
      case TreatmentConflict(local, remote) =>
        resolveTreatmentConflict(local, remote)
    }
  }
  
  private def resolveDiagnosisConflict(
    local: DiagnosisChange,
    remote: DiagnosisChange
  ): ConflictResolution = {
    
    // Diagnosis conflicts are safety-critical - preserve both
    if (isDiagnosisSafetyCritical(local.diagnosis) || isDiagnosisSafetyCritical(remote.diagnosis)) {
      ConflictResolution(
        strategy = ConflictResolutionStrategy.SafetyFirst,
        result = ConflictResult.PreserveBoth(local, remote),
        requiresManualReview = true,
        safetyImplications = true,
        warning = Some("Critical diagnosis conflict requires manual review")
      )
    } else {
      // Non-critical diagnosis - use practitioner hierarchy
      val practitionerComparison = comparePractitionerAuthority(
        local.practitionerId,
        remote.practitionerId
      )
      
      practitionerComparison match {
        case PractitionerComparison.LocalHigher =>
          ConflictResolution.acceptLocal(local)
        case PractitionerComparison.RemoteHigher =>
          ConflictResolution.acceptRemote(remote)
        case PractitionerComparison.Equal =>
          // Same authority level - use chronological order
          if (local.timestamp.isAfter(remote.timestamp)) {
            ConflictResolution.acceptLocal(local)
          } else {
            ConflictResolution.acceptRemote(remote)
          }
      }
    }
  }
  
  private def resolveMedicationConflict(
    local: MedicationChange,
    remote: MedicationChange
  ): ConflictResolution = {
    
    // Medication changes are always safety-critical
    val drugInteractionCheck = checkDrugInteractions(local.medication, remote.medication)
    
    if (drugInteractionCheck.hasSevereInteraction) {
      ConflictResolution(
        strategy = ConflictResolutionStrategy.SafetyFirst,
        result = ConflictResult.RequireManualReview,
        requiresManualReview = true,
        safetyImplications = true,
        warning = Some(s"Severe drug interaction detected: ${drugInteractionCheck.description}")
      )
    } else {
      // Check if changes are complementary (e.g., different medications)
      if (areComplementaryMedications(local.medication, remote.medication)) {
        ConflictResolution(
          strategy = ConflictResolutionStrategy.SafetyFirst,
          result = ConflictResult.MergeBoth(local, remote),
          requiresManualReview = false,
          safetyImplications = false
        )
      } else {
        // Conflicting medications - require manual review
        ConflictResolution(
          strategy = ConflictResolutionStrategy.ManualReview,
          result = ConflictResult.RequireManualReview,
          requiresManualReview = true,
          safetyImplications = true,
          warning = Some("Conflicting medication changes require practitioner review")
        )
      }
    }
  }
  
  private def resolveAllergyConflict(
    local: AllergyChange,
    remote: AllergyChange
  ): ConflictResolution = {
    
    // Allergy information is safety-critical - always preserve additions
    (local.changeType, remote.changeType) match {
      case (ChangeType.Add, ChangeType.Add) =>
        // Both adding allergies - merge them
        ConflictResolution.mergeBoth(local, remote)
        
      case (ChangeType.Add, ChangeType.Remove) | (ChangeType.Remove, ChangeType.Add) =>
        // One adding, one removing - require manual review
        ConflictResolution(
          strategy = ConflictResolutionStrategy.ManualReview,
          result = ConflictResult.RequireManualReview,
          requiresManualReview = true,
          safetyImplications = true,
          warning = Some("Conflicting allergy changes (add/remove) require manual review")
        )
        
      case (ChangeType.Remove, ChangeType.Remove) =>
        // Both removing - use chronological order
        if (local.timestamp.isAfter(remote.timestamp)) {
          ConflictResolution.acceptLocal(local)
        } else {
          ConflictResolution.acceptRemote(remote)
        }
    }
  }
}

// Operational transformation for real-time collaboration
class MedicalOperationalTransform {
  
  def transform(
    localOperation: MedicalOperation,
    remoteOperation: MedicalOperation
  ): (MedicalOperation, MedicalOperation) = {
    
    (localOperation, remoteOperation) match {
      case (localInsert: InsertOperation, remoteInsert: InsertOperation) =>
        transformInsertInsert(localInsert, remoteInsert)
      case (localInsert: InsertOperation, remoteDelete: DeleteOperation) =>
        transformInsertDelete(localInsert, remoteDelete)
      case (localDelete: DeleteOperation, remoteInsert: InsertOperation) =>
        transformDeleteInsert(localDelete, remoteInsert)
      case (localDelete: DeleteOperation, remoteDelete: DeleteOperation) =>
        transformDeleteDelete(localDelete, remoteDelete)
      case (localUpdate: UpdateOperation, remoteUpdate: UpdateOperation) =>
        transformUpdateUpdate(localUpdate, remoteUpdate)
    }
  }
  
  private def transformInsertInsert(
    local: InsertOperation,
    remote: InsertOperation
  ): (MedicalOperation, MedicalOperation) = {
    
    if (local.position <= remote.position) {
      // Local insert before remote - adjust remote position
      val adjustedRemote = remote.copy(position = remote.position + local.length)
      (local, adjustedRemote)
    } else {
      // Remote insert before local - adjust local position
      val adjustedLocal = local.copy(position = local.position + remote.length)
      (adjustedLocal, remote)
    }
  }
  
  private def transformUpdateUpdate(
    local: UpdateOperation,
    remote: UpdateOperation
  ): (MedicalOperation, MedicalOperation) = {
    
    if (local.targetField != remote.targetField) {
      // Updating different fields - no transformation needed
      (local, remote)
    } else {
      // Updating same field - apply medical conflict resolution
      val conflictResolution = resolveMedicalFieldConflict(local, remote)
      
      conflictResolution match {
        case FieldResolution.AcceptLocal =>
          (local, NoOperation)
        case FieldResolution.AcceptRemote =>
          (NoOperation, remote)
        case FieldResolution.RequireReview =>
          // Mark both operations as requiring manual review
          val reviewLocal = local.copy(requiresReview = true)
          val reviewRemote = remote.copy(requiresReview = true)
          (reviewLocal, reviewRemote)
      }
    }
  }
  
  private def resolveMedicalFieldConflict(
    local: UpdateOperation,
    remote: UpdateOperation
  ): FieldResolution = {
    
    local.targetField match {
      case MedicalField.Allergies | MedicalField.Medications | MedicalField.Diagnosis =>
        // Safety-critical fields require manual review
        FieldResolution.RequireReview
        
      case MedicalField.Demographics | MedicalField.ContactInfo =>
        // Non-critical fields - use timestamp
        if (local.timestamp.isAfter(remote.timestamp)) {
          FieldResolution.AcceptLocal
        } else {
          FieldResolution.AcceptRemote
        }
        
      case MedicalField.TreatmentNotes =>
        // Treatment notes can often be merged
        if (canMergeTreatmentNotes(local.value, remote.value)) {
          FieldResolution.AcceptLocal // Will be merged in higher-level logic
        } else {
          FieldResolution.RequireReview
        }
    }
  }
}
```

---

## 💾 Advanced Data Storage and Retrieval Patterns

### Pattern 4: Medical Data Partitioning and Archival

**Problem**: Long-term patient records accumulate substantial data over decades, but Caribbean practices have limited storage capacity and need fast access to recent data while maintaining complete historical records.

**Solution**: Intelligent data tiering with medical record lifecycle management and efficient compression.

**Implementation**:
```scala
// Medical data lifecycle manager
class MedicalDataLifecycleManager {
  
  private val activeStorage = new HighPerformanceStorage()
  private val nearlineStorage = new CompressedStorage()
  private val archiveStorage = new EncryptedArchiveStorage()
  
  def partitionPatientData(patientId: PatientId): DataPartitioningPlan = {
    val patientHistory = getPatientHistory(patientId)
    
    val partitioningRules = List(
      ActiveDataRule(timeRange = 6.months),
      NearlineDataRule(timeRange = 2.years),
      ArchiveDataRule(timeRange = Duration.Inf),
      CriticalDataRule(alwaysActive = true)
    )
    
    val partitionedData = partitioningRules.foldLeft(Map.empty[StorageTier, List[MedicalRecord]]) {
      (partitions, rule) =>
        val matchingRecords = patientHistory.filter(rule.matches)
        partitions + (rule.targetTier -> matchingRecords)
    }
    
    DataPartitioningPlan(
      patientId = patientId,
      partitions = partitionedData,
      compressionStrategy = selectCompressionStrategy(partitionedData),
      indexingStrategy = selectIndexingStrategy(partitionedData)
    )
  }
  
  def executeDataLifecyclePolicy(): Unit = {
    val allPatients = patientRepository.getAllPatientIds()
    
    allPatients.foreach { patientId =>
      val plan = partitionPatientData(patientId)
      executePartitioningPlan(plan)
    }
  }
  
  private def executePartitioningPlan(plan: DataPartitioningPlan): Unit = {
    plan.partitions.foreach { case (tier, records) =>
      tier match {
        case StorageTier.Active =>
          // Keep in high-performance storage
          records.foreach(activeStorage.store)
          
        case StorageTier.Nearline =>
          // Compress and move to nearline storage
          val compressedRecords = compressRecords(records, plan.compressionStrategy)
          compressedRecords.foreach(nearlineStorage.store)
          
        case StorageTier.Archive =>
          // Encrypt, compress, and archive
          val processedRecords = records
            .map(encryptRecord)
            .map(compressRecord)
          processedRecords.foreach(archiveStorage.store)
      }
    }
  }
  
  private def selectCompressionStrategy(
    partitions: Map[StorageTier, List[MedicalRecord]]
  ): CompressionStrategy = {
    
    val totalDataSize = partitions.values.flatten.map(_.dataSize).sum
    val storageCapacity = systemInfo.getAvailableStorage()
    
    val compressionRatio = totalDataSize.toDouble / storageCapacity.toDouble
    
    compressionRatio match {
      case ratio if ratio > 0.8 =>
        // High storage pressure - aggressive compression
        CompressionStrategy.MaximumCompression
      case ratio if ratio > 0.6 =>
        // Moderate storage pressure - balanced compression
        CompressionStrategy.BalancedCompression
      case _ =>
        // Low storage pressure - fast compression
        CompressionStrategy.FastCompression
    }
  }
}

// Efficient medical record compression
class MedicalRecordCompressor {
  
  def compressRecord(
    record: MedicalRecord,
    strategy: CompressionStrategy
  ): CompressedMedicalRecord = {
    
    strategy match {
      case CompressionStrategy.MaximumCompression =>
        compressWithLZMA(record)
      case CompressionStrategy.BalancedCompression =>
        compressWithZstd(record)
      case CompressionStrategy.FastCompression =>
        compressWithLZ4(record)
    }
  }
  
  private def compressWithLZMA(record: MedicalRecord): CompressedMedicalRecord = {
    // LZMA provides excellent compression ratio for text-heavy medical records
    val textComponents = extractTextComponents(record)
    val binaryComponents = extractBinaryComponents(record)
    
    val compressedText = lzmaCompressor.compress(textComponents)
    val compressedBinary = lz4Compressor.compress(binaryComponents) // LZ4 for binary data
    
    CompressedMedicalRecord(
      recordId = record.id,
      compressedText = compressedText,
      compressedBinary = compressedBinary,
      compressionAlgorithm = CompressionAlgorithm.LZMA,
      originalSize = record.dataSize,
      compressedSize = compressedText.length + compressedBinary.length,
      compressionMetadata = CompressionMetadata(
        textRatio = compressedText.length.toDouble / textComponents.length.toDouble,
        binaryRatio = compressedBinary.length.toDouble / binaryComponents.length.toDouble
      )
    )
  }
  
  private def extractTextComponents(record: MedicalRecord): Array[Byte] = {
    // Extract and serialize text components for optimal compression
    val textData = MedicalTextData(
      demographics = record.demographics,
      diagnoses = record.diagnoses.map(_.description),
      treatmentNotes = record.treatments.map(_.notes),
      medications = record.medications.map(_.name),
      allergies = record.allergies.map(_.description)
    )
    
    jsonSerializer.serialize(textData)
  }
  
  def decompressRecord(compressed: CompressedMedicalRecord): MedicalRecord = {
    compressed.compressionAlgorithm match {
      case CompressionAlgorithm.LZMA =>
        decompressLZMA(compressed)
      case CompressionAlgorithm.ZSTD =>
        decompressZstd(compressed)
      case CompressionAlgorithm.LZ4 =>
        decompressLZ4(compressed)
    }
  }
}

// Intelligent medical record indexing
class MedicalRecordIndex {
  
  private val patientIndex = new PatientIndex()
  private val diagnosisIndex = new DiagnosisIndex()
  private val medicationIndex = new MedicationIndex()
  private val procedureIndex = new ProcedureIndex()
  private val dateIndex = new TemporalIndex()
  
  def buildIndex(records: List[MedicalRecord]): IndexBuildResult = {
    val indexBuilder = new ParallelIndexBuilder()
    
    val indexingTasks = List(
      indexBuilder.buildPatientIndex(records),
      indexBuilder.buildDiagnosisIndex(records),
      indexBuilder.buildMedicationIndex(records),
      indexBuilder.buildProcedureIndex(records),
      indexBuilder.buildDateIndex(records)
    )
    
    // Execute indexing tasks in parallel
    val indexResults = indexingTasks.map(_.execute())
    
    IndexBuildResult(
      totalRecords = records.length,
      indexSizes = indexResults.map(_.indexSize).toMap,
      buildDuration = indexResults.map(_.buildTime).max,
      memoryUsage = indexResults.map(_.memoryUsage).sum
    )
  }
  
  def searchRecords(query: MedicalQuery): List[MedicalRecord] = {
    val candidateRecords = query.filters.foldLeft(Set.empty[RecordId]) {
      (candidates, filter) =>
        val filterResults = executeFilter(filter)
        if (candidates.isEmpty) {
          filterResults
        } else {
          candidates.intersect(filterResults)
        }
    }
    
    // Retrieve records and sort by relevance
    val records = candidateRecords.toList.map(recordStorage.getRecord)
    sortByRelevance(records, query)
  }
  
  private def executeFilter(filter: SearchFilter): Set[RecordId] = {
    filter match {
      case PatientFilter(patientId) =>
        patientIndex.getRecords(patientId)
      case DiagnosisFilter(diagnosisCode) =>
        diagnosisIndex.getRecords(diagnosisCode)
      case MedicationFilter(medicationName) =>
        medicationIndex.getRecords(medicationName)
      case DateRangeFilter(startDate, endDate) =>
        dateIndex.getRecordsInRange(startDate, endDate)
      case ProcedureFilter(procedureCode) =>
        procedureIndex.getRecords(procedureCode)
    }
  }
}
```

### Pattern 5: Medical Data Synchronization and Replication

**Problem**: Patient data must be synchronized across multiple devices and locations while maintaining consistency and handling network partitions common in Caribbean environments.

**Solution**: Multi-master replication with medical conflict resolution and partition tolerance.

**Implementation**:
```scala
// Medical data synchronization engine
class MedicalDataSyncEngine {
  
  private val replicationManager = new MedicalReplicationManager()
  private val conflictResolver = new MedicalConflictResolver()
  private val networkManager = new CaribbeanNetworkManager()
  
  def initializeSync(
    localNode: NodeId,
    remoteNodes: List[NodeId]
  ): SyncConfiguration = {
    
    val syncConfig = SyncConfiguration(
      localNode = localNode,
      remoteNodes = remoteNodes,
      syncStrategy = SyncStrategy.MedicalPriority,
      conflictResolution = ConflictResolution.SafetyFirst,
      networkTolerance = NetworkTolerance.High // For Caribbean conditions
    )
    
    replicationManager.configure(syncConfig)
    syncConfig
  }
  
  def synchronizePatientData(
    patientId: PatientId,
    targetNodes: List[NodeId]
  ): Future[SyncResult] = {
    
    val localVersion = getLocalPatientVersion(patientId)
    
    // Get remote versions from all target nodes
    val remoteVersionFutures = targetNodes.map { nodeId =>
      getRemotePatientVersion(patientId, nodeId)
        .map(version => nodeId -> version)
        .recover { case _ => nodeId -> None }
    }
    
    Future.sequence(remoteVersionFutures).flatMap { remoteVersions =>
      val syncPlan = createSyncPlan(patientId, localVersion, remoteVersions.toMap)
      executeSyncPlan(syncPlan)
    }
  }
  
  private def createSyncPlan(
    patientId: PatientId,
    localVersion: PatientDataVersion,
    remoteVersions: Map[NodeId, Option[PatientDataVersion]]
  ): SyncPlan = {
    
    val syncActions = remoteVersions.map { case (nodeId, remoteVersionOpt) =>
      remoteVersionOpt match {
        case Some(remoteVersion) =>
          compareVersions(localVersion, remoteVersion) match {
            case VersionComparison.LocalNewer =>
              SyncAction.Push(patientId, nodeId, localVersion)
            case VersionComparison.RemoteNewer =>
              SyncAction.Pull(patientId, nodeId, remoteVersion)
            case VersionComparison.Diverged =>
              SyncAction.Merge(patientId, nodeId, localVersion, remoteVersion)
            case VersionComparison.Identical =>
              SyncAction.NoOp(patientId, nodeId)
          }
        case None =>
          // Remote node doesn't have this patient - push our data
          SyncAction.Push(patientId, nodeId, localVersion)
      }
    }.toList
    
    SyncPlan(
      patientId = patientId,
      actions = syncActions,
      priority = calculateSyncPriority(patientId),
      estimatedDuration = estimateSyncDuration(syncActions)
    )
  }
  
  private def executeSyncPlan(plan: SyncPlan): Future[SyncResult] = {
    val actionResults = plan.actions.map { action =>
      executeSyncAction(action).recover { case e =>
        SyncActionResult.Failed(action, e.getMessage)
      }
    }
    
    Future.sequence(actionResults).map { results =>
      val successful = results.count(_.isSuccess)
      val failed = results.count(_.isFailure)
      
      SyncResult(
        patientId = plan.patientId,
        totalActions = plan.actions.length,
        successfulActions = successful,
        failedActions = failed,
        conflicts = results.flatMap(_.conflicts),
        duration = results.map(_.duration).sum
      )
    }
  }
  
  private def executeSyncAction(action: SyncAction): Future[SyncActionResult] = {
    action match {
      case push: SyncAction.Push =>
        executePushAction(push)
      case pull: SyncAction.Pull =>
        executePullAction(pull)
      case merge: SyncAction.Merge =>
        executeMergeAction(merge)
      case noop: SyncAction.NoOp =>
        Future.successful(SyncActionResult.Success(noop))
    }
  }
  
  private def executeMergeAction(merge: SyncAction.Merge): Future[SyncActionResult] = {
    for {
      // Get detailed data from both versions
      localData <- getDetailedPatientData(merge.patientId, merge.localVersion)
      remoteData <- getRemoteDetailedPatientData(merge.patientId, merge.nodeId, merge.remoteVersion)
      
      // Perform three-way merge
      mergeResult <- performThreeWayMerge(localData, remoteData)
      
      // Apply merge results
      _ <- applyMergeResult(merge.patientId, mergeResult)
      
    } yield {
      SyncActionResult.Success(
        action = merge,
        conflicts = mergeResult.conflicts,
        dataChanges = mergeResult.changes
      )
    }
  }
  
  private def performThreeWayMerge(
    local: DetailedPatientData,
    remote: DetailedPatientData
  ): Future[MergeResult] = {
    
    // Find common ancestor version
    val commonAncestor = findCommonAncestor(local.version, remote.version)
    
    commonAncestor match {
      case Some(ancestor) =>
        // Perform three-way merge using common ancestor
        val localChanges = calculateChanges(ancestor, local)
        val remoteChanges = calculateChanges(ancestor, remote)
        
        val conflictingChanges = detectConflicts(localChanges, remoteChanges)
        val nonConflictingChanges = mergeNonConflictingChanges(localChanges, remoteChanges)
        
        // Resolve conflicts using medical conflict resolution
        val resolvedConflicts = conflictingChanges.map { conflict =>
          conflictResolver.resolveConflict(conflict)
        }
        
        Future.successful(MergeResult(
          changes = nonConflictingChanges ++ resolvedConflicts.map(_.resolvedChange),
          conflicts = resolvedConflicts.filter(_.requiresManualReview),
          mergeStrategy = MergeStrategy.ThreeWay
        ))
        
      case None =>
        // No common ancestor - fallback to two-way merge
        val conflicts = detectDirectConflicts(local, remote)
        val resolvedConflicts = conflicts.map { conflict =>
          conflictResolver.resolveConflict(conflict)
        }
        
        Future.successful(MergeResult(
          changes = resolvedConflicts.map(_.resolvedChange),
          conflicts = resolvedConflicts.filter(_.requiresManualReview),
          mergeStrategy = MergeStrategy.TwoWay
        ))
    }
  }
}

// Caribbean-specific network management
class CaribbeanNetworkManager {
  
  private val connectionMonitor = new NetworkConnectionMonitor()
  private val bandwidthManager = new BandwidthManager()
  
  def optimizeForCaribbeanConditions(): NetworkConfiguration = {
    val currentConditions = assessNetworkConditions()
    
    NetworkConfiguration(
      maxConcurrentConnections = calculateOptimalConnections(currentConditions),
      connectionTimeout = calculateOptimalTimeout(currentConditions),
      retryPolicy = createCaribbeanRetryPolicy(currentConditions),
      compressionLevel = selectCompressionLevel(currentConditions),
      prioritization = NetworkPrioritization.MedicalFirst
    )
  }
  
  private def assessNetworkConditions(): CaribbeanNetworkConditions = {
    val latency = connectionMonitor.measureLatency()
    val bandwidth = bandwidthManager.measureBandwidth()
    val stability = connectionMonitor.measureStability()
    val powerStatus = powerMonitor.getPowerStatus()
    
    CaribbeanNetworkConditions(
      latency = latency,
      bandwidth = bandwidth,
      stability = stability,
      isPowerStable = powerStatus.isStable,
      weatherConditions = weatherService.getCurrentConditions()
    )
  }
  
  private def createCaribbeanRetryPolicy(
    conditions: CaribbeanNetworkConditions
  ): RetryPolicy = {
    
    val baseDelay = if (conditions.stability.isLow) 5.seconds else 1.second
    val maxRetries = if (conditions.weatherConditions.hasStorm) 10 else 5
    
    RetryPolicy(
      maxRetries = maxRetries,
      baseDelay = baseDelay,
      backoffStrategy = ExponentialBackoffWithJitter,
      retryableExceptions = List(
        classOf[NetworkTimeoutException],
        classOf[ConnectionResetException],
        classOf[PowerFailureException]
      ),
      nonRetryableExceptions = List(
        classOf[AuthenticationException],
        classOf[DataCorruptionException]
      )
    )
  }
}
```

---

## 🔗 Related Patterns

- **Offline-First-Desktop-Architecture.md** - Foundation architecture patterns
- **Desktop-Healthcare-Data-Security.md** - Security patterns for local medical data
- **Jamaica-EHR-Compliance-Patterns.md** - Regulatory compliance for data handling
- **Caribbean-Desktop-Resilience-Patterns.md** - Infrastructure resilience patterns

---

## 📊 Performance Metrics and Monitoring

### Data Architecture Performance Indicators

| Metric | Target | Critical Threshold | Measurement Method |
|--------|--------|--------------------|-------------------|
| **Medical Record Retrieval Time** | < 200ms | > 1s | Time from query to display |
| **Conflict Resolution Time** | < 5s | > 30s | Time to resolve data conflicts |
| **Sync Completion Rate** | > 99% | < 95% | Successful syncs / Total sync attempts |
| **Data Integrity Verification** | 100% | < 100% | Verified records / Total records |
| **Storage Efficiency** | > 80% compression | < 60% | Compressed size / Original size |

### Health Monitoring Implementation

```scala
// Medical data health monitoring
class MedicalDataHealthMonitor {
  
  def monitorDataHealth(): HealthReport = {
    HealthReport(
      integrityScore = calculateIntegrityScore(),
      performanceMetrics = gatherPerformanceMetrics(),
      storageHealth = assessStorageHealth(),
      syncHealth = assessSyncHealth(),
      conflictResolutionHealth = assessConflictResolutionHealth()
    )
  }
  
  private def calculateIntegrityScore(): IntegrityScore = {
    val totalRecords = patientRepository.getTotalRecordCount()
    val corruptRecords = integrityChecker.findCorruptRecords().length
    val orphanedRecords = integrityChecker.findOrphanedRecords().length
    val missingSignatures = integrityChecker.findMissingSignatures().length
    
    val issues = corruptRecords + orphanedRecords + missingSignatures
    val score = ((totalRecords - issues).toDouble / totalRecords.toDouble) * 100
    
    IntegrityScore(
      score = score,
      totalRecords = totalRecords,
      issues = IntegrityIssues(corruptRecords, orphanedRecords, missingSignatures)
    )
  }
}
```

---

**Last Updated**: January 17, 2026  
**Maintained By**: Healthcare Data Architect + Clinical Systems Engineer  
**Review Frequency**: Weekly during active development, monthly during maintenance  
**Version**: 1.0.0

---

**Key Insight**: Medical data architecture in Caribbean environments requires **safety-first design principles**. Every architectural decision should prioritize patient data integrity and safety over performance or convenience. Implement multiple redundant safety mechanisms and assume that any single point of failure will eventually fail due to environmental or infrastructure challenges.
