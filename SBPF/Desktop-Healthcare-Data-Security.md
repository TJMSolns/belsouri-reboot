# Desktop Healthcare Data Security
## Security Patterns for Caribbean Dental Practice Patient Data Protection

**Purpose**: Security patterns for protecting patient healthcare data in desktop applications operating in Caribbean environments with limited IT infrastructure, variable connectivity, and regulatory requirements.

**Context**: Jamaica EHR compliance, HIPAA considerations for US territory patients, limited cybersecurity expertise, unreliable internet for security updates, and local data sovereignty requirements.

**Key Principle**: **Security through simplicity and depth** - Layer multiple simple security measures rather than relying on complex systems that may fail or be misconfigured.

---

## 🔒 Data Protection and Encryption Patterns

### Pattern 1: Multi-Layer Local Data Encryption

**Problem**: Patient data must be protected at rest, even if the computer is stolen or accessed by unauthorized persons, while maintaining performance on older Caribbean hardware.

**Solution**: Layered encryption with hardware-appropriate algorithms and automatic key management.

**Encryption Strategy**:
- **Database encryption** - Transparent database-level encryption
- **File-level encryption** - Individual patient file protection  
- **Transport encryption** - All data movement encrypted
- **Backup encryption** - Encrypted backups with separate keys
- **Memory protection** - Clear sensitive data from memory

**Implementation**:
```scala
// Multi-layer encryption service
class HealthcareDataEncryption {
  
  private val databaseEncryption = new DatabaseEncryptionService()
  private val fileEncryption = new FileEncryptionService()
  private val keyManagement = new KeyManagementService()
  
  // Hardware-appropriate encryption selection
  def selectEncryptionAlgorithm(): EncryptionAlgorithm = {
    val hardwareCapabilities = systemInfo.getCPUFeatures()
    
    hardwareCapabilities match {
      case features if features.hasAESNI =>
        // Modern CPU with AES hardware acceleration
        EncryptionAlgorithm.AES256_GCM
      case features if features.supportsSSSE3 =>
        // Older CPU but with SSSE3 support
        EncryptionAlgorithm.AES256_CBC
      case _ =>
        // Very old CPU - use ChaCha20 (faster than AES on old hardware)
        EncryptionAlgorithm.ChaCha20_Poly1305
    }
  }
  
  def encryptPatientData(patientData: PatientData): EncryptedPatientData = {
    // Generate unique encryption key for this patient
    val patientKey = keyManagement.generatePatientKey(patientData.patientId)
    
    // Encrypt different data types with appropriate methods
    val encryptedDemographics = encryptDemographics(patientData.demographics, patientKey)
    val encryptedClinical = encryptClinicalData(patientData.clinicalData, patientKey)
    val encryptedImages = encryptImages(patientData.images, patientKey)
    val encryptedDocuments = encryptDocuments(patientData.documents, patientKey)
    
    EncryptedPatientData(
      patientId = patientData.patientId, // Not encrypted (needed for indexing)
      demographics = encryptedDemographics,
      clinicalData = encryptedClinical,
      images = encryptedImages,
      documents = encryptedDocuments,
      encryptionMetadata = EncryptionMetadata(
        algorithm = selectEncryptionAlgorithm(),
        keyId = patientKey.id,
        encryptedAt = Instant.now()
      )
    )
  }
  
  private def encryptClinicalData(clinical: ClinicalData, key: EncryptionKey): EncryptedBlob = {
    // Clinical data requires highest security
    val serialized = clinicalDataSerializer.serialize(clinical)
    
    // Add integrity protection
    val withIntegrity = integrityProtection.addMAC(serialized, key)
    
    // Encrypt with authenticated encryption
    val encrypted = authenticatedEncryption.encrypt(withIntegrity, key)
    
    EncryptedBlob(encrypted)
  }
  
  private def encryptImages(images: List[MedicalImage], key: EncryptionKey): List[EncryptedImage] = {
    images.map { image =>
      // Images are large - use streaming encryption for memory efficiency
      val encryptedStream = streamingEncryption.encryptStream(image.dataStream, key)
      
      EncryptedImage(
        imageId = image.id,
        encryptedData = encryptedStream,
        originalFormat = image.format,
        encryptionAlgorithm = selectEncryptionAlgorithm()
      )
    }
  }
  
  def decryptPatientData(
    encrypted: EncryptedPatientData,
    userCredentials: UserCredentials
  ): Either[DecryptionError, PatientData] = {
    
    // Verify user has access to this patient
    if (!accessControl.canAccess(userCredentials, encrypted.patientId)) {
      return Left(DecryptionError.AccessDenied)
    }
    
    try {
      // Retrieve patient encryption key
      val patientKey = keyManagement.getPatientKey(
        patientId = encrypted.patientId,
        userCredentials = userCredentials
      )
      
      // Decrypt all data components
      val demographics = decryptDemographics(encrypted.demographics, patientKey)
      val clinical = decryptClinicalData(encrypted.clinicalData, patientKey)
      val images = decryptImages(encrypted.images, patientKey)
      val documents = decryptDocuments(encrypted.documents, patientKey)
      
      Right(PatientData(
        patientId = encrypted.patientId,
        demographics = demographics,
        clinicalData = clinical,
        images = images,
        documents = documents
      ))
      
    } catch {
      case e: KeyNotFoundException => Left(DecryptionError.KeyNotFound)
      case e: IntegrityCheckFailedException => Left(DecryptionError.DataTampered)
      case e: Exception => Left(DecryptionError.DecryptionFailed(e.getMessage))
    }
  }
}

// Secure key management for Caribbean environments
class KeyManagementService {
  
  private val masterKeyProvider = new MasterKeyProvider()
  private val keyDerivation = new KeyDerivationService()
  
  def generatePatientKey(patientId: PatientId): EncryptionKey = {
    // Derive patient-specific key from master key
    val masterKey = masterKeyProvider.getMasterKey()
    val patientSalt = generatePatientSalt(patientId)
    
    // Use PBKDF2 with high iteration count (adjusted for hardware)
    val iterations = calculateOptimalIterations()
    val derivedKey = keyDerivation.deriveKey(
      password = masterKey,
      salt = patientSalt,
      iterations = iterations,
      keyLength = 256 // 256-bit key
    )
    
    EncryptionKey(
      id = generateKeyId(patientId),
      keyData = derivedKey,
      algorithm = selectEncryptionAlgorithm(),
      createdAt = Instant.now()
    )
  }
  
  private def calculateOptimalIterations(): Int = {
    val cpuBenchmark = performCPUBenchmark()
    
    // Target 100ms key derivation time for acceptable user experience
    // while maintaining security
    cpuBenchmark.iterationsFor100ms.max(100000) // Minimum 100k iterations
  }
  
  def rotateMasterKey(): KeyRotationResult = {
    val oldMasterKey = masterKeyProvider.getMasterKey()
    val newMasterKey = masterKeyProvider.generateNewMasterKey()
    
    // Re-encrypt all patient keys with new master key
    val patientKeys = keyStore.getAllPatientKeys()
    val reencryptedKeys = mutable.ListBuffer[EncryptedPatientKey]()
    
    patientKeys.foreach { encryptedKey =>
      try {
        // Decrypt with old master key
        val decryptedKey = decryptPatientKey(encryptedKey, oldMasterKey)
        
        // Re-encrypt with new master key
        val reencryptedKey = encryptPatientKey(decryptedKey, newMasterKey)
        reencryptedKeys += reencryptedKey
        
      } catch {
        case e: Exception =>
          // Log error but continue with other keys
          logger.error(s"Failed to rotate key for patient ${encryptedKey.patientId}", e)
      }
    }
    
    // Atomic update of all keys
    keyStore.replaceAllKeys(reencryptedKeys.toList)
    
    // Securely delete old master key
    securelyDeleteKey(oldMasterKey)
    
    KeyRotationResult(
      rotatedKeys = reencryptedKeys.length,
      failedKeys = patientKeys.length - reencryptedKeys.length
    )
  }
}

// Hardware security module simulation for practices without HSM
class SoftwareHSMService {
  
  // Simulate HSM key storage using encrypted key files
  def storeKey(key: EncryptionKey, protection: KeyProtection): KeyHandle = {
    val protectedKey = protectKey(key, protection)
    
    // Store in encrypted file with tamper detection
    val keyFile = createSecureKeyFile(protectedKey)
    
    KeyHandle(
      keyId = key.id,
      filePath = keyFile.absolutePath,
      checksum = calculateChecksum(keyFile)
    )
  }
  
  private def protectKey(key: EncryptionKey, protection: KeyProtection): ProtectedKey = {
    protection match {
      case KeyProtection.Password(password) =>
        // Encrypt key with password-derived key
        val salt = generateSalt()
        val iterations = calculateOptimalIterations()
        val protectionKey = deriveKeyFromPassword(password, salt, iterations)
        
        ProtectedKey(
          encryptedKey = encrypt(key.keyData, protectionKey),
          protection = protection,
          salt = salt,
          iterations = iterations
        )
        
      case KeyProtection.Biometric(template) =>
        // Encrypt key with biometric-derived key (if available)
        val biometricKey = deriveBiometricKey(template)
        
        ProtectedKey(
          encryptedKey = encrypt(key.keyData, biometricKey),
          protection = protection,
          template = template
        )
        
      case KeyProtection.SmartCard(cardId) =>
        // Use smart card for key protection (if available)
        val cardKey = smartCardService.getDerivedKey(cardId)
        
        ProtectedKey(
          encryptedKey = encrypt(key.keyData, cardKey),
          protection = protection,
          cardId = cardId
        )
    }
  }
  
  def retrieveKey(handle: KeyHandle, credentials: UserCredentials): Either[KeyError, EncryptionKey] = {
    try {
      // Verify key file integrity
      val currentChecksum = calculateChecksum(new File(handle.filePath))
      if (currentChecksum != handle.checksum) {
        return Left(KeyError.KeyTampered)
      }
      
      // Load protected key
      val protectedKey = loadProtectedKey(handle.filePath)
      
      // Decrypt based on protection method
      val keyData = decryptProtectedKey(protectedKey, credentials)
      
      Right(EncryptionKey(
        id = handle.keyId,
        keyData = keyData,
        algorithm = protectedKey.algorithm,
        createdAt = protectedKey.createdAt
      ))
      
    } catch {
      case e: FileNotFoundException => Left(KeyError.KeyNotFound)
      case e: DecryptionException => Left(KeyError.InvalidCredentials)
      case e: Exception => Left(KeyError.RetrievalFailed(e.getMessage))
    }
  }
}
```

### Pattern 2: Secure Data Transmission and Sync

**Problem**: Patient data must be securely transmitted to cloud services or other locations despite unreliable Caribbean internet connections and potential man-in-the-middle attacks.

**Solution**: End-to-end encryption with certificate pinning and connection integrity verification.

**Implementation**:
```scala
// Secure cloud synchronization service
class SecureCloudSyncService {
  
  private val certificatePinner = new CertificatePinner()
  private val encryptionService = new EndToEndEncryption()
  private val connectionValidator = new ConnectionIntegrityValidator()
  
  def syncPatientData(
    patientData: PatientData,
    destination: SyncDestination
  ): Future[SyncResult] = {
    
    for {
      // Validate destination and connection
      _ <- validateSyncDestination(destination)
      
      // Encrypt data end-to-end (separate from transport encryption)
      encryptedData <- encryptForTransmission(patientData, destination.publicKey)
      
      // Create secure connection with certificate pinning
      connection <- createSecureConnection(destination)
      
      // Transmit with integrity verification
      result <- transmitSecurely(encryptedData, connection)
      
    } yield result
  }
  
  private def validateSyncDestination(destination: SyncDestination): Future[Unit] = {
    Future {
      // Verify destination certificate against known good certificates
      val knownCertificates = certificateStore.getKnownCertificates(destination.hostname)
      val currentCertificate = tlsService.getCertificate(destination.hostname)
      
      if (!certificatePinner.isValidCertificate(currentCertificate, knownCertificates)) {
        throw new CertificateValidationException("Certificate validation failed")
      }
      
      // Verify destination is authorized for this practice
      if (!authorizationService.isAuthorizedDestination(destination)) {
        throw new AuthorizationException("Destination not authorized")
      }
    }
  }
  
  private def encryptForTransmission(
    patientData: PatientData,
    destinationPublicKey: PublicKey
  ): Future[EncryptedTransmissionPackage] = {
    Future {
      // Generate ephemeral symmetric key for this transmission
      val ephemeralKey = cryptoRandom.generateSymmetricKey(256)
      
      // Encrypt patient data with ephemeral key
      val encryptedData = encryptionService.encryptData(patientData, ephemeralKey)
      
      // Encrypt ephemeral key with destination's public key
      val encryptedKey = publicKeyEncryption.encrypt(ephemeralKey, destinationPublicKey)
      
      // Create authenticated package
      val packageData = TransmissionPackage(
        encryptedData = encryptedData,
        encryptedKey = encryptedKey,
        sourceId = practiceConfiguration.practiceId,
        timestamp = Instant.now()
      )
      
      // Sign package with practice private key
      val signature = digitalSigning.sign(packageData, practiceConfiguration.privateKey)
      
      EncryptedTransmissionPackage(packageData, signature)
    }
  }
  
  private def createSecureConnection(destination: SyncDestination): Future[SecureConnection] = {
    Future {
      // Configure TLS with strong cipher suites
      val tlsConfig = TLSConfiguration(
        minVersion = TLSVersion.TLS12,
        cipherSuites = List(
          "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
          "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256"
        ),
        certificateValidation = CertificateValidation.Strict
      )
      
      // Create connection with certificate pinning
      val connection = tlsService.createConnection(destination.endpoint, tlsConfig)
      
      // Verify connection integrity
      connectionValidator.validateConnection(connection)
      
      SecureConnection(connection)
    }
  }
  
  private def transmitSecurely(
    data: EncryptedTransmissionPackage,
    connection: SecureConnection
  ): Future[SyncResult] = {
    
    // Split large data into chunks for reliability over poor connections
    val chunks = chunkData(data, chunkSize = 1.megabyte)
    
    val transmissionResults = chunks.zipWithIndex.map { case (chunk, index) =>
      transmitChunkWithRetry(chunk, index, connection)
    }
    
    // Wait for all chunks to complete
    Future.sequence(transmissionResults).map { results =>
      val failedChunks = results.filter(_.failed)
      
      if (failedChunks.isEmpty) {
        SyncResult.Success
      } else {
        SyncResult.PartialFailure(failedChunks.map(_.chunkIndex))
      }
    }
  }
  
  private def transmitChunkWithRetry(
    chunk: DataChunk,
    chunkIndex: Int,
    connection: SecureConnection,
    maxRetries: Int = 3
  ): Future[ChunkTransmissionResult] = {
    
    def attemptTransmission(attempt: Int): Future[ChunkTransmissionResult] = {
      connection.sendChunk(chunk).recoverWith {
        case e if attempt < maxRetries =>
          // Wait with exponential backoff before retry
          val delay = (math.pow(2, attempt) * 1000).milliseconds
          Thread.sleep(delay.toMillis)
          attemptTransmission(attempt + 1)
        case e =>
          Future.successful(ChunkTransmissionResult.Failed(chunkIndex, e))
      }
    }
    
    attemptTransmission(0)
  }
}

// Network security monitoring for Caribbean environments
class NetworkSecurityMonitor {
  
  def monitorNetworkSecurity(): Unit = {
    // Monitor for suspicious network activity
    startConnectionMonitoring()
    
    // Monitor for man-in-the-middle attacks
    startMITMDetection()
    
    // Monitor DNS integrity
    startDNSIntegrityMonitoring()
    
    // Monitor for data exfiltration
    startDataExfiltrationDetection()
  }
  
  private def startMITMDetection(): Unit = {
    val certificateMonitor = new CertificateMonitor()
    
    certificateMonitor.onCertificateChange { change =>
      val knownCertificate = certificateStore.getKnownCertificate(change.hostname)
      
      if (knownCertificate.isDefined && change.newCertificate != knownCertificate.get) {
        // Certificate changed unexpectedly - possible MITM attack
        handlePossibleMITMAttack(change)
      }
    }
  }
  
  private def handlePossibleMITMAttack(change: CertificateChange): Unit = {
    // Block connection immediately
    connectionManager.blockHost(change.hostname)
    
    // Alert user
    showSecurityAlert(
      title = "Possible Security Threat",
      message = s"The security certificate for ${change.hostname} has changed unexpectedly. " +
                "This could indicate a man-in-the-middle attack. Connection blocked for safety.",
      severity = SecurityAlertSeverity.Critical
    )
    
    // Log security event
    securityLogger.logSecurityEvent(
      eventType = SecurityEventType.PossibleMITMAttack,
      hostname = change.hostname,
      details = change,
      timestamp = Instant.now()
    )
    
    // Disable automatic syncing to this host
    syncService.disableAutomaticSync(change.hostname)
  }
}
```

---

## 🔐 Access Control and Authentication Patterns

### Pattern 3: Multi-Factor Authentication for Caribbean Context

**Problem**: Strong authentication is required for healthcare data access, but Caribbean practices may lack advanced authentication infrastructure or reliable internet for cloud-based MFA.

**Solution**: Hybrid local/cloud MFA with offline fallback options and practical security for small practices.

**Implementation**:
```scala
// Multi-factor authentication manager
class HealthcareMFAManager {
  
  private val localAuthenticators = mutable.Map[AuthenticatorType, LocalAuthenticator]()
  private val cloudMFAService = new CloudMFAService()
  
  def setupMFA(user: User, preferences: MFAPreferences): MFASetupResult = {
    val availableAuthenticators = detectAvailableAuthenticators()
    
    val selectedAuthenticators = selectOptimalAuthenticators(
      available = availableAuthenticators,
      preferences = preferences,
      practiceSize = practiceConfiguration.practiceSize
    )
    
    selectedAuthenticators.map { authenticator =>
      setupAuthenticator(user, authenticator)
    }
  }
  
  private def detectAvailableAuthenticators(): List[AuthenticatorType] = {
    val available = mutable.ListBuffer[AuthenticatorType]()
    
    // SMS (most common in Caribbean)
    if (smsService.isAvailable) {
      available += AuthenticatorType.SMS
    }
    
    // Smartphone apps (TOTP)
    available += AuthenticatorType.TOTP // Always available (offline)
    
    // Biometric authentication
    if (biometricScanner.isAvailable) {
      available += AuthenticatorType.Biometric
    }
    
    // Hardware tokens (if available)
    val hardwareTokens = usbTokenScanner.scanForTokens()
    if (hardwareTokens.nonEmpty) {
      available += AuthenticatorType.HardwareToken
    }
    
    // Backup codes (always available)
    available += AuthenticatorType.BackupCodes
    
    available.toList
  }
  
  private def selectOptimalAuthenticators(
    available: List[AuthenticatorType],
    preferences: MFAPreferences,
    practiceSize: PracticeSize
  ): List[AuthenticatorType] = {
    
    practiceSize match {
      case PracticeSize.Solo =>
        // Single practitioner - prioritize convenience with security
        List(AuthenticatorType.TOTP, AuthenticatorType.BackupCodes)
        
      case PracticeSize.Small => // 2-5 practitioners
        // Small practice - balance convenience and security
        val primary = preferences.preferredMethods.headOption.getOrElse(AuthenticatorType.TOTP)
        List(primary, AuthenticatorType.BackupCodes)
        
      case PracticeSize.Medium | PracticeSize.Large =>
        // Larger practices - stronger security requirements
        val primaryMethods = preferences.preferredMethods.take(2)
        primaryMethods :+ AuthenticatorType.BackupCodes
    }
  }
  
  def authenticate(
    user: User,
    primaryCredentials: PrimaryCredentials
  ): Future[AuthenticationResult] = {
    
    for {
      // First verify primary credentials (username/password)
      primaryResult <- validatePrimaryCredentials(user, primaryCredentials)
      
      // If primary successful, request MFA
      mfaResult <- if (primaryResult.success) {
        performMFA(user)
      } else {
        Future.successful(AuthenticationResult.PrimaryFailed)
      }
      
    } yield mfaResult
  }
  
  private def performMFA(user: User): Future[AuthenticationResult] = {
    val userMFAMethods = mfaConfigService.getUserMFAMethods(user.id)
    
    // Try methods in order of preference/availability
    performMFAWithMethods(user, userMFAMethods)
  }
  
  private def performMFAWithMethods(
    user: User,
    methods: List[MFAMethod],
    attemptedMethods: List[MFAMethod] = List.empty
  ): Future[AuthenticationResult] = {
    
    methods match {
      case Nil =>
        // No more methods available
        Future.successful(AuthenticationResult.MFAFailed)
        
      case method :: remainingMethods =>
        attemptMFAMethod(user, method).flatMap {
          case AuthenticationResult.MFASuccess =>
            Future.successful(AuthenticationResult.Success)
            
          case AuthenticationResult.MFAFailed if remainingMethods.nonEmpty =>
            // Try next method
            performMFAWithMethods(user, remainingMethods, method :: attemptedMethods)
            
          case AuthenticationResult.MFAUnavailable if remainingMethods.nonEmpty =>
            // Method unavailable (e.g., no cell signal) - try next
            performMFAWithMethods(user, remainingMethods, attemptedMethods)
            
          case _ =>
            Future.successful(AuthenticationResult.MFAFailed)
        }
    }
  }
  
  private def attemptMFAMethod(user: User, method: MFAMethod): Future[AuthenticationResult] = {
    method match {
      case sms: SMSMethod =>
        attemptSMSAuthentication(user, sms)
      case totp: TOTPMethod =>
        attemptTOTPAuthentication(user, totp)
      case biometric: BiometricMethod =>
        attemptBiometricAuthentication(user, biometric)
      case backup: BackupCodesMethod =>
        attemptBackupCodeAuthentication(user, backup)
    }
  }
  
  private def attemptSMSAuthentication(user: User, sms: SMSMethod): Future[AuthenticationResult] = {
    if (!networkMonitor.hasCellularConnectivity) {
      return Future.successful(AuthenticationResult.MFAUnavailable)
    }
    
    val code = generateSMSCode()
    
    smsService.sendCode(sms.phoneNumber, code).flatMap { sendResult =>
      if (sendResult.success) {
        // Show SMS input dialog
        val userCode = showSMSInputDialog(sms.phoneNumber)
        
        userCode match {
          case Some(inputCode) if inputCode == code =>
            Future.successful(AuthenticationResult.MFASuccess)
          case _ =>
            Future.successful(AuthenticationResult.MFAFailed)
        }
      } else {
        Future.successful(AuthenticationResult.MFAUnavailable)
      }
    }
  }
  
  private def attemptTOTPAuthentication(user: User, totp: TOTPMethod): Future[AuthenticationResult] = {
    // TOTP is always available (offline)
    val userCode = showTOTPInputDialog(totp.applicationName)
    
    userCode match {
      case Some(inputCode) =>
        val isValid = totpValidator.validateCode(inputCode, totp.secret)
        if (isValid) {
          Future.successful(AuthenticationResult.MFASuccess)
        } else {
          Future.successful(AuthenticationResult.MFAFailed)
        }
      case None =>
        Future.successful(AuthenticationResult.MFAFailed)
    }
  }
}

// Role-based access control for dental practices
class DentalRoleBasedAccessControl {
  
  case class Permission(resource: String, action: String, conditions: List[AccessCondition] = List.empty)
  
  sealed trait DentalRole {
    def permissions: Set[Permission]
    def canDelegate: Boolean
    def maxSessionDuration: Duration
  }
  
  object DentalRole {
    case object Dentist extends DentalRole {
      val permissions = Set(
        Permission("patient-records", "read"),
        Permission("patient-records", "write"),
        Permission("patient-records", "delete", List(AccessCondition.OwnPatient)),
        Permission("prescriptions", "write"),
        Permission("treatment-plans", "approve"),
        Permission("practice-settings", "read"),
        Permission("reports", "generate"),
        Permission("user-management", "read")
      )
      val canDelegate = true
      val maxSessionDuration = 12.hours // Long sessions for clinical work
    }
    
    case object Hygienist extends DentalRole {
      val permissions = Set(
        Permission("patient-records", "read"),
        Permission("patient-records", "write", List(AccessCondition.HygieneData)),
        Permission("periodontal-charts", "write"),
        Permission("prophylaxis-notes", "write"),
        Permission("patient-education", "access")
      )
      val canDelegate = false
      val maxSessionDuration = 8.hours
    }
    
    case object Assistant extends DentalRole {
      val permissions = Set(
        Permission("patient-records", "read", List(AccessCondition.CurrentAppointment)),
        Permission("patient-records", "write", List(AccessCondition.BasicData)),
        Permission("appointments", "schedule"),
        Permission("imaging", "capture"),
        Permission("treatment-notes", "write", List(AccessCondition.UnderSupervision))
      )
      val canDelegate = false
      val maxSessionDuration = 8.hours
    }
    
    case object Receptionist extends DentalRole {
      val permissions = Set(
        Permission("patient-demographics", "read"),
        Permission("patient-demographics", "write"),
        Permission("appointments", "schedule"),
        Permission("appointments", "modify"),
        Permission("billing", "generate"),
        Permission("insurance", "verify"),
        Permission("reports", "basic")
      )
      val canDelegate = false
      val maxSessionDuration = 8.hours
    }
    
    case object PracticeManager extends DentalRole {
      val permissions = Set(
        Permission("all-patient-records", "read"),
        Permission("practice-settings", "write"),
        Permission("user-management", "write"),
        Permission("reports", "all"),
        Permission("audit-logs", "read"),
        Permission("data-export", "execute"),
        Permission("backup-restore", "execute")
      )
      val canDelegate = true
      val maxSessionDuration = 8.hours
    }
  }
  
  sealed trait AccessCondition
  object AccessCondition {
    case object OwnPatient extends AccessCondition // Can only access patients assigned to user
    case object CurrentAppointment extends AccessCondition // Only during active appointment
    case object HygieneData extends AccessCondition // Only hygiene-related data
    case object BasicData extends AccessCondition // Only basic demographic/scheduling data
    case object UnderSupervision extends AccessCondition // Only when dentist is present
    case object BusinessHours extends AccessCondition // Only during practice hours
    case object OnPremisesOnly extends AccessCondition // Only from practice location
  }
  
  def checkAccess(
    user: User,
    resource: String,
    action: String,
    context: AccessContext
  ): AccessDecision = {
    
    val userPermissions = user.role.permissions
    val matchingPermissions = userPermissions.filter { permission =>
      permission.resource == resource && permission.action == action
    }
    
    if (matchingPermissions.isEmpty) {
      return AccessDecision.Denied("No permission for this resource/action")
    }
    
    // Check all conditions for matching permissions
    val conditionResults = matchingPermissions.flatMap(_.conditions).map { condition =>
      evaluateCondition(condition, user, context)
    }
    
    if (conditionResults.forall(_.allowed)) {
      AccessDecision.Allowed
    } else {
      val failedConditions = conditionResults.filter(!_.allowed).map(_.reason)
      AccessDecision.Denied(s"Access conditions not met: ${failedConditions.mkString(", ")}")
    }
  }
  
  private def evaluateCondition(
    condition: AccessCondition,
    user: User,
    context: AccessContext
  ): ConditionResult = {
    
    condition match {
      case AccessCondition.OwnPatient =>
        if (context.patientId.exists(patientService.isAssignedToUser(_, user.id))) {
          ConditionResult.allowed
        } else {
          ConditionResult.denied("Patient not assigned to user")
        }
        
      case AccessCondition.CurrentAppointment =>
        if (context.patientId.exists(appointmentService.hasActiveAppointment(_, user.id))) {
          ConditionResult.allowed
        } else {
          ConditionResult.denied("No active appointment for this patient")
        }
        
      case AccessCondition.BusinessHours =>
        val currentTime = LocalTime.now()
        val businessHours = practiceConfiguration.businessHours
        
        if (businessHours.includes(currentTime)) {
          ConditionResult.allowed
        } else {
          ConditionResult.denied("Outside business hours")
        }
        
      case AccessCondition.OnPremisesOnly =>
        if (locationService.isUserOnPremises(user.id)) {
          ConditionResult.allowed
        } else {
          ConditionResult.denied("Access only allowed from practice premises")
        }
        
      case AccessCondition.UnderSupervision =>
        if (supervisionService.isDentistPresent()) {
          ConditionResult.allowed
        } else {
          ConditionResult.denied("Dentist supervision required")
        }
        
      case _ =>
        ConditionResult.allowed // Default allow for unimplemented conditions
    }
  }
}
```

---

## 📱 Device Security and Endpoint Protection Patterns

### Pattern 4: Endpoint Security for Caribbean Desktop Environments

**Problem**: Desktop computers in Caribbean dental practices need protection from malware, unauthorized access, and data theft, often without enterprise-grade security infrastructure.

**Solution**: Lightweight endpoint protection with local threat detection and automatic security hardening.

**Implementation**:
```scala
// Lightweight endpoint protection service
class HealthcareEndpointSecurity {
  
  private val threatDetector = new LocalThreatDetector()
  private val securityHardening = new SecurityHardeningService()
  private val dataLeakProtection = new DataLeakProtectionService()
  
  def initializeEndpointSecurity(): Unit = {
    // Enable real-time threat monitoring
    startThreatMonitoring()
    
    // Apply security hardening
    applySecurityHardening()
    
    // Enable data leak protection
    enableDataLeakProtection()
    
    // Start security maintenance tasks
    startSecurityMaintenance()
  }
  
  private def startThreatMonitoring(): Unit = {
    // Monitor for suspicious processes
    threatDetector.monitorProcesses { process =>
      val threatLevel = assessProcessThreat(process)
      
      if (threatLevel >= ThreatLevel.Medium) {
        handleSuspiciousProcess(process, threatLevel)
      }
    }
    
    // Monitor file system for malware signatures
    threatDetector.monitorFileSystem { fileEvent =>
      if (fileEvent.eventType == FileEventType.Created) {
        scanNewFile(fileEvent.filePath)
      }
    }
    
    // Monitor network connections
    threatDetector.monitorNetworkConnections { connection =>
      if (isUnauthorizedConnection(connection)) {
        blockConnection(connection)
      }
    }
  }
  
  private def assessProcessThreat(process: ProcessInfo): ThreatLevel = {
    var threatScore = 0
    
    // Check against known malware signatures
    if (malwareSignatures.contains(process.hash)) {
      threatScore += 100 // Definite malware
    }
    
    // Check for suspicious behavior
    if (process.isAccessingPatientData && !process.isAuthorizedApplication) {
      threatScore += 50 // Unauthorized data access
    }
    
    if (process.isNetworkActive && process.isUnknown) {
      threatScore += 30 // Unknown process with network activity
    }
    
    if (process.isEncryptingFiles && !process.isAuthorizedEncryption) {
      threatScore += 70 // Possible ransomware
    }
    
    threatScore match {
      case score if score >= 100 => ThreatLevel.Critical
      case score if score >= 70 => ThreatLevel.High
      case score if score >= 40 => ThreatLevel.Medium
      case score if score >= 20 => ThreatLevel.Low
      case _ => ThreatLevel.None
    }
  }
  
  private def handleSuspiciousProcess(process: ProcessInfo, threatLevel: ThreatLevel): Unit = {
    threatLevel match {
      case ThreatLevel.Critical =>
        // Immediately terminate and quarantine
        processManager.terminateProcess(process.pid)
        quarantineService.quarantineFile(process.executablePath)
        showCriticalThreatAlert(process)
        
      case ThreatLevel.High =>
        // Terminate and alert user
        processManager.terminateProcess(process.pid)
        showHighThreatAlert(process)
        
      case ThreatLevel.Medium =>
        // Alert user and ask for decision
        showThreatConfirmationDialog(process) match {
          case ThreatAction.Terminate =>
            processManager.terminateProcess(process.pid)
          case ThreatAction.Allow =>
            whitelistService.addTrustedProcess(process)
          case ThreatAction.AlwaysAllow =>
            whitelistService.addPermanentlyTrustedProcess(process)
        }
    }
    
    // Log security event
    securityLogger.logThreatEvent(
      threatLevel = threatLevel,
      process = process,
      action = "DETECTED",
      timestamp = Instant.now()
    )
  }
  
  private def applySecurityHardening(): Unit = {
    // Disable unnecessary services
    disableUnnecessaryServices()
    
    // Configure Windows Defender (if available)
    configureWindowsDefender()
    
    // Harden network settings
    hardenNetworkConfiguration()
    
    // Configure automatic updates
    configureAutomaticUpdates()
    
    // Set up application whitelisting
    configureApplicationWhitelisting()
  }
  
  private def disableUnnecessaryServices(): Unit = {
    val unnecessaryServices = List(
      "Telnet", "FTP", "SNMP", "RemoteRegistry", "RemoteAssistance"
    )
    
    unnecessaryServices.foreach { service =>
      try {
        serviceManager.disableService(service)
        logger.info(s"Disabled unnecessary service: $service")
      } catch {
        case e: Exception =>
          logger.warn(s"Could not disable service $service: ${e.getMessage}")
      }
    }
  }
  
  private def configureApplicationWhitelisting(): Unit = {
    val trustedApplications = List(
      // Practice management software
      practiceConfiguration.applicationPath,
      
      // Common medical applications
      "C:\\Program Files\\*\\Dexis\\*.exe",
      "C:\\Program Files\\*\\Schick\\*.exe",
      "C:\\Program Files\\*\\Carestream\\*.exe",
      
      // System applications
      "C:\\Windows\\System32\\*.exe",
      "C:\\Windows\\SysWOW64\\*.exe",
      
      // Office applications (if used)
      "C:\\Program Files\\Microsoft Office\\*\\*.exe",
      
      // Browsers (for web-based applications)
      "C:\\Program Files\\*\\Chrome\\Application\\chrome.exe",
      "C:\\Program Files\\*\\Firefox\\firefox.exe"
    )
    
    applicationWhitelist.setTrustedApplications(trustedApplications)
    applicationWhitelist.setDefaultAction(WhitelistAction.Block)
  }
}

// Data leak protection for patient information
class DataLeakProtectionService {
  
  private val patientDataClassifier = new PatientDataClassifier()
  private val networkMonitor = new NetworkTrafficMonitor()
  
  def enableDataLeakProtection(): Unit = {
    // Monitor outbound network traffic for patient data
    monitorOutboundTraffic()
    
    // Monitor clipboard for sensitive data
    monitorClipboardActivity()
    
    // Monitor USB device usage
    monitorUSBActivity()
    
    // Monitor email and messaging applications
    monitorCommunicationApplications()
  }
  
  private def monitorOutboundTraffic(): Unit = {
    networkMonitor.onOutboundTraffic { traffic =>
      val dataClassification = patientDataClassifier.classifyData(traffic.payload)
      
      if (dataClassification.containsPatientData) {
        handlePotentialDataLeak(
          LeakSource.NetworkTraffic,
          dataClassification,
          traffic.destination
        )
      }
    }
  }
  
  private def handlePotentialDataLeak(
    source: LeakSource,
    classification: DataClassification,
    destination: Any
  ): Unit = {
    
    val riskLevel = assessLeakRisk(classification, destination)
    
    riskLevel match {
      case RiskLevel.Critical =>
        // Block immediately and alert
        blockDataTransmission(source, destination)
        showCriticalDataLeakAlert(classification, destination)
        
      case RiskLevel.High =>
        // Ask user for confirmation
        showDataLeakConfirmationDialog(classification, destination) match {
          case LeakAction.Allow =>
            logDataTransmission(source, classification, destination, "ALLOWED_BY_USER")
          case LeakAction.Block =>
            blockDataTransmission(source, destination)
        }
        
      case RiskLevel.Medium =>
        // Log but allow
        logDataTransmission(source, classification, destination, "MONITORED")
        
      case RiskLevel.Low =>
        // Allow silently
        // No action needed
    }
  }
  
  private def assessLeakRisk(
    classification: DataClassification,
    destination: Any
  ): RiskLevel = {
    
    var riskScore = 0
    
    // Risk based on data sensitivity
    classification.dataTypes.foreach {
      case DataType.SSN => riskScore += 50
      case DataType.MedicalRecord => riskScore += 40
      case DataType.PersonalContact => riskScore += 20
      case DataType.Demographics => riskScore += 10
      case _ => // No additional risk
    }
    
    // Risk based on destination
    destination match {
      case email: EmailDestination if !email.isInternal =>
        riskScore += 30 // External email
      case network: NetworkDestination if !network.isTrusted =>
        riskScore += 40 // Untrusted network destination
      case usb: USBDevice if !usb.isEncrypted =>
        riskScore += 25 // Unencrypted USB
      case _ => // No additional risk
    }
    
    riskScore match {
      case score if score >= 80 => RiskLevel.Critical
      case score if score >= 60 => RiskLevel.High
      case score if score >= 30 => RiskLevel.Medium
      case _ => RiskLevel.Low
    }
  }
}
```

---

## 🔗 Related Patterns

- **Jamaica-EHR-Compliance-Patterns.md** - Regulatory compliance requirements for data security
- **Offline-First-Desktop-Architecture.md** - Local data security when offline
- **Caribbean-Desktop-Resilience-Patterns.md** - Physical security considerations
- **Clinical-Desktop-UX-Patterns.md** - Secure user interface patterns

---

## 📊 Security Metrics and Compliance Tracking

### Security Performance Metrics

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| **Authentication Success Rate** | > 99% | Successful logins / Total login attempts |
| **MFA Adoption Rate** | > 95% | Users with MFA enabled / Total users |
| **Data Encryption Coverage** | 100% | Encrypted data / Total patient data |
| **Security Incident Response Time** | < 15 minutes | Time from detection to containment |
| **Vulnerability Patching Time** | < 48 hours | Time from vulnerability disclosure to patch |

### Compliance Monitoring

```scala
// Automated compliance monitoring
class ComplianceMonitor {
  
  def generateComplianceReport(): ComplianceReport = {
    ComplianceReport(
      jamaicaEHR = assessJamaicaEHRCompliance(),
      hipaa = assessHIPAACompliance(),
      dataProtection = assessDataProtectionCompliance(),
      accessControl = assessAccessControlCompliance(),
      auditTrail = assessAuditTrailCompliance()
    )
  }
  
  private def assessJamaicaEHRCompliance(): ComplianceAssessment = {
    val requirements = List(
      checkPatientConsentManagement(),
      checkDataSovereignty(),
      checkAuditLogging(),
      checkDataRetention(),
      checkSecurityControls()
    )
    
    ComplianceAssessment(
      framework = "Jamaica EHR",
      requirements = requirements,
      overallScore = requirements.map(_.score).sum / requirements.length
    )
  }
}
```

---

**Last Updated**: January 17, 2026  
**Maintained By**: Healthcare Security Architect + Compliance Officer  
**Review Frequency**: Monthly and after security incidents  
**Version**: 1.0.0

---

**Key Insight**: Healthcare data security in Caribbean environments requires **practical defense in depth**. Layer multiple simple, robust security measures rather than depending on complex systems that may fail due to power outages, limited IT support, or network connectivity issues. Always provide offline security capabilities and assume that internet-dependent security services may be unavailable.