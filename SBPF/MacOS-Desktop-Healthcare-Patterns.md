# macOS Desktop Healthcare Patterns
## macOS-Specific Implementation Strategies for Caribbean Dental Practice Software

**Purpose**: Comprehensive macOS-specific patterns for healthcare desktop applications in Caribbean environments, focusing on Cocoa/AppKit integration, Keychain services, sandboxing, and macOS-specific optimization techniques.

**Context**: macOS adoption is growing in Caribbean professional environments, particularly among younger practitioners and technology-forward practices. Applications must integrate seamlessly with the macOS ecosystem while maintaining compliance and security requirements.

**Key Principle**: **macOS-native excellence** - Embrace macOS design principles and capabilities while ensuring robust healthcare functionality and Caribbean-specific adaptations.

---

## 🍎 macOS Healthcare Ecosystem Integration

### Pattern 1: Cocoa Healthcare Application Architecture

**Problem**: Healthcare applications need deep macOS integration for optimal user experience, including Keychain integration, Spotlight indexing, and native UI components.

**Solution**: Native Cocoa application architecture using AppKit and Foundation frameworks, with healthcare-specific extensions.

**Implementation**:
```scala
// macOS-native healthcare application using Scala and JNI bridges
class MacOSHealthcareApplication {
  
  private val cocoaIntegration = new CocoaHealthcareIntegration()
  private val keychainManager = new MacOSKeychainManager()
  private val spotlightIndexer = new HealthcareSpotlightIndexer()
  private val notificationManager = new MacOSHealthcareNotifications()
  
  def initializeMacOSIntegration(): Future[MacOSIntegrationResult] = {
    for {
      // Initialize Cocoa healthcare components
      cocoaResult <- initializeCocoaHealthcare()
      
      // Setup Keychain integration
      keychainResult <- setupHealthcareKeychain()
      
      // Configure Spotlight indexing for patient data
      spotlightResult <- configureHealthcareSpotlight()
      
      // Setup native notifications
      notificationResult <- setupHealthcareNotifications()
      
    } yield MacOSIntegrationResult(cocoaResult, keychainResult, spotlightResult, notificationResult)
  }
  
  // Cocoa integration for healthcare workflows
  class CocoaHealthcareIntegration {
    
    def createHealthcareWindow(): NSWindow = {
      // Use JNI to call Objective-C Cocoa APIs
      val window = NSWindow.alloc()
      
      window.initWithContentRect(
        NSMakeRect(0, 0, 1200, 800),
        NSTitledWindowMask | NSClosableWindowMask | NSMiniaturizableWindowMask | NSResizableWindowMask,
        NSBackingStoreBuffered,
        false
      )
      
      // Set healthcare-specific window properties
      window.setTitle("Caribbean Dental Practice")
      window.setMinSize(NSMakeSize(800, 600))
      window.setDelegate(new HealthcareWindowDelegate())
      
      // Configure window for healthcare workflows
      configureHealthcareWindow(window)
      
      window
    }
    
    private def configureHealthcareWindow(window: NSWindow): Unit = {
      // Enable automatic tabbing for patient records
      window.setTabbingMode(NSWindowTabbingModePreferred)
      
      // Set appropriate content protection level
      window.setSharingType(NSWindowSharingNone) // Prevent screen sharing of patient data
      
      // Configure for healthcare accessibility
      window.setContentView(createHealthcareContentView())
    }
    
    def createHealthcareContentView(): NSView = {
      val contentView = NSView.alloc().initWithFrame(NSMakeRect(0, 0, 1200, 800))
      
      // Create healthcare-specific UI components
      val patientListView = createPatientListView()
      val chartingView = createChartingView()
      val toolbarView = createHealthcareToolbar()
      
      // Setup Auto Layout constraints for healthcare workflow
      contentView.addSubview(toolbarView)
      contentView.addSubview(patientListView)
      contentView.addSubview(chartingView)
      
      setupHealthcareLayoutConstraints(contentView, toolbarView, patientListView, chartingView)
      
      contentView
    }
    
    private def createPatientListView(): NSTableView = {
      val scrollView = NSScrollView.alloc().initWithFrame(NSMakeRect(0, 0, 300, 600))
      val tableView = NSTableView.alloc().initWithFrame(scrollView.bounds())
      
      // Configure table for patient data
      val patientColumn = NSTableColumn.alloc().initWithIdentifier("patient")
      patientColumn.setTitle("Patients")
      patientColumn.setWidth(280)
      tableView.addTableColumn(patientColumn)
      
      // Set healthcare-specific table properties
      tableView.setUsesAlternatingRowBackgroundColors(true)
      tableView.setSelectionHighlightStyle(NSTableViewSelectionHighlightStyleRegular)
      tableView.setDelegate(new PatientListDelegate())
      tableView.setDataSource(new PatientListDataSource())
      
      scrollView.setDocumentView(tableView)
      scrollView.setHasVerticalScroller(true)
      
      tableView
    }
    
    def createChartingView(): NSView = {
      val chartingView = HealthcareChartingView.alloc().initWithFrame(NSMakeRect(300, 0, 900, 600))
      
      // Configure for dental charting
      chartingView.setBackgroundColor(NSColor.whiteColor())
      chartingView.setCanDrawSubviewsIntoLayer(true) // Enable Core Animation
      
      // Add dental chart layers
      val toothDiagram = createToothDiagramLayer()
      val periodontalChart = createPeriodontalChartLayer()
      
      chartingView.layer().addSublayer(toothDiagram)
      chartingView.layer().addSublayer(periodontalChart)
      
      chartingView
    }
    
    class HealthcareWindowDelegate extends NSWindowDelegate {
      
      override def windowShouldClose(sender: NSWindow): Boolean = {
        // Check for unsaved patient data before closing
        val hasUnsavedData = checkForUnsavedPatientData()
        
        if (hasUnsavedData) {
          val alert = NSAlert.alloc().init()
          alert.setMessageText("Unsaved Patient Data")
          alert.setInformativeText("You have unsaved patient data. Do you want to save before closing?")
          alert.addButtonWithTitle("Save")
          alert.addButtonWithTitle("Don't Save")
          alert.addButtonWithTitle("Cancel")
          
          val response = alert.runModal()
          
          response match {
            case NSAlertFirstButtonReturn => // Save
              savePatientData()
              true
            case NSAlertSecondButtonReturn => // Don't Save
              true
            case _ => // Cancel
              false
          }
        } else {
          true
        }
      }
      
      override def windowDidBecomeKey(notification: NSNotification): Unit = {
        // Refresh patient data when window becomes active
        refreshPatientDisplay()
        
        // Log healthcare window activation for audit
        auditLogger.logWindowActivation("healthcare_main_window")
      }
    }
  }
  
  // macOS Keychain integration for healthcare credentials
  class MacOSKeychainManager {
    
    def storeHealthcareCredentials(service: String, account: String, password: String): Future[KeychainResult] = {
      Future {
        try {
          // Use Security framework via JNI
          val securityFramework = SecurityFramework.getInstance()
          
          val keychainItem = KeychainItem(
            service = service,
            account = account,
            password = password,
            accessGroup = "caribbean.dental.healthcare", // App-specific access group
            accessible = kSecAttrAccessibleWhenUnlockedThisDeviceOnly // High security
          )
          
          val status = securityFramework.SecItemAdd(keychainItem.toCFDictionary(), null)
          
          status match {
            case errSecSuccess =>
              KeychainResult(success = true, service = service, account = account)
            case errSecDuplicateItem =>
              // Update existing item
              val updateResult = updateKeychainItem(service, account, password)
              updateResult
            case _ =>
              KeychainResult(
                success = false,
                service = service,
                account = account,
                errorMessage = Some(s"Keychain error: ${getSecurityErrorMessage(status)}")
              )
          }
        } catch {
          case e: Exception =>
            KeychainResult(
              success = false,
              service = service,
              account = account,
              errorMessage = Some(e.getMessage)
            )
        }
      }
    }
    
    def retrieveHealthcareCredentials(service: String, account: String): Future[Option[HealthcareCredentials]] = {
      Future {
        try {
          val securityFramework = SecurityFramework.getInstance()
          
          val query = CFDictionary(
            kSecClass -> kSecClassGenericPassword,
            kSecAttrService -> service,
            kSecAttrAccount -> account,
            kSecReturnData -> kCFBooleanTrue,
            kSecReturnAttributes -> kCFBooleanTrue
          )
          
          val result = new CFTypeRef()
          val status = securityFramework.SecItemCopyMatching(query, result)
          
          status match {
            case errSecSuccess =>
              val itemData = result.toCFDictionary()
              val passwordData = itemData.getValue(kSecValueData).asInstanceOf[CFData]
              val password = passwordData.toByteArray().map(_.toChar).mkString
              
              Some(HealthcareCredentials(
                service = service,
                account = account,
                password = password,
                retrievedAt = Instant.now()
              ))
            case errSecItemNotFound =>
              None
            case _ =>
              logger.warn(s"Keychain retrieval error: ${getSecurityErrorMessage(status)}")
              None
          }
        } catch {
          case e: Exception =>
            logger.error(s"Exception retrieving keychain credentials: ${e.getMessage}")
            None
        }
      }
    }
    
    def setupHealthcareKeychainAccess(): Future[KeychainAccessSetupResult] = {
      Future {
        try {
          val securityFramework = SecurityFramework.getInstance()
          
          // Create access control for healthcare data
          val accessControl = securityFramework.SecAccessControlCreateWithFlags(
            kCFAllocatorDefault,
            kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            kSecAccessControlBiometryCurrentSet | kSecAccessControlApplicationPassword,
            null
          )
          
          if (accessControl != null) {
            // Store access control reference for healthcare operations
            setHealthcareAccessControl(accessControl)
            
            KeychainAccessSetupResult(
              success = true,
              biometryRequired = true,
              deviceOnlyAccess = true
            )
          } else {
            KeychainAccessSetupResult(
              success = false,
              errorMessage = Some("Failed to create access control")
            )
          }
        } catch {
          case e: Exception =>
            KeychainAccessSetupResult(
              success = false,
              errorMessage = Some(e.getMessage)
            )
        }
      }
    }
    
    // Touch ID integration for healthcare access
    def authenticateWithTouchID(reason: String): Future[BiometricAuthResult] = {
      Future {
        try {
          val localAuthentication = LocalAuthenticationFramework.getInstance()
          val context = localAuthentication.LAContext.alloc().init()
          
          // Check if biometric authentication is available
          val biometryError = new NSErrorPointer()
          val canEvaluate = context.canEvaluatePolicyWithError(
            LAPolicyDeviceOwnerAuthenticationWithBiometrics,
            biometryError
          )
          
          if (canEvaluate) {
            // Perform biometric authentication
            val semaphore = new CountDownLatch(1)
            var authResult: BiometricAuthResult = null
            
            context.evaluatePolicyWithLocalizedReasonReply(
              LAPolicyDeviceOwnerAuthenticationWithBiometrics,
              reason,
              new LAContextReply {
                override def call(success: Boolean, error: NSError): Unit = {
                  authResult = BiometricAuthResult(
                    success = success,
                    errorMessage = Option(error).map(_.localizedDescription())
                  )
                  semaphore.countDown()
                }
              }
            )
            
            semaphore.await(30, TimeUnit.SECONDS) // 30-second timeout
            
            authResult ?: BiometricAuthResult(
              success = false,
              errorMessage = Some("Authentication timeout")
            )
          } else {
            BiometricAuthResult(
              success = false,
              errorMessage = Some("Biometric authentication not available")
            )
          }
        } catch {
          case e: Exception =>
            BiometricAuthResult(
              success = false,
              errorMessage = Some(e.getMessage)
            )
        }
      }
    }
  }
  
  // Spotlight integration for healthcare data indexing
  class HealthcareSpotlightIndexer {
    
    def indexPatientRecords(patients: List[PatientRecord]): Future[SpotlightIndexingResult] = {
      Future {
        try {
          val coreSpotlight = CoreSpotlightFramework.getInstance()
          val searchableItems = patients.map(createSearchableItem)
          
          val indexingCompletion = new CSIndexingCompletion()
          var indexingResult: SpotlightIndexingResult = null
          val semaphore = new CountDownLatch(1)
          
          coreSpotlight.CSSearchableIndex.defaultSearchableIndex().indexSearchableItemsWithCompletionHandler(
            searchableItems.toArray,
            new CSIndexingCompletionHandler {
              override def call(error: NSError): Unit = {
                indexingResult = SpotlightIndexingResult(
                  success = error == null,
                  indexedItems = if (error == null) patients.length else 0,
                  errorMessage = Option(error).map(_.localizedDescription())
                )
                semaphore.countDown()
              }
            }
          )
          
          semaphore.await(60, TimeUnit.SECONDS) // 60-second timeout for indexing
          
          indexingResult ?: SpotlightIndexingResult(
            success = false,
            indexedItems = 0,
            errorMessage = Some("Indexing timeout")
          )
        } catch {
          case e: Exception =>
            SpotlightIndexingResult(
              success = false,
              indexedItems = 0,
              errorMessage = Some(e.getMessage)
            )
        }
      }
    }
    
    private def createSearchableItem(patient: PatientRecord): CSSearchableItem = {
      val attributeSet = CSSearchableItemAttributeSet.alloc().initWithItemContentType(kUTTypeData)
      
      // Set searchable attributes (careful with PHI)
      attributeSet.setTitle(s"Patient: ${patient.lastName}, ${patient.firstName}")
      attributeSet.setContentDescription(s"Dental patient record")
      attributeSet.setKeywords(Array(
        "patient",
        "dental",
        "healthcare",
        patient.patientId,
        // Only include non-sensitive search terms
        patient.lastVisit.getYear.toString
      ))
      
      // Set custom attributes for healthcare searches
      attributeSet.setValue("caribbean.dental.patient", forCustomKey = "category")
      attributeSet.setValue(patient.patientId, forCustomKey = "patientId")
      attributeSet.setValue(patient.lastVisit.toString, forCustomKey = "lastVisit")
      
      // Create searchable item
      val searchableItem = CSSearchableItem.alloc().initWithUniqueIdentifierDomainIdentifierAttributeSet(
        s"patient:${patient.patientId}",
        "caribbean.dental.patients",
        attributeSet
      )
      
      // Set expiration date for automatic cleanup
      searchableItem.setExpirationDate(patient.lastVisit.plus(7, ChronoUnit.YEARS)) // 7-year retention
      
      searchableItem
    }
    
    def handleSpotlightSelection(itemIdentifier: String): Future[SpotlightSelectionResult] = {
      Future {
        if (itemIdentifier.startsWith("patient:")) {
          val patientId = itemIdentifier.substring("patient:".length)
          
          // Open patient record in application
          openPatientRecord(patientId) match {
            case Success(patient) =>
              SpotlightSelectionResult(
                success = true,
                action = SpotlightAction.OpenPatientRecord,
                patientId = Some(patientId)
              )
            case Failure(e) =>
              SpotlightSelectionResult(
                success = false,
                errorMessage = Some(s"Failed to open patient record: ${e.getMessage}")
              )
          }
        } else {
          SpotlightSelectionResult(
            success = false,
            errorMessage = Some("Unknown spotlight item type")
          )
        }
      }
    }
  }
  
  // macOS-native notifications for healthcare events
  class MacOSHealthcareNotifications {
    
    def setupHealthcareNotifications(): Future[NotificationSetupResult] = {
      Future {
        try {
          val userNotifications = UserNotificationsFramework.getInstance()
          val center = userNotifications.UNUserNotificationCenter.currentNotificationCenter()
          
          // Request notification permissions
          val options = UNAuthorizationOptions.Alert | UNAuthorizationOptions.Sound | UNAuthorizationOptions.Badge
          
          val permissionSemaphore = new CountDownLatch(1)
          var permissionGranted = false
          
          center.requestAuthorizationWithOptionsCompletionHandler(
            options,
            new UNAuthorizationCompletionHandler {
              override def call(granted: Boolean, error: NSError): Unit = {
                permissionGranted = granted
                permissionSemaphore.countDown()
              }
            }
          )
          
          permissionSemaphore.await(10, TimeUnit.SECONDS)
          
          if (permissionGranted) {
            // Setup notification categories for healthcare
            setupHealthcareNotificationCategories(center)
            
            NotificationSetupResult(
              success = true,
              permissionsGranted = true
            )
          } else {
            NotificationSetupResult(
              success = false,
              permissionsGranted = false,
              errorMessage = Some("Notification permissions not granted")
            )
          }
        } catch {
          case e: Exception =>
            NotificationSetupResult(
              success = false,
              permissionsGranted = false,
              errorMessage = Some(e.getMessage)
            )
        }
      }
    }
    
    private def setupHealthcareNotificationCategories(center: UNUserNotificationCenter): Unit = {
      val appointmentReminderCategory = createAppointmentReminderCategory()
      val treatmentAlertCategory = createTreatmentAlertCategory()
      val backupCompleteCategory = createBackupCompleteCategory()
      
      val categories = Set(
        appointmentReminderCategory,
        treatmentAlertCategory,
        backupCompleteCategory
      )
      
      center.setNotificationCategories(categories)
    }
    
    private def createAppointmentReminderCategory(): UNNotificationCategory = {
      val confirmAction = UNNotificationAction.actionWithIdentifierTitleOptions(
        "CONFIRM_APPOINTMENT",
        "Confirm",
        UNNotificationActionOptions.None
      )
      
      val rescheduleAction = UNNotificationAction.actionWithIdentifierTitleOptions(
        "RESCHEDULE_APPOINTMENT",
        "Reschedule",
        UNNotificationActionOptions.Foreground
      )
      
      UNNotificationCategory.categoryWithIdentifierActionsIntentIdentifiersOptions(
        "APPOINTMENT_REMINDER",
        Array(confirmAction, rescheduleAction),
        Array.empty[String],
        UNNotificationCategoryOptions.None
      )
    }
    
    def scheduleAppointmentReminder(appointment: Appointment): Future[NotificationScheduleResult] = {
      Future {
        try {
          val userNotifications = UserNotificationsFramework.getInstance()
          val center = userNotifications.UNUserNotificationCenter.currentNotificationCenter()
          
          // Create notification content
          val content = UNMutableNotificationContent.alloc().init()
          content.setTitle("Upcoming Appointment")
          content.setBody(s"Patient: ${appointment.patientName} at ${appointment.time}")
          content.setCategoryIdentifier("APPOINTMENT_REMINDER")
          content.setSound(UNNotificationSound.defaultSound())
          
          // Add healthcare-specific user info
          content.setUserInfo(CFDictionary(
            "appointmentId" -> appointment.id,
            "patientId" -> appointment.patientId,
            "type" -> "appointment_reminder"
          ))
          
          // Schedule for 30 minutes before appointment
          val triggerDate = appointment.scheduledTime.minus(30, ChronoUnit.MINUTES)
          val dateComponents = Calendar.getInstance()
          dateComponents.setTime(Date.from(triggerDate))
          
          val trigger = UNCalendarNotificationTrigger.triggerWithDateMatchingComponentsRepeats(
            dateComponents,
            false // No repeat
          )
          
          // Create notification request
          val request = UNNotificationRequest.requestWithIdentifierContentTrigger(
            s"appointment:${appointment.id}",
            content,
            trigger
          )
          
          // Schedule notification
          val scheduleSemaphore = new CountDownLatch(1)
          var scheduleSuccess = false
          
          center.addNotificationRequestWithCompletionHandler(
            request,
            new UNNotificationRequestCompletionHandler {
              override def call(error: NSError): Unit = {
                scheduleSuccess = error == null
                scheduleSemaphore.countDown()
              }
            }
          )
          
          scheduleSemaphore.await(5, TimeUnit.SECONDS)
          
          NotificationScheduleResult(
            success = scheduleSuccess,
            notificationId = s"appointment:${appointment.id}",
            scheduledFor = triggerDate
          )
        } catch {
          case e: Exception =>
            NotificationScheduleResult(
              success = false,
              errorMessage = Some(e.getMessage)
            )
        }
      }
    }
    
    def handleNotificationResponse(response: UNNotificationResponse): Future[NotificationResponseResult] = {
      Future {
        val actionIdentifier = response.actionIdentifier()
        val userInfo = response.notification().request().content().userInfo()
        
        val appointmentId = userInfo.getValue("appointmentId").asInstanceOf[String]
        val notificationType = userInfo.getValue("type").asInstanceOf[String]
        
        (actionIdentifier, notificationType) match {
          case ("CONFIRM_APPOINTMENT", "appointment_reminder") =>
            confirmAppointment(appointmentId) match {
              case Success(_) =>
                NotificationResponseResult(
                  success = true,
                  action = NotificationAction.AppointmentConfirmed,
                  appointmentId = Some(appointmentId)
                )
              case Failure(e) =>
                NotificationResponseResult(
                  success = false,
                  errorMessage = Some(s"Failed to confirm appointment: ${e.getMessage}")
                )
            }
          
          case ("RESCHEDULE_APPOINTMENT", "appointment_reminder") =>
            // Open appointment reschedule dialog
            openRescheduleDialog(appointmentId)
            NotificationResponseResult(
              success = true,
              action = NotificationAction.OpenRescheduleDialog,
              appointmentId = Some(appointmentId)
            )
          
          case _ =>
            NotificationResponseResult(
              success = false,
              errorMessage = Some("Unknown notification action")
            )
        }
      }
    }
  }
}
```

### Pattern 2: macOS Sandboxing and Security

**Problem**: macOS applications must work within the App Sandbox while maintaining healthcare functionality and data access requirements.

**Solution**: Proper sandboxing configuration with security-scoped bookmarks and healthcare-appropriate entitlements.

**Implementation**:
```scala
// macOS Sandbox integration for healthcare applications
class MacOSSandboxManager {
  
  def configureSandboxForHealthcare(): Future[SandboxConfigurationResult] = {
    Future {
      val entitlements = createHealthcareEntitlements()
      val securityBookmarks = setupSecurityScopedBookmarks()
      val fileSystemAccess = configureFileSystemAccess()
      
      SandboxConfigurationResult(
        entitlements = entitlements,
        securityBookmarks = securityBookmarks,
        fileSystemAccess = fileSystemAccess
      )
    }
  }
  
  private def createHealthcareEntitlements(): HealthcareEntitlements = {
    HealthcareEntitlements(
      // Network access for healthcare data synchronization
      networkOutbound = true,
      networkInbound = false, // Only outbound for security
      
      // File system access for healthcare data
      userSelectedFiles = true, // Allow user to select files
      downloadsFolder = true,   // For importing healthcare data
      documentsFolder = true,   // For healthcare documents
      
      // Security entitlements
      keychainAccessGroups = List("caribbean.dental.healthcare"),
      hardwareAcceleration = true, // For medical imaging
      
      // Prevent dangerous operations
      scriptingTargets = false,
      systemPreferences = false,
      addressBook = false, // Use healthcare-specific contacts only
      
      // Healthcare-specific entitlements
      cameraAccess = true,  // For intraoral cameras
      usbAccess = true,     // For dental equipment
      bluetoothCentral = true // For wireless devices
    )
  }
  
  def createSecurityScopedBookmark(url: NSURL): Future[SecurityScopedBookmarkResult] = {
    Future {
      try {
        // Create security-scoped bookmark for healthcare data folder
        val bookmarkData = url.bookmarkDataWithOptionsIncludingResourceValuesForKeysRelativeToURLError(
          NSURLBookmarkCreationWithSecurityScope | NSURLBookmarkCreationSecurityScopeAllowOnlyReadAccess,
          null, // No resource values needed
          null, // Not relative to another URL
          null  // Error pointer
        )
        
        if (bookmarkData != null) {
          // Store bookmark for later access
          storeSecurityBookmark(url.path(), bookmarkData)
          
          SecurityScopedBookmarkResult(
            success = true,
            path = url.path(),
            bookmarkData = bookmarkData
          )
        } else {
          SecurityScopedBookmarkResult(
            success = false,
            path = url.path(),
            errorMessage = Some("Failed to create security-scoped bookmark")
          )
        }
      } catch {
        case e: Exception =>
          SecurityScopedBookmarkResult(
            success = false,
            path = url.path(),
            errorMessage = Some(e.getMessage)
          )
      }
    }
  }
  
  def accessSecurityScopedResource[T](path: String)(operation: => T): Future[ScopedAccessResult[T]] = {
    Future {
      retrieveSecurityBookmark(path) match {
        case Some(bookmarkData) =>
          try {
            // Resolve bookmark to get access
            var isStale = false
            val resolvedURL = NSURL.URLByResolvingBookmarkDataOptionsRelativeToURLBookmarkDataIsStaleError(
              bookmarkData,
              NSURLBookmarkResolutionWithSecurityScope,
              null, // Not relative to another URL
              isStale,
              null  // Error pointer
            )
            
            if (resolvedURL != null && !isStale) {
              // Start accessing security-scoped resource
              val accessGranted = resolvedURL.startAccessingSecurityScopedResource()
              
              if (accessGranted) {
                try {
                  val result = operation
                  ScopedAccessResult(success = true, result = Some(result))
                } finally {
                  // Always stop accessing the resource
                  resolvedURL.stopAccessingSecurityScopedResource()
                }
              } else {
                ScopedAccessResult[T](
                  success = false,
                  errorMessage = Some("Failed to access security-scoped resource")
                )
              }
            } else {
              ScopedAccessResult[T](
                success = false,
                errorMessage = Some("Failed to resolve security-scoped bookmark")
              )
            }
          } catch {
            case e: Exception =>
              ScopedAccessResult[T](
                success = false,
                errorMessage = Some(e.getMessage)
              )
          }
        case None =>
          ScopedAccessResult[T](
            success = false,
            errorMessage = Some("No security bookmark found for path")
          )
      }
    }
  }
  
  // Healthcare-specific file system access patterns
  class HealthcareFileSystemAccess {
    
    def requestHealthcareDataFolderAccess(): Future[FolderAccessResult] = {
      Future {
        // Use NSOpenPanel to let user select healthcare data folder
        val openPanel = NSOpenPanel.openPanel()
        openPanel.setCanChooseFiles(false)
        openPanel.setCanChooseDirectories(true)
        openPanel.setAllowsMultipleSelection(false)
        openPanel.setMessage("Select Healthcare Data Folder")
        openPanel.setPrompt("Grant Access")
        
        val response = openPanel.runModal()
        
        if (response == NSModalResponseOK) {
          val selectedURL = openPanel.URLs().firstObject().asInstanceOf[NSURL]
          
          // Create security-scoped bookmark for persistent access
          createSecurityScopedBookmark(selectedURL).map { bookmarkResult =>
            if (bookmarkResult.success) {
              FolderAccessResult(
                success = true,
                selectedPath = selectedURL.path(),
                hasPermission = true
              )
            } else {
              FolderAccessResult(
                success = false,
                errorMessage = bookmarkResult.errorMessage
              )
            }
          }.await
        } else {
          FolderAccessResult(
            success = false,
            errorMessage = Some("User cancelled folder selection")
          )
        }
      }
    }
    
    def accessHealthcareDatabase(databasePath: String): Future[DatabaseAccessResult] = {
      accessSecurityScopedResource(databasePath) {
        // Access SQLite database for healthcare data
        val connection = DriverManager.getConnection(s"jdbc:sqlite:$databasePath")
        
        try {
          // Verify it's a valid healthcare database
          val metadata = connection.getMetaData
          val tables = getTableNames(connection)
          
          val isHealthcareDb = tables.exists(table =>
            table.toLowerCase.contains("patient") ||
            table.toLowerCase.contains("appointment") ||
            table.toLowerCase.contains("treatment")
          )
          
          if (isHealthcareDb) {
            DatabaseAccessResult(
              success = true,
              databaseType = "Healthcare SQLite",
              tableCount = tables.length,
              connection = Some(connection)
            )
          } else {
            connection.close()
            DatabaseAccessResult(
              success = false,
              errorMessage = Some("Database does not appear to contain healthcare data")
            )
          }
        } catch {
          case e: Exception =>
            connection.close()
            throw e
        }
      }.map(_.result.getOrElse(DatabaseAccessResult(
        success = false,
        errorMessage = Some("Failed to access database")
      )))
    }
  }
}

// macOS-specific healthcare device integration
class MacOSHealthcareDeviceManager {
  
  def setupHealthcareUSBDevices(): Future[USBDeviceSetupResult] = {
    Future {
      try {
        // Use IOKit to enumerate healthcare USB devices
        val ioKit = IOKitFramework.getInstance()
        
        // Create matching dictionary for healthcare devices
        val matchingDict = CFDictionaryCreateMutable(
          kCFAllocatorDefault,
          0,
          kCFTypeDictionaryKeyCallBacks,
          kCFTypeDictionaryValueCallBacks
        )
        
        // Add criteria for healthcare devices (dental cameras, sensors, etc.)
        CFDictionaryAddValue(matchingDict, CFSTR(kIOProviderClassKey), CFSTR("IOUSBDevice"))
        
        // Get iterator for matching devices
        val deviceIterator = new io_iterator_t()
        val result = ioKit.IOServiceGetMatchingServices(kIOMasterPortDefault, matchingDict, deviceIterator)
        
        if (result == KERN_SUCCESS) {
          val devices = mutable.ListBuffer[HealthcareUSBDevice]()
          
          var device = ioKit.IOIteratorNext(deviceIterator)
          while (device != 0) {
            val deviceInfo = getUSBDeviceInfo(device)
            
            // Check if it's a known healthcare device
            if (isHealthcareDevice(deviceInfo)) {
              devices += createHealthcareUSBDevice(deviceInfo)
            }
            
            ioKit.IOObjectRelease(device)
            device = ioKit.IOIteratorNext(deviceIterator)
          }
          
          ioKit.IOObjectRelease(deviceIterator)
          
          USBDeviceSetupResult(
            success = true,
            devicesFound = devices.toList,
            totalDevices = devices.length
          )
        } else {
          USBDeviceSetupResult(
            success = false,
            devicesFound = List.empty,
            errorMessage = Some(s"IOServiceGetMatchingServices failed: $result")
          )
        }
      } catch {
        case e: Exception =>
          USBDeviceSetupResult(
            success = false,
            devicesFound = List.empty,
            errorMessage = Some(e.getMessage)
          )
      }
    }
  }
  
  private def isHealthcareDevice(deviceInfo: USBDeviceInfo): Boolean = {
    val healthcareVendors = Set(
      0x1234, // Example: Dental imaging device vendor
      0x5678, // Example: Intraoral camera vendor
      0x9ABC  // Example: X-ray sensor vendor
    )
    
    val healthcareProducts = Set(
      "Dental Camera",
      "Intraoral Camera",
      "X-Ray Sensor",
      "Periodontal Probe",
      "Digital Radiography"
    )
    
    healthcareVendors.contains(deviceInfo.vendorId) ||
    healthcareProducts.exists(product => 
      deviceInfo.productName.toLowerCase.contains(product.toLowerCase)
    )
  }
  
  def setupBluetoothHealthcareDevices(): Future[BluetoothDeviceSetupResult] = {
    Future {
      try {
        // Use Core Bluetooth for healthcare device discovery
        val coreBluetooth = CoreBluetoothFramework.getInstance()
        val centralManager = CBCentralManager.alloc().initWithDelegateQueueOptions(
          new HealthcareCentralManagerDelegate(),
          null, // Use main queue
          null  // No options
        )
        
        // Start scanning for healthcare devices
        val serviceUUIDs = Array(
          CBUUID.UUIDWithString("1800"), // Generic Access
          CBUUID.UUIDWithString("180F"), // Battery Service
          CBUUID.UUIDWithString("1234")  // Custom healthcare service
        )
        
        centralManager.scanForPeripheralsWithServicesOptions(serviceUUIDs, null)
        
        // Wait for discovery results
        Thread.sleep(10000) // 10 second discovery period
        
        val discoveredDevices = getDiscoveredHealthcareDevices()
        
        BluetoothDeviceSetupResult(
          success = true,
          devicesFound = discoveredDevices,
          totalDevices = discoveredDevices.length
        )
      } catch {
        case e: Exception =>
          BluetoothDeviceSetupResult(
            success = false,
            devicesFound = List.empty,
            errorMessage = Some(e.getMessage)
          )
      }
    }
  }
  
  class HealthcareCentralManagerDelegate extends CBCentralManagerDelegate {
    
    private val discoveredDevices = mutable.ListBuffer[HealthcareBluetoothDevice]()
    
    override def centralManagerDidUpdateState(central: CBCentralManager): Unit = {
      central.state() match {
        case CBManagerStatePoweredOn =>
          logger.info("Bluetooth powered on, ready for healthcare device discovery")
        case CBManagerStatePoweredOff =>
          logger.warn("Bluetooth powered off")
        case CBManagerStateUnsupported =>
          logger.error("Bluetooth not supported on this device")
        case _ =>
          logger.info(s"Bluetooth state: ${central.state()}")
      }
    }
    
    override def centralManagerDidDiscoverPeripheralAdvertisementDataRSSI(
      central: CBCentralManager,
      peripheral: CBPeripheral,
      advertisementData: NSDictionary,
      rssi: NSNumber
    ): Unit = {
      
      val deviceName = peripheral.name()
      val localName = advertisementData.objectForKey(CBAdvertisementDataLocalNameKey).asInstanceOf[String]
      
      // Check if this is a healthcare device
      if (isHealthcareBluetoothDevice(deviceName, advertisementData)) {
        val device = HealthcareBluetoothDevice(
          peripheral = peripheral,
          name = deviceName ?: localName ?: "Unknown Device",
          rssi = rssi.intValue(),
          advertisementData = advertisementData,
          discoveredAt = Instant.now()
        )
        
        discoveredDevices += device
        logger.info(s"Discovered healthcare Bluetooth device: ${device.name}")
      }
    }
    
    def getDiscoveredDevices(): List[HealthcareBluetoothDevice] = {
      discoveredDevices.toList
    }
  }
}
```

---

## 🍎 macOS Performance and Integration Patterns

### Pattern 3: Core Animation for Healthcare Visualizations

**Problem**: Healthcare applications need smooth, responsive visualizations for dental charts, X-rays, and patient data while maintaining performance on varied macOS hardware.

**Solution**: Leverage Core Animation and Metal for hardware-accelerated healthcare visualizations.

**Implementation**:
```scala
// Core Animation healthcare visualization
class HealthcareCoreAnimationManager {
  
  def createDentalChartVisualization(): CALayer = {
    val dentalChart = CALayer.layer()
    dentalChart.setBounds(CGRectMake(0, 0, 800, 600))
    dentalChart.setBackgroundColor(CGColorCreateGenericRGB(1.0f, 1.0f, 1.0f, 1.0f))
    
    // Create tooth layers with animations
    val toothLayers = (1 to 32).map(createAnimatedToothLayer)
    toothLayers.foreach(dentalChart.addSublayer)
    
    // Add periodontal chart overlay
    val perioChart = createPeriodontalChartLayer()
    dentalChart.addSublayer(perioChart)
    
    dentalChart
  }
  
  private def createAnimatedToothLayer(toothNumber: Int): CAShapeLayer = {
    val toothLayer = CAShapeLayer.layer()
    
    // Create tooth shape path
    val toothPath = createToothPath(toothNumber)
    toothLayer.setPath(toothPath)
    toothLayer.setFillColor(CGColorCreateGenericRGB(1.0f, 1.0f, 1.0f, 1.0f))
    toothLayer.setStrokeColor(CGColorCreateGenericRGB(0.0f, 0.0f, 0.0f, 1.0f))
    toothLayer.setLineWidth(2.0f)
    
    // Position tooth in dental arch
    val position = calculateToothPosition(toothNumber)
    toothLayer.setPosition(position)
    
    // Add hover animation
    setupToothHoverAnimation(toothLayer)
    
    toothLayer
  }
  
  private def setupToothHoverAnimation(toothLayer: CAShapeLayer): Unit = {
    // Create scale animation for tooth selection
    val scaleAnimation = CABasicAnimation.animationWithKeyPath("transform.scale")
    scaleAnimation.setFromValue(NSNumber.numberWithFloat(1.0f))
    scaleAnimation.setToValue(NSNumber.numberWithFloat(1.1f))
    scaleAnimation.setDuration(0.2)
    scaleAnimation.setAutoreverses(true)
    scaleAnimation.setRemovedOnCompletion(false)
    
    // Create glow animation for treatment indication
    val glowAnimation = CABasicAnimation.animationWithKeyPath("shadowRadius")
    glowAnimation.setFromValue(NSNumber.numberWithFloat(0.0f))
    glowAnimation.setToValue(NSNumber.numberWithFloat(10.0f))
    glowAnimation.setDuration(1.0)
    glowAnimation.setRepeatCount(Float.MaxValue)
    glowAnimation.setAutoreverses(true)
    
    // Store animations for later use
    toothLayer.setValue(scaleAnimation, forKey = "hoverAnimation")
    toothLayer.setValue(glowAnimation, forKey = "treatmentAnimation")
  }
  
  def animateToothTreatment(toothNumber: Int, treatmentType: TreatmentType): Unit = {
    val toothLayer = findToothLayer(toothNumber)
    
    val colorAnimation = treatmentType match {
      case TreatmentType.Filling =>
        createFillingAnimation()
      case TreatmentType.Crown =>
        createCrownAnimation()
      case TreatmentType.Extraction =>
        createExtractionAnimation()
      case TreatmentType.RootCanal =>
        createRootCanalAnimation()
    }
    
    toothLayer.addAnimation(colorAnimation, forKey = "treatmentVisualization")
  }
  
  private def createFillingAnimation(): CAKeyframeAnimation = {
    val animation = CAKeyframeAnimation.animationWithKeyPath("fillColor")
    
    val colors = Array(
      CGColorCreateGenericRGB(1.0f, 1.0f, 1.0f, 1.0f), // White (healthy)
      CGColorCreateGenericRGB(0.8f, 0.8f, 0.8f, 1.0f), // Light gray (filling)
      CGColorCreateGenericRGB(0.6f, 0.6f, 0.6f, 1.0f)  // Gray (filled)
    )
    
    animation.setValues(colors)
    animation.setKeyTimes(Array(
      NSNumber.numberWithFloat(0.0f),
      NSNumber.numberWithFloat(0.5f),
      NSNumber.numberWithFloat(1.0f)
    ))
    animation.setDuration(2.0)
    animation.setRemovedOnCompletion(false)
    animation.setFillMode(kCAFillModeForwards)
    
    animation
  }
}

// macOS-specific healthcare preferences and settings
class MacOSHealthcarePreferences {
  
  def setupHealthcarePreferences(): Future[PreferencesSetupResult] = {
    Future {
      val userDefaults = NSUserDefaults.standardUserDefaults()
      
      // Setup healthcare-specific default preferences
      val healthcareDefaults = Map(
        "AutoBackupEnabled" -> true,
        "BackupIntervalHours" -> 4,
        "EncryptPatientData" -> true,
        "RequireTouchIDForAccess" -> true,
        "SyncWithCloud" -> false, // Default off for security
        "AuditLoggingEnabled" -> true,
        "SessionTimeoutMinutes" -> 30,
        "XRayImageQuality" -> "High",
        "ToothChartAnimations" -> true,
        "ColorBlindAccessibility" -> false
      )
      
      // Register defaults
      healthcareDefaults.foreach { case (key, value) =>
        value match {
          case bool: Boolean =>
            userDefaults.registerDefaults(NSDictionary.dictionaryWithObjectForKey(
              NSNumber.numberWithBool(bool), key
            ))
          case int: Int =>
            userDefaults.registerDefaults(NSDictionary.dictionaryWithObjectForKey(
              NSNumber.numberWithInt(int), key
            ))
          case str: String =>
            userDefaults.registerDefaults(NSDictionary.dictionaryWithObjectForKey(
              NSString.stringWithString(str), key
            ))
        }
      }
      
      // Synchronize defaults
      userDefaults.synchronize()
      
      PreferencesSetupResult(
        success = true,
        preferencesSet = healthcareDefaults.size
      )
    }
  }
  
  def createHealthcarePreferencesWindow(): NSWindow = {
    val window = NSWindow.alloc().initWithContentRect(
      NSMakeRect(0, 0, 600, 500),
      NSTitledWindowMask | NSClosableWindowMask,
      NSBackingStoreBuffered,
      false
    )
    
    window.setTitle("Caribbean Dental Preferences")
    window.setContentView(createPreferencesContentView())
    
    window
  }
  
  private def createPreferencesContentView(): NSView = {
    val contentView = NSView.alloc().initWithFrame(NSMakeRect(0, 0, 600, 500))
    
    // Create tabbed preferences interface
    val tabView = NSTabView.alloc().initWithFrame(NSMakeRect(20, 20, 560, 460))
    
    // General preferences tab
    val generalTab = NSTabViewItem.alloc().initWithIdentifier("general")
    generalTab.setLabel("General")
    generalTab.setView(createGeneralPreferencesView())
    tabView.addTabViewItem(generalTab)
    
    // Security preferences tab
    val securityTab = NSTabViewItem.alloc().initWithIdentifier("security")
    securityTab.setLabel("Security")
    securityTab.setView(createSecurityPreferencesView())
    tabView.addTabViewItem(securityTab)
    
    // Backup preferences tab
    val backupTab = NSTabViewItem.alloc().initWithIdentifier("backup")
    backupTab.setLabel("Backup")
    backupTab.setView(createBackupPreferencesView())
    tabView.addTabViewItem(backupTab)
    
    contentView.addSubview(tabView)
    contentView
  }
  
  private def createSecurityPreferencesView(): NSView = {
    val securityView = NSView.alloc().initWithFrame(NSMakeRect(0, 0, 540, 420))
    
    // Touch ID toggle
    val touchIdButton = NSButton.alloc().initWithFrame(NSMakeRect(20, 380, 300, 25))
    touchIdButton.setButtonType(NSButtonTypeSwitch)
    touchIdButton.setTitle("Require Touch ID for access")
    touchIdButton.setTarget(new PreferencesController())
    touchIdButton.setAction(sel_registerName("toggleTouchID:"))
    securityView.addSubview(touchIdButton)
    
    // Encryption toggle
    val encryptionButton = NSButton.alloc().initWithFrame(NSMakeRect(20, 350, 300, 25))
    encryptionButton.setButtonType(NSButtonTypeSwitch)
    encryptionButton.setTitle("Encrypt patient data")
    encryptionButton.setTarget(new PreferencesController())
    encryptionButton.setAction(sel_registerName("toggleEncryption:"))
    securityView.addSubview(encryptionButton)
    
    // Session timeout slider
    val timeoutLabel = NSTextField.alloc().initWithFrame(NSMakeRect(20, 310, 200, 25))
    timeoutLabel.setStringValue("Session timeout (minutes):")
    timeoutLabel.setEditable(false)
    timeoutLabel.setBezeled(false)
    timeoutLabel.setDrawsBackground(false)
    securityView.addSubview(timeoutLabel)
    
    val timeoutSlider = NSSlider.alloc().initWithFrame(NSMakeRect(230, 310, 200, 25))
    timeoutSlider.setMinValue(5)
    timeoutSlider.setMaxValue(120)
    timeoutSlider.setIntValue(30)
    timeoutSlider.setTarget(new PreferencesController())
    timeoutSlider.setAction(sel_registerName("changeSessionTimeout:"))
    securityView.addSubview(timeoutSlider)
    
    securityView
  }
}
```

---

## 🔗 Related Patterns

- **Cross-Platform-Desktop-Development-Strategies.md** - Platform abstraction strategies
- **Desktop-Healthcare-Data-Security.md** - macOS-specific security implementation
- **Caribbean-Desktop-Deployment-Strategies.md** - macOS deployment specifics
- **Clinical-Desktop-UX-Patterns.md** - macOS UI guidelines and Human Interface Guidelines

---

## 📊 macOS-Specific Metrics and Monitoring

### macOS Performance Indicators

| Metric | Target | Critical Threshold | macOS API |
|--------|--------|--------------------|-----------|
| **Memory Pressure** | Low/Medium | High | NSProcessInfo.processInfo.memoryPressure |
| **Thermal State** | Normal | Critical | NSProcessInfo.processInfo.thermalState |
| **Power State** | AC Power | Battery < 20% | IOKit Power Management |
| **Sandbox Violations** | 0 | > 5 per hour | Console.app monitoring |
| **Keychain Access Time** | < 100ms | > 1000ms | CFAbsoluteTimeGetCurrent |

### Monitoring Implementation

```scala
class MacOSHealthcareMonitoring {
  
  def startMacOSSpecificMonitoring(): Unit = {
    // Monitor macOS-specific performance metrics
    scheduler.scheduleAtFixedRate(30.seconds) {
      val metrics = gatherMacOSMetrics()
      analyzeMacOSPerformance(metrics)
    }
  }
  
  private def gatherMacOSMetrics(): MacOSPerformanceMetrics = {
    val processInfo = NSProcessInfo.processInfo()
    
    MacOSPerformanceMetrics(
      memoryPressure = processInfo.memoryPressure(),
      thermalState = processInfo.thermalState(),
      batteryLevel = getBatteryLevel(),
      sandboxViolations = checkSandboxViolations(),
      keychainResponseTime = measureKeychainAccess()
    )
  }
  
  private def measureKeychainAccess(): Duration = {
    val startTime = CFAbsoluteTimeGetCurrent()
    
    // Perform a simple keychain operation
    keychainManager.retrieveHealthcareCredentials("test", "test").await
    
    val endTime = CFAbsoluteTimeGetCurrent()
    Duration.ofMillis(((endTime - startTime) * 1000).toLong)
  }
}
```

---

---

## 🔧 GraalVM Native Image Configuration for macOS

### JNA + Cocoa Reflection Configuration

**Critical**: JNA macOS/Cocoa integration requires GraalVM reflection configuration for native compilation.

**graalvm/reflect-config.json** (macOS-specific additions):
```json
[
  {
    "name": "com.sun.jna.Native",
    "allDeclaredMethods": true,
    "allDeclaredConstructors": true,
    "allPublicMethods": true
  },
  {
    "name": "com.sun.jna.Structure",
    "allDeclaredMethods": true,
    "allDeclaredConstructors": true,
    "allPublicMethods": true
  },
  {
    "name": "com.sun.jna.Pointer",
    "allDeclaredMethods": true,
    "allDeclaredConstructors": true
  },
  {
    "name": "com.sun.jna.platform.mac.CoreFoundation",
    "allDeclaredMethods": true
  },
  {
    "name": "com.sun.jna.platform.mac.SystemB",
    "allDeclaredMethods": true
  },
  {
    "name": "rococoa.cocoa.foundation.NSObject",
    "allDeclaredMethods": true,
    "allDeclaredConstructors": true
  },
  {
    "name": "rococoa.cocoa.foundation.NSString",
    "allDeclaredMethods": true
  },
  {
    "name": "rococoa.cocoa.appkit.NSWindow",
    "allDeclaredMethods": true
  },
  {
    "name": "rococoa.cocoa.appkit.NSView",
    "allDeclaredMethods": true
  }
]
```

**graalvm/jni-config.json** (macOS Objective-C bridge):
```json
[
  {
    "name": "com.sun.jna.Native",
    "methods": [{"name": "dispose"}, {"name": "invokePointer"}]
  },
  {
    "name": "org.rococoa.ObjCObjectInvocationHandler",
    "methods": [{"name": "invoke"}]
  }
]
```

**graalvm/resource-config.json** (macOS frameworks):
```json
{
  "resources": {
    "includes": [
      {"pattern": ".*\\.dylib$"},
      {"pattern": "darwin/.*"},
      {"pattern": "com/sun/jna/darwin/.*"},
      {"pattern": ".*\\.nib$"},
      {"pattern": ".*\\.strings$"}
    ]
  }
}
```

### macOS-Specific Native Image Build Options

```scala
// build.sc - macOS native image with framework linking
def nativeImageMacOS = T {
  val jar = assembly().path
  
  os.proc(
    "native-image",
    "--no-fallback",
    "-H:+ReportExceptionStackTraces",
    "--initialize-at-build-time=scala,pekko",
    s"-H:ReflectionConfigurationFiles=${reflectionConfig().path}",
    s"-H:JNIConfigurationFiles=${jniConfig().path}",
    "-H:+JNI",
    "-H:+AddAllCharsets",
    // macOS framework linking
    "-H:NativeLinkerOption=-framework",
    "-H:NativeLinkerOption=Foundation",
    "-H:NativeLinkerOption=-framework",
    "-H:NativeLinkerOption=AppKit",
    "-H:NativeLinkerOption=-framework",
    "-H:NativeLinkerOption=Security",
    "-H:NativeLinkerOption=-framework",
    "-H:NativeLinkerOption=CoreFoundation",
    s"-jar", jar,
    "caribbean-dental"
  ).call(cwd = T.dest)
  
  PathRef(T.dest / "caribbean-dental")
}
```

### Testing macOS Native Image

```bash
# Build macOS native executable
mill caribbeanDental.nativeImageMacOS

# Test Keychain integration
./out/caribbeanDental/nativeImageMacOS.dest/caribbean-dental --test-keychain

# Test Cocoa UI
./out/caribbeanDental/nativeImageMacOS.dest/caribbean-dental --test-ui

# Test Touch ID
./out/caribbeanDental/nativeImageMacOS.dest/caribbean-dental --test-touchid

# Expected: All macOS integrations work in native executable
```

### macOS Code Signing for Native Executable

```bash
# Sign native executable for distribution
codesign --force --sign "Developer ID Application: Caribbean Dental" \
  --options runtime \
  --entitlements caribbeanDental.entitlements \
  ./out/caribbeanDental/nativeImageMacOS.dest/caribbean-dental

# Verify signature
codesign --verify --verbose=4 \
  ./out/caribbeanDental/nativeImageMacOS.dest/caribbean-dental

# Notarize for distribution
xcrun notarytool submit caribbean-dental.zip \
  --apple-id "developer@caribbeandental.com" \
  --team-id "TEAMID123" \
  --password "app-specific-password" \
  --wait
```

---

**Last Updated**: January 17, 2026  
**Maintained By**: macOS Integration Specialist + Caribbean Healthcare IT  
**Review Frequency**: Quarterly and after macOS updates  
**Version**: 1.0.0

---

**Key Insight**: macOS healthcare integration requires **embracing Apple's ecosystem** while maintaining healthcare compliance and security. Leverage native macOS frameworks like Keychain, Core Animation, and User Notifications, but always respect sandbox limitations and user privacy expectations. The best macOS healthcare application feels naturally integrated with the platform while providing robust healthcare functionality. **GraalVM Native Image requires framework linking and reflection configuration**, but delivers instant startup and App Store-ready distribution.