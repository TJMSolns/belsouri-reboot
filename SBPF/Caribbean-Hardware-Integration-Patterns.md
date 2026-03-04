# Caribbean Hardware Integration Patterns
## Medical Device and Peripheral Integration for Caribbean Dental Practices

**Purpose**: Patterns for integrating dental equipment, imaging devices, and practice management hardware in Caribbean environments with variable power, connectivity, and vendor support.

**Context**: Mixed vendor ecosystems, limited local support, custom import requirements, aging equipment, and non-standard electrical configurations common in Caribbean healthcare.

**Key Principle**: **Assume vendor independence** - Design integration patterns that work without vendor support, use standard protocols where possible, and provide fallback options for proprietary systems.

---

## 🦷 Dental Equipment Integration Patterns

### Pattern 1: Digital Radiography Integration

**Problem**: Caribbean dental practices use various digital X-ray sensors from different vendors, often with proprietary software that doesn't integrate with practice management systems.

**Solution**: Vendor-neutral DICOM integration with fallback image capture and standardized workflow integration.

**Common Caribbean Digital X-Ray Systems**:
- **Dexis** - Popular but expensive, proprietary integration
- **Schick CDR** - Common in established practices
- **Sopro** - European import, limited Caribbean support
- **Generic sensors** - Chinese/Korean imports, minimal software
- **Film scanners** - Converting existing film X-rays

**Implementation**:
```scala
// Vendor-neutral digital radiography integration
trait DigitalRadiographyDevice {
  def deviceId: String
  def vendorName: String
  def supportedFormats: List[ImageFormat]
  def isConnected: Boolean
  
  def captureImage(settings: CaptureSettings): Future[RadiographicImage]
  def getDeviceStatus(): DeviceStatus
  def calibrate(): Future[CalibrationResult]
}

// DICOM-compliant implementation
class DICOMRadiographyDevice(
  val deviceId: String,
  val vendorName: String,
  dicomConfig: DICOMConfiguration
) extends DigitalRadiographyDevice {
  
  private val dicomService = new DICOMService(dicomConfig)
  
  def captureImage(settings: CaptureSettings): Future[RadiographicImage] = {
    for {
      // Capture via DICOM protocol
      dicomImage <- dicomService.captureImage(settings)
      
      // Convert to standard format
      standardImage <- convertToStandardFormat(dicomImage)
      
      // Add practice metadata
      enrichedImage <- addPracticeMetadata(standardImage, settings)
      
    } yield enrichedImage
  }
  
  private def convertToStandardFormat(dicomImage: DICOMImage): Future[RadiographicImage] = {
    Future {
      RadiographicImage(
        patientId = dicomImage.getPatientId,
        toothNumber = extractToothNumber(dicomImage.getStudyDescription),
        captureDate = dicomImage.getStudyDate,
        imageData = dicomImage.getPixelData,
        format = ImageFormat.JPEG2000, // Lossless compression
        metadata = RadiographicMetadata(
          kvp = dicomImage.getKVP,
          mas = dicomImage.getExposure,
          exposureTime = dicomImage.getExposureTime
        )
      )
    }
  }
}

// Fallback image capture for non-DICOM devices
class GenericImageCaptureDevice(
  val deviceId: String,
  captureMethod: ImageCaptureMethod
) extends DigitalRadiographyDevice {
  
  def captureImage(settings: CaptureSettings): Future[RadiographicImage] = {
    captureMethod match {
      case ImageCaptureMethod.ScreenCapture =>
        captureViaScreenshot(settings)
      case ImageCaptureMethod.DirectoryWatch =>
        captureViaFileWatcher(settings)
      case ImageCaptureMethod.TWAIN =>
        captureViaTWAIN(settings)
      case ImageCaptureMethod.Manual =>
        captureViaManualImport(settings)
    }
  }
  
  private def captureViaScreenshot(settings: CaptureSettings): Future[RadiographicImage] = {
    Future {
      // Show instruction dialog for user
      showCaptureInstructions(
        "1. Position X-ray on vendor software screen",
        "2. Click 'Capture' when ready",
        "3. System will automatically capture the image"
      )
      
      // Wait for user confirmation
      waitForUserConfirmation()
      
      // Capture screenshot of specific screen region
      val screenshot = screenCapture.captureRegion(settings.captureRegion)
      
      // Convert to radiographic format
      convertScreenshotToRadiographic(screenshot, settings)
    }
  }
  
  private def captureViaFileWatcher(settings: CaptureSettings): Future[RadiographicImage] = {
    // Watch vendor software export directory
    val watchDir = settings.watchDirectory
    val fileWatcher = new FileWatcher(watchDir)
    
    fileWatcher.watchForNewFiles(
      extensions = List(".jpg", ".png", ".tiff", ".dcm"),
      timeout = 30.seconds
    ).map { newFile =>
      // Import and standardize the captured file
      importAndStandardize(newFile, settings)
    }
  }
}

// Multi-vendor device manager
class RadiographyDeviceManager {
  
  private val connectedDevices = mutable.Map[String, DigitalRadiographyDevice]()
  
  def scanForDevices(): List[DigitalRadiographyDevice] = {
    val detectedDevices = mutable.ListBuffer[DigitalRadiographyDevice]()
    
    // Scan for DICOM devices
    val dicomDevices = DICOMDeviceScanner.scan()
    detectedDevices ++= dicomDevices
    
    // Scan for TWAIN devices
    val twainDevices = TWAINDeviceScanner.scan()
    detectedDevices ++= twainDevices
    
    // Check for known vendor software installations
    val vendorDevices = VendorSoftwareDetector.detectInstalled()
    detectedDevices ++= vendorDevices
    
    detectedDevices.toList
  }
  
  def getPreferredDevice(): Option[DigitalRadiographyDevice] = {
    // Priority: DICOM > TWAIN > Vendor-specific > Manual
    connectedDevices.values.toList.sortBy(_.priority).headOption
  }
  
  def captureImageFromBestAvailableDevice(
    patientId: PatientId,
    toothNumber: ToothNumber,
    imageType: RadiographicImageType
  ): Future[RadiographicImage] = {
    
    getPreferredDevice() match {
      case Some(device) =>
        val settings = CaptureSettings(patientId, toothNumber, imageType)
        device.captureImage(settings)
        
      case None =>
        // No devices available - offer manual import
        offerManualImageImport(patientId, toothNumber, imageType)
    }
  }
}

// Image quality validation for Caribbean conditions
class RadiographicImageValidator {
  
  def validateImageQuality(image: RadiographicImage): ImageQualityReport = {
    val report = ImageQualityReport()
    
    // Check exposure quality
    report.exposure = validateExposure(image)
    
    // Check contrast and density
    report.contrast = validateContrast(image)
    
    // Check for motion artifacts
    report.sharpness = validateSharpness(image)
    
    // Check for proper positioning
    report.positioning = validatePositioning(image)
    
    // Caribbean-specific checks
    report.tropicalArtifacts = checkForTropicalArtifacts(image)
    
    report
  }
  
  private def checkForTropicalArtifacts(image: RadiographicImage): ArtifactReport = {
    val artifacts = mutable.ListBuffer[ImageArtifact]()
    
    // Check for humidity-related artifacts
    if (detectCondensationArtifacts(image)) {
      artifacts += ImageArtifact.HumidityCondensation
    }
    
    // Check for electrical interference (common with unstable power)
    if (detectElectricalInterference(image)) {
      artifacts += ImageArtifact.ElectricalNoise
    }
    
    // Check for dust or debris (common in tropical environments)
    if (detectDebrisArtifacts(image)) {
      artifacts += ImageArtifact.SensorContamination
    }
    
    ArtifactReport(artifacts.toList)
  }
}
```

### Pattern 2: Intraoral Camera Integration

**Problem**: Intraoral cameras from various vendors need to integrate seamlessly with patient records, with many using proprietary capture software.

**Solution**: Universal camera integration with standard USB Video Class (UVC) support and vendor-specific adaptations.

**Common Caribbean Intraoral Camera Systems**:
- **Carestream CS 1500** - Professional grade, TWAIN compatible
- **Dexis Iris** - Integrated with Dexis software
- **MouthWatch** - Telemedicine focused
- **Generic USB cameras** - Cost-effective Chinese imports
- **Smartphone adapters** - Creative low-cost solutions

**Implementation**:
```scala
// Universal intraoral camera interface
trait IntraoralCamera {
  def cameraId: String
  def resolution: CameraResolution
  def hasAutoFocus: Boolean
  def hasLEDIllumination: Boolean
  
  def startPreview(): Future[CameraPreview]
  def captureImage(): Future[IntraoralImage]
  def captureVideo(duration: Duration): Future[IntraoralVideo]
  def adjustSettings(settings: CameraSettings): Future[Unit]
}

// USB Video Class (UVC) camera implementation
class UVCIntraoralCamera(
  val cameraId: String,
  val devicePath: String
) extends IntraoralCamera {
  
  private val uvcDevice = new UVCDevice(devicePath)
  
  def startPreview(): Future[CameraPreview] = {
    Future {
      // Configure camera for intraoral use
      uvcDevice.setResolution(CameraResolution.HD720p)
      uvcDevice.setFrameRate(30)
      uvcDevice.setAutoWhiteBalance(true)
      uvcDevice.setBrightness(0.7) // Slightly bright for intraoral
      
      // Start video stream
      val videoStream = uvcDevice.startVideoStream()
      
      CameraPreview(videoStream)
    }
  }
  
  def captureImage(): Future[IntraoralImage] = {
    for {
      preview <- startPreview()
      rawFrame <- preview.getCurrentFrame()
      processedImage <- processIntraoralImage(rawFrame)
    } yield processedImage
  }
  
  private def processIntraoralImage(rawFrame: VideoFrame): Future[IntraoralImage] = {
    Future {
      // Apply intraoral-specific processing
      val enhanced = imageProcessor.enhanceIntraoralImage(rawFrame)
      
      // Apply color correction for dental photography
      val colorCorrected = colorCorrector.applyDentalColorProfile(enhanced)
      
      // Adjust for typical intraoral lighting conditions
      val lightingAdjusted = lightingProcessor.adjustForIntraoralLighting(colorCorrected)
      
      IntraoralImage(
        imageData = lightingAdjusted,
        captureTimestamp = Instant.now(),
        cameraSettings = getCurrentCameraSettings(),
        processingApplied = List("enhancement", "color_correction", "lighting_adjustment")
      )
    }
  }
}

// Smartphone camera integration (common in Caribbean practices)
class SmartphoneIntraoralAdapter extends IntraoralCamera {
  
  private val qrCodeGenerator = new QRCodeGenerator()
  private val wirelessReceiver = new WirelessImageReceiver()
  
  def startPreview(): Future[CameraPreview] = {
    Future {
      // Generate QR code for smartphone connection
      val connectionCode = generateConnectionCode()
      val qrCode = qrCodeGenerator.generate(connectionCode)
      
      // Show QR code to user
      showSmartphoneConnectionDialog(qrCode)
      
      // Wait for smartphone connection
      val connectedDevice = wirelessReceiver.waitForConnection(connectionCode)
      
      SmartphonePreview(connectedDevice)
    }
  }
  
  def captureImage(): Future[IntraoralImage] = {
    // Send capture command to connected smartphone
    wirelessReceiver.sendCaptureCommand().flatMap { captureResponse =>
      // Receive and process image from smartphone
      processSmartphoneImage(captureResponse.imageData)
    }
  }
  
  private def showSmartphoneConnectionDialog(qrCode: BufferedImage): Unit = {
    val dialog = new SmartphoneConnectionDialog()
    
    dialog.setInstructions(
      "1. Install 'DentalCam' app on your smartphone",
      "2. Open the app and scan this QR code",
      "3. Position phone camera for intraoral photography",
      "4. Use the app's capture button or return to computer"
    )
    
    dialog.setQRCode(qrCode)
    dialog.show()
  }
}

// Camera calibration for consistent results
class IntraoralCameraCalibrator {
  
  def calibrateCamera(camera: IntraoralCamera): CalibrationResult = {
    // Use standard dental color reference card
    val referenceCard = DentalColorReference.VitaClassical
    
    showCalibrationInstructions(
      "1. Place Vita Classical color reference card in mouth",
      "2. Ensure even illumination across the card", 
      "3. Click 'Capture Reference' when ready"
    )
    
    val referenceImage = camera.captureImage().get() // Blocking for calibration
    
    // Analyze color accuracy
    val colorAnalysis = analyzeColorAccuracy(referenceImage, referenceCard)
    
    // Generate correction profile
    val correctionProfile = generateColorCorrectionProfile(colorAnalysis)
    
    // Test the calibration
    val testResult = testCalibration(camera, correctionProfile)
    
    CalibrationResult(
      success = testResult.isAcceptable,
      colorProfile = correctionProfile,
      accuracy = testResult.accuracy,
      recommendations = generateCalibrationRecommendations(testResult)
    )
  }
}
```

### Pattern 3: Dental Equipment Serial Communication

**Problem**: Many dental devices use RS-232 serial communication, which is increasingly rare on modern computers, especially in Caribbean practices using mixed hardware.

**Solution**: USB-to-serial adapters with robust communication protocols and error handling for unstable connections.

**Common Serial Dental Equipment**:
- **Periodontal probes** - Florida Probe, Idexx
- **Apex locators** - Root ZX series
- **Curing light meters** - Demetron, Kerr
- **Handpiece monitors** - KaVo, NSK
- **Digital scales** - Patient weight, medication dosing

**Implementation**:
```scala
// Robust serial communication for dental devices
class DentalSerialDevice(
  val deviceName: String,
  val serialPort: String,
  val baudRate: Int = 9600,
  val protocol: SerialProtocol
) {
  
  private val serialConnection = new SerialConnection(serialPort, baudRate)
  private val messageQueue = new ConcurrentLinkedQueue[SerialMessage]()
  
  def initialize(): Future[Boolean] = {
    Future {
      try {
        // Open serial port with Caribbean-appropriate timeouts
        serialConnection.open(
          readTimeout = 5000,  // 5 second timeout (longer for unstable power)
          writeTimeout = 3000  // 3 second write timeout
        )
        
        // Test communication
        val pingResult = pingDevice()
        
        if (pingResult.isSuccess) {
          // Start message processing
          startMessageProcessor()
          true
        } else {
          logger.warn(s"Device $deviceName failed ping test: ${pingResult.error}")
          false
        }
        
      } catch {
        case e: SerialException =>
          logger.error(s"Failed to initialize serial device $deviceName", e)
          false
      }
    }
  }
  
  def sendCommand(command: SerialCommand): Future[SerialResponse] = {
    val promise = Promise[SerialResponse]()
    
    try {
      // Add checksum for data integrity (important for electrical noise)
      val commandWithChecksum = protocol.addChecksum(command)
      
      // Send command with retry logic
      sendWithRetry(commandWithChecksum, maxRetries = 3) match {
        case Success(response) =>
          promise.success(response)
        case Failure(error) =>
          promise.failure(error)
      }
      
    } catch {
      case e: Exception =>
        promise.failure(e)
    }
    
    promise.future
  }
  
  private def sendWithRetry(command: SerialCommand, maxRetries: Int): Try[SerialResponse] = {
    var attempt = 0
    var lastError: Option[Throwable] = None
    
    while (attempt < maxRetries) {
      try {
        // Send command
        serialConnection.write(command.toByteArray)
        
        // Wait for response
        val response = waitForResponse(command.expectedResponseLength, timeout = 2000)
        
        // Validate response checksum
        if (protocol.validateChecksum(response)) {
          return Success(SerialResponse(response))
        } else {
          throw new SerialException("Invalid checksum in response")
        }
        
      } catch {
        case e: Exception =>
          lastError = Some(e)
          attempt += 1
          
          if (attempt < maxRetries) {
            // Wait before retry (exponential backoff)
            Thread.sleep(100 * attempt)
          }
      }
    }
    
    Failure(lastError.getOrElse(new SerialException("All retry attempts failed")))
  }
  
  private def waitForResponse(expectedLength: Int, timeout: Long): Array[Byte] = {
    val buffer = new Array[Byte](expectedLength)
    var bytesRead = 0
    val startTime = System.currentTimeMillis()
    
    while (bytesRead < expectedLength && (System.currentTimeMillis() - startTime) < timeout) {
      val available = serialConnection.bytesAvailable()
      
      if (available > 0) {
        val toRead = math.min(available, expectedLength - bytesRead)
        val newBytes = serialConnection.read(toRead)
        System.arraycopy(newBytes, 0, buffer, bytesRead, newBytes.length)
        bytesRead += newBytes.length
      } else {
        Thread.sleep(10) // Small delay to avoid busy waiting
      }
    }
    
    if (bytesRead < expectedLength) {
      throw new SerialTimeoutException(s"Timeout waiting for response: got $bytesRead of $expectedLength bytes")
    }
    
    buffer
  }
}

// Florida Probe integration example
class FloridaProbeDevice extends DentalSerialDevice(
  deviceName = "Florida Probe",
  serialPort = detectFloridaProbePort(),
  baudRate = 9600,
  protocol = FloridaProbeProtocol
) {
  
  def startPeriodontalCharting(patientId: PatientId): Future[PeriodontalChartingSession] = {
    for {
      // Initialize probe for new patient
      _ <- sendCommand(FloridaProbeCommand.InitializePatient(patientId))
      
      // Start charting mode
      _ <- sendCommand(FloridaProbeCommand.StartCharting)
      
      // Create session to receive measurements
      session <- createChartingSession()
      
    } yield session
  }
  
  private def createChartingSession(): Future[PeriodontalChartingSession] = {
    Future {
      val session = new PeriodontalChartingSession()
      
      // Listen for probe measurements
      startMeasurementListener { measurement =>
        session.addMeasurement(
          toothNumber = measurement.toothNumber,
          site = measurement.site,
          depth = measurement.depth,
          bleeding = measurement.bleedingOnProbing,
          suppuration = measurement.suppuration
        )
      }
      
      session
    }
  }
}

// USB-to-Serial adapter management for Caribbean environments
class USBSerialAdapterManager {
  
  def detectAvailableAdapters(): List[USBSerialAdapter] = {
    val systemPorts = SerialPortList.getPortNames()
    
    systemPorts.map { portName =>
      val adapter = identifyAdapter(portName)
      
      // Test adapter reliability (important for Caribbean power conditions)
      val reliabilityScore = testAdapterReliability(adapter)
      
      adapter.copy(reliabilityScore = reliabilityScore)
    }.toList
  }
  
  private def testAdapterReliability(adapter: USBSerialAdapter): Double = {
    val testIterations = 100
    var successfulTests = 0
    
    // Open connection
    val connection = new SerialConnection(adapter.portName, 9600)
    
    try {
      connection.open()
      
      for (i <- 1 to testIterations) {
        try {
          // Send test data
          val testData = s"TEST$i".getBytes()
          connection.write(testData)
          
          // Read back (loopback test if available)
          val response = connection.read(testData.length, timeout = 100)
          
          if (response.sameElements(testData)) {
            successfulTests += 1
          }
          
        } catch {
          case _: Exception => // Count as failure
        }
      }
      
    } finally {
      connection.close()
    }
    
    successfulTests.toDouble / testIterations
  }
  
  def recommendBestAdapter(adapters: List[USBSerialAdapter]): Option[USBSerialAdapter] = {
    adapters
      .filter(_.reliabilityScore > 0.95) // Only highly reliable adapters
      .sortBy(-_.reliabilityScore) // Highest reliability first
      .headOption
  }
}
```

---

## 🔌 Power and Connectivity Integration Patterns

### Pattern 4: UPS and Power Management Integration

**Problem**: Dental equipment requires stable power, but Caribbean power infrastructure is unreliable. UPS systems need to coordinate with practice management software.

**Solution**: UPS monitoring and coordinated shutdown procedures to protect both computers and dental equipment.

**Common Caribbean UPS Systems**:
- **APC Smart-UPS** - Professional grade with network monitoring
- **CyberPower** - Cost-effective with USB monitoring
- **Tripp Lite** - Hospital grade for medical equipment
- **Local Caribbean brands** - Variable quality, limited monitoring
- **Generator backup** - Whole-practice backup power

**Implementation**:
```scala
// UPS monitoring and coordination
class UPSPowerManager {
  
  private val upsDevices = mutable.Map[String, UPSDevice]()
  private val protectedEquipment = mutable.ListBuffer[ProtectedDevice]()
  
  def scanForUPSDevices(): List[UPSDevice] = {
    val detectedUPS = mutable.ListBuffer[UPSDevice]()
    
    // Scan for USB-connected UPS devices
    val usbUPS = USBDeviceScanner.scanForUPS()
    detectedUPS ++= usbUPS
    
    // Scan for network-connected UPS devices (SNMP)
    val networkUPS = NetworkUPSScanner.scanSubnet()
    detectedUPS ++= networkUPS
    
    // Check for APC PowerChute software
    val apcUPS = APCPowerChuteInterface.detectInstalled()
    detectedUPS ++= apcUPS
    
    detectedUPS.toList
  }
  
  def registerProtectedEquipment(device: ProtectedDevice): Unit = {
    protectedEquipment += device
    
    // Assign device to appropriate UPS
    val assignedUPS = findBestUPSForDevice(device)
    device.assignToUPS(assignedUPS)
  }
  
  def startPowerMonitoring(): Unit = {
    upsDevices.values.foreach { ups =>
      ups.onPowerEvent { event =>
        handlePowerEvent(ups, event)
      }
      
      ups.onBatteryLevelChange { level =>
        handleBatteryLevelChange(ups, level)
      }
      
      ups.onLoadChange { load =>
        handleLoadChange(ups, load)
      }
    }
  }
  
  private def handlePowerEvent(ups: UPSDevice, event: PowerEvent): Unit = {
    event match {
      case PowerEvent.PowerLoss =>
        handlePowerLoss(ups)
      case PowerEvent.PowerRestored =>
        handlePowerRestored(ups)
      case PowerEvent.OverloadDetected =>
        handleOverload(ups)
      case PowerEvent.BatteryTestFailed =>
        handleBatteryFailure(ups)
    }
  }
  
  private def handlePowerLoss(ups: UPSDevice): Unit = {
    logger.info(s"Power loss detected on UPS ${ups.deviceId}")
    
    // Estimate runtime based on current load
    val estimatedRuntime = ups.estimateRuntimeMinutes()
    
    // Notify all protected devices
    protectedEquipment.filter(_.assignedUPS == ups.deviceId).foreach { device =>
      device.notifyPowerLoss(estimatedRuntime)
    }
    
    // Plan coordinated shutdown if runtime is low
    if (estimatedRuntime < 10) { // Less than 10 minutes
      planCoordinatedShutdown(ups, estimatedRuntime)
    }
  }
  
  private def planCoordinatedShutdown(ups: UPSDevice, runtimeMinutes: Int): Unit = {
    val shutdownPlan = createShutdownPlan(ups, runtimeMinutes)
    
    shutdownPlan.phases.foreach { phase =>
      scheduleShutdownPhase(phase)
    }
  }
  
  private def createShutdownPlan(ups: UPSDevice, runtimeMinutes: Int): ShutdownPlan = {
    val protectedDevs = protectedEquipment.filter(_.assignedUPS == ups.deviceId)
    
    // Phase 1: Save data and notify users (75% of runtime)
    val dataPhase = ShutdownPhase(
      name = "Data Preservation",
      startTime = (runtimeMinutes * 0.75).minutes,
      actions = List(
        SaveAllPatientData,
        NotifyActiveUsers,
        DisableNewSessions
      )
    )
    
    // Phase 2: Shutdown non-critical equipment (50% of runtime)
    val nonCriticalPhase = ShutdownPhase(
      name = "Non-Critical Shutdown",
      startTime = (runtimeMinutes * 0.50).minutes,
      actions = protectedDevs
        .filter(_.priority == DevicePriority.NonCritical)
        .map(dev => ShutdownDevice(dev.deviceId))
    )
    
    // Phase 3: Shutdown critical equipment (25% of runtime)
    val criticalPhase = ShutdownPhase(
      name = "Critical Shutdown", 
      startTime = (runtimeMinutes * 0.25).minutes,
      actions = protectedDevs
        .filter(_.priority == DevicePriority.Critical)
        .map(dev => ShutdownDevice(dev.deviceId))
    )
    
    // Phase 4: Emergency system shutdown (5% of runtime)
    val emergencyPhase = ShutdownPhase(
      name = "Emergency System Shutdown",
      startTime = (runtimeMinutes * 0.05).minutes,
      actions = List(
        EmergencyDataSave,
        ForceSystemShutdown
      )
    )
    
    ShutdownPlan(List(dataPhase, nonCriticalPhase, criticalPhase, emergencyPhase))
  }
}

// Generator power coordination
class GeneratorPowerCoordinator {
  
  def detectGeneratorPower(): GeneratorStatus = {
    val voltagePattern = voltageMonitor.getVoltagePattern()
    val frequencyPattern = frequencyMonitor.getFrequencyPattern()
    
    (voltagePattern, frequencyPattern) match {
      case (VoltagePattern.Stable, FrequencyPattern.Stable) =>
        GeneratorStatus.NotRunning // Grid power
      case (VoltagePattern.SlightlyVariable, FrequencyPattern.SlightlyVariable) =>
        GeneratorStatus.RunningStable // Good generator
      case (VoltagePattern.Variable, _) | (_, FrequencyPattern.Variable) =>
        GeneratorStatus.RunningUnstable // Poor quality generator
      case (VoltagePattern.Unstable, _) | (_, FrequencyPattern.Unstable) =>
        GeneratorStatus.Critical // Dangerous generator operation
    }
  }
  
  def adaptToGeneratorPower(status: GeneratorStatus): Unit = {
    status match {
      case GeneratorStatus.RunningStable =>
        // Minor adaptations for generator power
        enableGeneratorMode()
      case GeneratorStatus.RunningUnstable =>
        // Protective measures for poor generator
        enableProtectiveMode()
      case GeneratorStatus.Critical =>
        // Emergency measures for dangerous generator
        enableEmergencyProtection()
      case GeneratorStatus.NotRunning =>
        // Normal grid power operation
        disableGeneratorAdaptations()
    }
  }
  
  private def enableProtectiveMode(): Unit = {
    // Increase UPS sensitivity to voltage variations
    upsManager.setVoltageSensitivity(UPSSensitivity.High)
    
    // Enable aggressive power conditioning
    powerConditioner.enableStrictFiltering()
    
    // Reduce load on generator
    loadManager.enableLoadReduction()
    
    // Show generator power warning
    statusDisplay.showGeneratorWarning("Running on unstable generator power - extra protection enabled")
  }
}
```

### Pattern 5: Network Equipment Integration

**Problem**: Caribbean dental practices often have limited and unreliable networking equipment that needs to coordinate with practice management software for connectivity-dependent features.

**Solution**: Network quality monitoring with adaptive functionality based on available bandwidth and stability.

**Common Caribbean Network Equipment**:
- **Consumer routers** - Linksys, Netgear, TP-Link (most common)
- **ISP-provided modems** - Variable quality, limited management
- **Cellular hotspots** - Digicel, Flow backup connectivity
- **Satellite internet** - Hughes Net, Viasat for remote areas
- **Point-to-point wireless** - Island-to-island connectivity

**Implementation**:
```scala
// Network quality monitoring and adaptation
class NetworkQualityMonitor {
  
  private val networkInterfaces = NetworkInterface.getNetworkInterfaces.asScala.toList
  private val qualityMetrics = new NetworkQualityMetrics()
  
  def startNetworkMonitoring(): Unit = {
    // Monitor primary interface
    val primaryInterface = getPrimaryNetworkInterface()
    
    if (primaryInterface.isDefined) {
      monitorInterface(primaryInterface.get)
    }
    
    // Monitor all available connections
    networkInterfaces.foreach { interface =>
      if (interface.isUp && !interface.isLoopback) {
        monitorInterface(interface)
      }
    }
  }
  
  private def monitorInterface(interface: NetworkInterface): Unit = {
    // Bandwidth monitoring
    startBandwidthMonitoring(interface)
    
    // Latency monitoring
    startLatencyMonitoring(interface)
    
    // Stability monitoring (packet loss)
    startStabilityMonitoring(interface)
    
    // Quality-based adaptation
    startQualityBasedAdaptation(interface)
  }
  
  private def startBandwidthMonitoring(interface: NetworkInterface): Unit = {
    val bandwidthTester = new BandwidthTester(interface)
    
    // Test bandwidth every 5 minutes
    val scheduler = Executors.newScheduledThreadPool(1)
    scheduler.scheduleAtFixedRate(() => {
      val bandwidth = bandwidthTester.measureBandwidth()
      qualityMetrics.recordBandwidth(interface.getName, bandwidth)
      
      adaptToBandwidth(bandwidth)
    }, 0, 5, TimeUnit.MINUTES)
  }
  
  private def adaptToBandwidth(bandwidth: BandwidthMeasurement): Unit = {
    bandwidth.downloadMbps match {
      case speed if speed < 1.0 => // Very slow connection
        enableMinimalBandwidthMode()
      case speed if speed < 5.0 => // Slow connection
        enableLowBandwidthMode()
      case speed if speed < 25.0 => // Moderate connection
        enableStandardBandwidthMode()
      case _ => // Fast connection
        enableHighBandwidthMode()
    }
  }
  
  private def enableLowBandwidthMode(): Unit = {
    // Disable automatic cloud sync
    cloudSyncService.pauseAutoSync()
    
    // Reduce image quality for uploads
    imageUploadService.setCompressionLevel(80) // 80% compression
    
    // Disable video features
    videoCallService.disable()
    
    // Enable data usage warnings
    dataUsageMonitor.enableWarnings(threshold = 100.megabytes)
    
    showBandwidthNotification("Low bandwidth detected - optimizing for data conservation")
  }
  
  private def startLatencyMonitoring(interface: NetworkInterface): Unit = {
    val latencyTester = new LatencyTester(interface)
    
    // Test latency to key servers every minute
    val keyServers = List(
      "8.8.8.8",      // Google DNS
      "1.1.1.1",      // Cloudflare DNS  
      "practice-management-cloud.com", // Our cloud service
      "local-gateway" // Router/gateway
    )
    
    keyServers.foreach { server =>
      latencyTester.startContinuousMonitoring(server) { latency =>
        qualityMetrics.recordLatency(server, latency)
        
        if (latency > 500) { // High latency
          adaptToHighLatency(server, latency)
        }
      }
    }
  }
  
  private def adaptToHighLatency(server: String, latency: Duration): Unit = {
    if (server.contains("practice-management-cloud")) {
      // High latency to our cloud - enable local-first mode
      enableLocalFirstMode()
    }
    
    if (server == "local-gateway" && latency > 100) {
      // High latency to local network - possible equipment issues
      suggestNetworkTroubleshooting()
    }
  }
}

// Caribbean ISP integration and failover
class CaribbeanISPManager {
  
  private val availableConnections = mutable.ListBuffer[InternetConnection]()
  
  def scanAvailableConnections(): List[InternetConnection] = {
    val connections = mutable.ListBuffer[InternetConnection]()
    
    // Wired connections (DSL, Cable, Fiber)
    val wiredConnections = scanWiredConnections()
    connections ++= wiredConnections
    
    // Cellular connections (Digicel, Flow, etc.)
    val cellularConnections = scanCellularConnections()
    connections ++= cellularConnections
    
    // Satellite connections
    val satelliteConnections = scanSatelliteConnections()
    connections ++= satelliteConnections
    
    // Backup connections (tethered phones, etc.)
    val backupConnections = scanBackupConnections()
    connections ++= backupConnections
    
    connections.toList
  }
  
  private def scanCellularConnections(): List[CellularConnection] = {
    val detectedConnections = mutable.ListBuffer[CellularConnection]()
    
    // Common Caribbean cellular providers
    val caribbeanProviders = List(
      "Digicel", "Flow", "bmobile", "LIME", "Cable & Wireless"
    )
    
    // Scan for USB cellular modems
    val usbModems = USBDeviceScanner.scanForCellularModems()
    
    usbModems.foreach { modem =>
      val provider = identifyCellularProvider(modem)
      if (caribbeanProviders.contains(provider)) {
        detectedConnections += CellularConnection(
          deviceId = modem.deviceId,
          provider = provider,
          signalStrength = modem.getSignalStrength(),
          dataAllowance = estimateDataAllowance(provider)
        )
      }
    }
    
    // Scan for built-in cellular (tablets, laptops)
    val builtinCellular = scanBuiltinCellular()
    detectedConnections ++= builtinCellular
    
    detectedConnections.toList
  }
  
  def setupAutomaticFailover(): Unit = {
    val primaryConnection = selectPrimaryConnection()
    val backupConnections = selectBackupConnections()
    
    // Monitor primary connection health
    networkMonitor.onConnectionFailure(primaryConnection) { failure =>
      handleConnectionFailure(primaryConnection, failure, backupConnections)
    }
    
    // Set up connection priority
    connectionManager.setPriority(primaryConnection, Priority.Primary)
    backupConnections.zipWithIndex.foreach { case (connection, index) =>
      connectionManager.setPriority(connection, Priority.Backup(index))
    }
  }
  
  private def handleConnectionFailure(
    primary: InternetConnection,
    failure: ConnectionFailure,
    backups: List[InternetConnection]
  ): Unit = {
    
    logger.warn(s"Primary connection ${primary.name} failed: ${failure.reason}")
    
    // Try to restore primary connection
    if (primary.canAutoRecover) {
      primary.attemptRecovery()
    }
    
    // Failover to best available backup
    val bestBackup = selectBestAvailableConnection(backups)
    
    bestBackup match {
      case Some(backup) =>
        connectionManager.switchToConnection(backup)
        showFailoverNotification(s"Switched to backup connection: ${backup.name}")
        
        // Continue trying to restore primary in background
        scheduleConnectionRecovery(primary)
        
      case None =>
        // No backup available - enable offline mode
        enableOfflineMode()
        showOfflineNotification("All internet connections failed - operating offline")
    }
  }
}
```

---

## 🔗 Related Patterns

- **Caribbean-Desktop-Resilience-Patterns.md** - Power and environmental protection for integrated hardware
- **Offline-First-Desktop-Architecture.md** - Local data management when hardware integration fails
- **Desktop-Healthcare-Data-Security.md** - Securing data transmitted through integrated devices
- **Clinical-Desktop-UX-Patterns.md** - User interface patterns for hardware interaction

---

## 📊 Integration Success Metrics

### Hardware Integration Quality Metrics

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| **Device Detection Rate** | > 95% | Percentage of connected devices automatically detected |
| **Communication Reliability** | > 99% | Successful command/response rate over 1000 operations |
| **Integration Setup Time** | < 10 minutes | Time from device connection to operational integration |
| **Data Loss Rate** | < 0.1% | Percentage of device data lost due to integration failures |
| **Power Event Recovery** | < 30 seconds | Time to restore device communication after power event |

### Caribbean-Specific Reliability Metrics

| Environmental Factor | Reliability Target | Mitigation Strategy |
|---------------------|-------------------|-------------------|
| **Voltage Fluctuation** | Operate 85V-140V | UPS integration + software monitoring |
| **Humidity Resistance** | 95% RH continuous | Device environmental monitoring |
| **Temperature Variation** | 20°C-40°C ambient | Thermal management + device protection |
| **Power Outage Recovery** | 100% data preservation | Coordinated UPS shutdown procedures |
| **Electrical Noise** | < 1% communication errors | Enhanced error correction + retry logic |

---

**Last Updated**: January 17, 2026  
**Maintained By**: Hardware Integration Specialist + Caribbean Operations Engineer  
**Review Frequency**: Quarterly and after major equipment installations  
**Version**: 1.0.0

---

**Key Insight**: Caribbean hardware integration requires **vendor independence and electrical resilience**. Design for common hardware failures, use standard protocols where possible, and always provide manual fallback procedures. The most sophisticated integration is worthless if it fails during a power surge or when vendor support is unavailable.