# Caribbean Desktop Resilience Patterns
## Hardware and Infrastructure Adaptation for Caribbean Dental Practices

**Purpose**: Patterns for building desktop applications that gracefully handle Caribbean infrastructure challenges including power instability, hardware variability, and limited IT support.

**Context**: Tropical climate, frequent power outages, aging hardware, unreliable internet, limited local technical support, and varying electrical infrastructure across islands.

**Key Principle**: **Assume failure, design for recovery** - Every system component must degrade gracefully and recover automatically.

---

## ⚡ Power and Electrical Resilience Patterns

### Pattern 1: Power Outage Recovery

**Problem**: Caribbean islands experience frequent power outages (daily in some areas) that can corrupt data and disrupt clinical workflows.

**Solution**: Multi-layered power failure protection with graceful degradation and automatic recovery.

**Power Failure Scenarios**:
- **Momentary outages** (< 1 second) - UPS handles, no user impact
- **Short outages** (1 second - 30 minutes) - UPS + battery backup
- **Extended outages** (30+ minutes) - Graceful shutdown, data preservation
- **Power surges** - Surge protection, hardware isolation

**Implementation**:
```scala
// Power monitoring and management system
class PowerResilienceManager {
  
  private val powerMonitor = new PowerMonitor()
  private val dataProtectionService = new DataProtectionService()
  private val batteryMonitor = new BatteryMonitor()
  
  def initializePowerProtection(): Unit = {
    // Monitor UPS status via USB/serial connection
    powerMonitor.onPowerEvent { event =>
      event match {
        case PowerEvent.PowerLoss =>
          handlePowerLoss()
        case PowerEvent.PowerRestored =>
          handlePowerRestored()
        case PowerEvent.LowBattery =>
          handleLowBattery()
        case PowerEvent.CriticalBattery =>
          handleCriticalBattery()
      }
    }
    
    // Auto-save active work every 30 seconds
    startAutoSaveTimer()
    
    // Monitor system battery (laptops/tablets)
    if (batteryMonitor.hasBattery) {
      batteryMonitor.onBatteryLevelChange { level =>
        if (level < 20) {
          enablePowerSavingMode()
        }
        if (level < 10) {
          initiateEmergencyShutdown()
        }
      }
    }
  }
  
  private def handlePowerLoss(): Unit = {
    // Immediately save all unsaved work
    dataProtectionService.saveAllUnsavedData()
    
    // Switch to low-power mode
    enableLowPowerMode()
    
    // Notify user of power loss
    showPowerLossNotification()
    
    // Start graceful shutdown timer
    scheduleGracefulShutdown(15 * 60 * 1000) // 15 minutes
  }
  
  private def handleLowBattery(): Unit = {
    // Save current state more frequently
    increaseAutoSaveFrequency(10000) // Every 10 seconds
    
    // Reduce screen brightness
    systemControls.setBrightness(30)
    
    // Disable non-essential features
    disableNonEssentialFeatures()
    
    showLowBatteryWarning()
  }
  
  private def handleCriticalBattery(): Unit = {
    // Emergency save everything
    dataProtectionService.emergencySaveAll()
    
    // Show critical battery dialog
    val dialog = new CriticalBatteryDialog()
    val action = dialog.showAndWait()
    
    action match {
      case DialogAction.Continue =>
        // User chooses to continue - their risk
        scheduleForceShutdown(2 * 60 * 1000) // 2 minutes
      case DialogAction.SaveAndShutdown =>
        initiateGracefulShutdown()
      case DialogAction.EmergencyShutdown =>
        initiateEmergencyShutdown()
    }
  }
  
  private def enableLowPowerMode(): Unit = {
    // Reduce CPU usage
    threadPoolManager.reduceThreadPoolSize()
    
    // Disable animations
    animationManager.disableAllAnimations()
    
    // Reduce screen refresh rate
    displayManager.setRefreshRate(30) // 30Hz instead of 60Hz
    
    // Pause background sync
    syncManager.pauseBackgroundSync()
    
    // Close non-essential windows
    windowManager.closeNonEssentialWindows()
  }
}

// Crash-resistant data protection
class DataProtectionService {
  
  private val journalWriter = new TransactionJournal()
  private val backupManager = new RollingBackupManager()
  
  def saveAllUnsavedData(): Unit = {
    val unsavedData = collectAllUnsavedData()
    
    unsavedData.foreach { data =>
      try {
        // Write to transaction journal first
        journalWriter.writeTransaction(data)
        
        // Then save to primary storage
        saveToStorage(data)
        
        // Mark transaction as complete
        journalWriter.completeTransaction(data.id)
        
      } catch {
        case e: Exception =>
          // Log error but continue with other data
          logger.error(s"Failed to save ${data.id}", e)
      }
    }
  }
  
  def emergencySaveAll(): Unit = {
    // Ultra-fast save - minimum data required for recovery
    val criticalData = collectCriticalData()
    
    // Save to multiple locations for redundancy
    criticalData.foreach { data =>
      Future {
        saveToLocation(data, PrimaryStorage)
        saveToLocation(data, BackupStorage) 
        saveToLocation(data, EmergencyStorage)
      }
    }
    
    // Write recovery marker
    writeRecoveryMarker()
  }
  
  // Recovery after power restoration
  def recoverFromPowerFailure(): RecoveryResult = {
    if (hasRecoveryMarker()) {
      // Emergency recovery needed
      recoverFromEmergencyState()
    } else if (hasIncompleteTransactions()) {
      // Normal recovery from transaction log
      recoverFromTransactionLog()
    } else {
      RecoveryResult.NoRecoveryNeeded
    }
  }
}

// Power-aware hardware monitoring
class HardwareMonitor {
  
  private val sensorReader = new HardwareSensorReader()
  
  def monitorHardwareHealth(): Unit = {
    // Monitor CPU temperature (important in tropical climate)
    sensorReader.onTemperatureChange { temp =>
      if (temp > 80) { // 80°C threshold
        enableThermalProtection()
      }
      if (temp > 90) { // Critical temperature
        initiateEmergencyCooling()
      }
    }
    
    // Monitor disk health
    sensorReader.onDiskHealth { health =>
      if (health.smartStatus == DiskStatus.Failing) {
        initiateDataMigration()
        alertUserToReplaceHardware()
      }
    }
    
    // Monitor memory usage (important for older hardware)
    sensorReader.onMemoryPressure { pressure =>
      if (pressure > 85) {
        enableMemoryConservationMode()
      }
    }
  }
  
  private def enableThermalProtection(): Unit = {
    // Reduce CPU usage
    processManager.setThrottleMode(true)
    
    // Increase fan speed if controllable
    fanController.setMaxSpeed()
    
    // Pause intensive operations
    backgroundTaskManager.pauseIntensiveTasks()
    
    // Show cooling warning
    showThermalWarning()
  }
}
```

### Pattern 2: Voltage Regulation and Surge Protection

**Problem**: Caribbean electrical infrastructure often has unstable voltage that can damage sensitive computer equipment.

**Solution**: Software-level voltage monitoring with hardware protection recommendations.

**Electrical Infrastructure Challenges**:
- **Voltage fluctuations** - 110V ±20V common
- **Brown-outs** - Sustained low voltage
- **Surges** - Lightning, generator switching
- **Frequency variations** - Generator-supplied power

**Implementation**:
```scala
// Voltage monitoring integration
class VoltageMonitor {
  
  private val upsInterface = new UPSInterface()
  private val voltageThresholds = VoltageThresholds(
    criticalLow = 90.0,   // 90V
    warningLow = 100.0,   // 100V  
    warningHigh = 130.0,  // 130V
    criticalHigh = 140.0  // 140V
  )
  
  def startVoltageMonitoring(): Unit = {
    if (upsInterface.isAvailable) {
      upsInterface.onVoltageReading { voltage =>
        processVoltageReading(voltage)
      }
    } else {
      // Software-only estimation based on system behavior
      startSoftwareVoltageEstimation()
    }
  }
  
  private def processVoltageReading(voltage: Double): Unit = {
    voltage match {
      case v if v < voltageThresholds.criticalLow =>
        handleCriticalLowVoltage(v)
      case v if v < voltageThresholds.warningLow =>
        handleLowVoltage(v)
      case v if v > voltageThresholds.criticalHigh =>
        handleCriticalHighVoltage(v)
      case v if v > voltageThresholds.warningHigh =>
        handleHighVoltage(v)
      case v =>
        handleNormalVoltage(v)
    }
  }
  
  private def handleCriticalLowVoltage(voltage: Double): Unit = {
    // Brown-out condition - risk of data corruption
    showCriticalVoltageAlert(voltage)
    
    // Immediately save all work
    dataProtectionService.emergencySaveAll()
    
    // Reduce power consumption
    enableUltraLowPowerMode()
    
    // Consider graceful shutdown
    if (voltage < 85.0) {
      initiateGracefulShutdown()
    }
  }
  
  private def handleHighVoltage(voltage: Double): Unit = {
    // Surge condition - risk of hardware damage
    showSurgeProtectionAlert(voltage)
    
    // Log the surge event
    surgeLogger.logSurgeEvent(voltage, System.currentTimeMillis())
    
    // If sustained, recommend shutdown
    if (isSustainedHighVoltage(voltage)) {
      recommendShutdownForSafety()
    }
  }
}

// Generator power detection
class GeneratorPowerDetector {
  
  def detectGeneratorPower(): PowerSource = {
    val voltageStability = measureVoltageStability()
    val frequencyStability = measureFrequencyStability()
    
    (voltageStability, frequencyStability) match {
      case (VoltageStability.High, FrequencyStability.High) =>
        PowerSource.GridPower
      case (VoltageStability.Medium, FrequencyStability.Medium) =>
        PowerSource.QualityGenerator
      case (VoltageStability.Low, _) | (_, FrequencyStability.Low) =>
        PowerSource.PoorQualityGenerator
      case _ =>
        PowerSource.Unknown
    }
  }
  
  def adaptToGeneratorPower(): Unit = {
    val powerSource = detectGeneratorPower()
    
    powerSource match {
      case PowerSource.PoorQualityGenerator =>
        // Enable extra protection for poor generator power
        enableGeneratorProtectionMode()
      case PowerSource.QualityGenerator =>
        // Minor adaptations for generator power
        enableGeneratorAdaptationMode()
      case PowerSource.GridPower =>
        // Normal operation
        enableNormalPowerMode()
      case PowerSource.Unknown =>
        // Conservative approach - assume worst case
        enableMaximumProtectionMode()
    }
  }
  
  private def enableGeneratorProtectionMode(): Unit = {
    // Increase auto-save frequency
    autoSaveManager.setInterval(15000) // Every 15 seconds
    
    // Enable voltage monitoring alerts
    voltageMonitor.setAlertThreshold(more_sensitive = true)
    
    // Reduce power-intensive operations
    graphicsManager.setReducedQuality(true)
    
    // Show generator power indicator
    statusBar.showPowerSourceIndicator(PowerSource.PoorQualityGenerator)
  }
}
```

---

## 🌡️ Climate and Environmental Resilience Patterns

### Pattern 3: Tropical Climate Adaptation

**Problem**: High temperatures, humidity, salt air, and storms affect computer hardware reliability and performance.

**Solution**: Environmental monitoring with adaptive behavior and maintenance guidance.

**Tropical Environmental Challenges**:
- **High temperature** (30-35°C ambient) - Overheating risk
- **High humidity** (80-95%) - Condensation and corrosion
- **Salt air** (coastal areas) - Corrosion acceleration
- **Dust and debris** - Fan clogging, component fouling
- **Hurricanes/storms** - Extended outages, surge damage

**Implementation**:
```scala
// Environmental monitoring system
class TropicalEnvironmentMonitor {
  
  private val temperatureSensor = new TemperatureSensor()
  private val humiditySensor = new HumiditySensor() 
  private val weatherService = new LocalWeatherService()
  
  def startEnvironmentalMonitoring(): Unit = {
    // Monitor internal temperature
    temperatureSensor.onTemperatureChange { temp =>
      handleTemperatureChange(temp)
    }
    
    // Monitor humidity (if available)
    if (humiditySensor.isAvailable) {
      humiditySensor.onHumidityChange { humidity =>
        handleHumidityChange(humidity)
      }
    }
    
    // Monitor weather conditions
    weatherService.onWeatherUpdate { weather =>
      handleWeatherConditions(weather)
    }
    
    // Seasonal maintenance reminders
    scheduleSeasonalMaintenance()
  }
  
  private def handleTemperatureChange(temperature: Double): Unit = {
    temperature match {
      case temp if temp > 85.0 => // Very hot
        enableAggressiveCooling()
      case temp if temp > 75.0 => // Warm
        enableModerateCooling()
      case temp if temp < 60.0 => // Cool (A/C running)
        enableNormalOperation()
      case _ =>
        maintainCurrentCoolingMode()
    }
  }
  
  private def enableAggressiveCooling(): Unit = {
    // Reduce CPU usage to decrease heat generation
    processManager.setProcessorThrottle(75) // 75% max CPU
    
    // Disable GPU-intensive operations
    graphicsManager.setLowPerformanceMode(true)
    
    // Increase fan speed if controllable
    fanController.setTargetSpeed(100)
    
    // Pause background tasks that generate heat
    backgroundProcessor.pauseIntensiveTasks()
    
    // Show thermal status warning
    showThermalManagementStatus("High Temperature Mode")
  }
  
  private def handleHumidityChange(humidity: Double): Unit = {
    humidity match {
      case h if h > 90.0 => // Very high humidity
        enableHighHumidityProtection()
      case h if h > 75.0 => // High humidity  
        enableMediumHumidityProtection()
      case h if h < 40.0 => // Low humidity (rare in Caribbean)
        enableLowHumidityProtection()
      case _ =>
        maintainNormalHumidityMode()
    }
  }
  
  private def enableHighHumidityProtection(): Unit = {
    // Increase internal airflow if possible
    fanController.setMinimumSpeed(50) // Keep fans running
    
    // Monitor for condensation indicators
    condensationDetector.enableMonitoring()
    
    // Avoid rapid temperature changes
    thermalManager.setGradualCooling(true)
    
    // Show humidity warning
    showEnvironmentalWarning("High humidity detected - monitoring for condensation")
  }
  
  private def handleWeatherConditions(weather: WeatherCondition): Unit = {
    weather match {
      case storm: StormCondition if storm.severity >= StormSeverity.Tropical =>
        prepareForStorm(storm)
      case rain: RainCondition if rain.intensity >= RainIntensity.Heavy =>
        prepareForHeavyRain(rain)
      case heat: HeatCondition if heat.temperature > 35.0 =>
        prepareForExtremeHeat(heat)
      case _ =>
        maintainNormalWeatherMode()
    }
  }
}

// Storm preparation and recovery
class StormResilienceManager {
  
  def prepareForStorm(storm: StormCondition): Unit = {
    // Show storm preparation dialog
    val dialog = new StormPreparationDialog(storm)
    dialog.showAndWait()
    
    // Automatically execute preparation steps
    executeStormPreparation()
  }
  
  private def executeStormPreparation(): Unit = {
    // Save all current work
    dataProtectionService.saveAllUnsavedData()
    
    // Create storm backup
    backupManager.createEmergencyBackup()
    
    // Sync critical data to cloud if internet available
    if (internetMonitor.isConnected) {
      cloudSyncService.prioritySync()
    }
    
    // Enable storm mode
    enableStormMode()
    
    // Schedule automatic shutdown if extended outage expected
    schedulePreemptiveShutdown()
  }
  
  private def enableStormMode(): Unit = {
    // Ultra-aggressive power saving
    enableUltraLowPowerMode()
    
    // Disable all non-essential services
    serviceManager.disableNonEssentialServices()
    
    // Enable rapid auto-save
    autoSaveManager.setInterval(5000) // Every 5 seconds
    
    // Show storm mode indicator
    statusBar.showStormModeIndicator()
  }
  
  def recoverFromStorm(): Unit = {
    // Check hardware status
    val hardwareStatus = hardwareMonitor.performPostStormCheck()
    
    if (hardwareStatus.hasIssues) {
      showHardwareIssuesDialog(hardwareStatus.issues)
    }
    
    // Check data integrity
    val dataStatus = dataIntegrityChecker.checkPostStormIntegrity()
    
    if (dataStatus.hasCorruption) {
      initiateDataRecovery(dataStatus.corruptedFiles)
    }
    
    // Resume normal operation
    disableStormMode()
  }
}

// Maintenance scheduling for tropical conditions  
class TropicalMaintenanceScheduler {
  
  def scheduleSeasonalMaintenance(): Unit = {
    // More frequent maintenance due to harsh environment
    
    // Monthly cleaning reminders
    scheduleRecurringTask("Hardware Cleaning", 30.days) {
      showMaintenanceReminder(
        "Clean dust from vents and fans",
        "High humidity and dust require monthly cleaning"
      )
    }
    
    // Quarterly thermal paste check (due to high temperatures)
    scheduleRecurringTask("Thermal Maintenance", 90.days) {
      showMaintenanceReminder(
        "Check thermal paste and cooling system",
        "High temperatures stress cooling systems"
      )
    }
    
    // Hurricane season preparation (June-November)
    scheduleSeasonalTask("Hurricane Preparation", HurricaneSeason) {
      showSeasonalPreparationDialog(DisasterType.Hurricane)
    }
    
    // Rainy season maintenance (May-October)
    scheduleSeasonalTask("Humidity Protection", RainySeason) {
      showSeasonalPreparationDialog(DisasterType.HighHumidity)
    }
  }
  
  private def showMaintenanceReminder(task: String, reason: String): Unit = {
    val dialog = new MaintenanceReminderDialog(task, reason)
    
    dialog.addAction("Mark Complete") { () =>
      logMaintenanceCompleted(task)
    }
    
    dialog.addAction("Snooze (1 week)") { () =>
      scheduleMaintenanceReminder(task, 7.days)
    }
    
    dialog.addAction("Skip This Time") { () =>
      logMaintenanceSkipped(task)
    }
    
    dialog.show()
  }
}
```

### Pattern 4: Dust and Contamination Protection

**Problem**: Tropical environments have high levels of dust, pollen, and airborne particles that can clog computer vents and contaminate optical components.

**Solution**: Proactive monitoring and user guidance for environmental protection.

**Implementation**:
```scala
// Dust and contamination monitoring
class ContaminationMonitor {
  
  private val airflowSensor = new AirflowSensor()
  private val fanSpeedMonitor = new FanSpeedMonitor()
  
  def monitorContamination(): Unit = {
    // Monitor fan performance degradation (indicates dust buildup)
    fanSpeedMonitor.onPerformanceDegradation { degradation =>
      if (degradation > 20) { // 20% performance loss
        recommendCleaning("Fan performance degraded - cleaning recommended")
      }
      if (degradation > 40) { // 40% performance loss
        urgentCleaningRequired("Critical: Fan performance severely degraded")
      }
    }
    
    // Monitor temperature trends (dust causes overheating)
    temperatureMonitor.onTemperatureTrend { trend =>
      if (trend.isIncreasing && trend.rate > 0.5) { // 0.5°C per day increase
        suspectDustBuildup()
      }
    }
  }
  
  private def recommendCleaning(reason: String): Unit = {
    val cleaningGuide = new CleaningGuideDialog(reason)
    
    cleaningGuide.addInstructions(
      "1. Shut down computer completely",
      "2. Unplug power cable", 
      "3. Use compressed air to blow dust from vents",
      "4. Clean keyboard and screen with appropriate cleaners",
      "5. Check that all vents are clear of obstruction"
    )
    
    cleaningGuide.addWarnings(
      "WARNING: Do not use water near electrical components",
      "WARNING: Ensure computer is completely dry before powering on",
      "NOTE: In high humidity, allow extra drying time"
    )
    
    cleaningGuide.show()
  }
}
```

---

## 🔧 Hardware Compatibility and Legacy Support Patterns

### Pattern 5: Mixed Hardware Environment Support

**Problem**: Caribbean dental practices often use a mix of older and newer hardware, requiring software that works across a wide range of specifications.

**Solution**: Adaptive performance scaling and graceful degradation for older hardware.

**Hardware Variability Challenges**:
- **Mixed OS versions** - Windows 7 through Windows 11
- **Varied RAM** - 2GB to 32GB
- **Different CPUs** - Single-core Pentium to multi-core i7
- **Graphics capabilities** - Integrated to discrete GPU
- **Storage types** - HDD to NVMe SSD

**Implementation**:
```scala
// Hardware capability detection and adaptation
class HardwareAdaptationManager {
  
  private val systemInfo = new SystemInformationService()
  
  case class HardwareProfile(
    cpuCores: Int,
    cpuSpeed: Double, // GHz
    totalRAM: Long,   // MB
    availableRAM: Long, // MB
    storageType: StorageType,
    graphicsType: GraphicsType,
    networkCapability: NetworkCapability
  )
  
  def detectHardwareCapabilities(): HardwareProfile = {
    HardwareProfile(
      cpuCores = Runtime.getRuntime.availableProcessors(),
      cpuSpeed = systemInfo.getCPUSpeed(),
      totalRAM = systemInfo.getTotalMemory() / (1024 * 1024),
      availableRAM = systemInfo.getAvailableMemory() / (1024 * 1024),
      storageType = detectStorageType(),
      graphicsType = detectGraphicsCapabilities(),
      networkCapability = detectNetworkCapabilities()
    )
  }
  
  def adaptApplicationToHardware(profile: HardwareProfile): AdaptationSettings = {
    val settings = AdaptationSettings()
    
    // CPU adaptation
    settings.threadPoolSize = profile.cpuCores match {
      case cores if cores >= 8 => cores - 2 // Leave 2 cores for OS
      case cores if cores >= 4 => cores - 1 // Leave 1 core for OS  
      case cores if cores >= 2 => cores     // Use all available
      case 1 => 1                            // Single core - conservative
    }
    
    // Memory adaptation
    settings.cacheSize = profile.availableRAM match {
      case ram if ram >= 8192 => 512  // 512MB cache for 8GB+ systems
      case ram if ram >= 4096 => 256  // 256MB cache for 4GB+ systems
      case ram if ram >= 2048 => 128  // 128MB cache for 2GB+ systems  
      case ram => math.max(32, ram / 16) // Conservative for low memory
    }
    
    // Graphics adaptation
    settings.graphicsQuality = profile.graphicsType match {
      case GraphicsType.Discrete => GraphicsQuality.High
      case GraphicsType.Integrated => GraphicsQuality.Medium
      case GraphicsType.Basic => GraphicsQuality.Low
    }
    
    // Storage adaptation
    settings.enableDiskCache = profile.storageType match {
      case StorageType.SSD | StorageType.NVMe => true
      case StorageType.HDD => false // Avoid random I/O on spinning disks
    }
    
    settings
  }
  
  def applyAdaptationSettings(settings: AdaptationSettings): Unit = {
    // Apply thread pool configuration
    threadPoolManager.setPoolSize(settings.threadPoolSize)
    
    // Apply memory settings
    cacheManager.setCacheSize(settings.cacheSize)
    
    // Apply graphics settings
    graphicsRenderer.setQualityLevel(settings.graphicsQuality)
    
    // Apply storage settings
    if (settings.enableDiskCache) {
      diskCacheManager.enableCache()
    } else {
      diskCacheManager.disableCache()
    }
    
    // Show adaptation summary to user
    showAdaptationSummary(settings)
  }
}

// Legacy system compatibility
class LegacySystemSupport {
  
  def enableLegacyCompatibility(): Unit = {
    val osVersion = systemInfo.getOSVersion()
    
    osVersion match {
      case version if version.startsWith("Windows 7") =>
        enableWindows7Compatibility()
      case version if version.startsWith("Windows 8") =>
        enableWindows8Compatibility()  
      case version if version.startsWith("Windows 10") =>
        enableWindows10Compatibility()
      case version if version.startsWith("Windows 11") =>
        enableWindows11Compatibility()
      case _ =>
        enableGenericCompatibility()
    }
  }
  
  private def enableWindows7Compatibility(): Unit = {
    // Disable modern Windows 10+ features
    modernUIManager.disableModernControls()
    
    // Use legacy file dialogs
    fileDialogManager.useLegacyDialogs()
    
    // Disable hardware acceleration (often problematic on Windows 7)
    graphicsRenderer.disableHardwareAcceleration()
    
    // Use reduced animation set
    animationManager.setLegacyAnimations(true)
    
    showCompatibilityNotice("Running in Windows 7 compatibility mode")
  }
  
  private def enableGenericCompatibility(): Unit = {
    // Ultra-conservative settings for unknown systems
    graphicsRenderer.setBasicRenderer()
    animationManager.disableAllAnimations()
    modernUIManager.disableModernControls()
    
    showCompatibilityNotice("Running in maximum compatibility mode")
  }
}

// Performance monitoring and automatic adjustment
class PerformanceMonitor {
  
  private val performanceMetrics = new PerformanceMetrics()
  
  def startPerformanceMonitoring(): Unit = {
    // Monitor UI responsiveness
    uiResponsivenessMonitor.onSlowResponse { responseTime =>
      if (responseTime > 200) { // 200ms threshold
        reduceUIComplexity()
      }
      if (responseTime > 500) { // 500ms critical threshold  
        enableEmergencyPerformanceMode()
      }
    }
    
    // Monitor memory pressure
    memoryMonitor.onHighPressure { pressure =>
      if (pressure > 80) {
        enableMemoryConservationMode()
      }
      if (pressure > 95) {
        enableEmergencyMemoryMode()
      }
    }
    
    // Monitor disk performance
    diskPerformanceMonitor.onSlowDisk { iopsRate =>
      if (iopsRate < 50) { // Very slow disk
        enableSlowDiskMode()
      }
    }
  }
  
  private def enableEmergencyPerformanceMode(): Unit = {
    // Disable all animations
    animationManager.disableAllAnimations()
    
    // Reduce graphics quality to minimum
    graphicsRenderer.setMinimumQuality()
    
    // Increase cache aggressiveness
    cacheManager.setAggressiveCaching(true)
    
    // Reduce background task frequency
    backgroundTaskManager.setReducedFrequency()
    
    showPerformanceNotice("Performance mode enabled due to slow system response")
  }
}
```

### Pattern 6: IT Support Limitations Adaptation

**Problem**: Caribbean dental practices typically have no on-site IT support and limited access to technical expertise.

**Solution**: Self-diagnostic capabilities, automated problem resolution, and clear user guidance for common issues.

**Implementation**:
```scala
// Self-diagnostic system
class SelfDiagnosticsManager {
  
  def runComprehensiveDiagnostics(): DiagnosticReport = {
    val report = DiagnosticReport()
    
    // Hardware diagnostics
    report.hardwareStatus = runHardwareDiagnostics()
    
    // Software diagnostics  
    report.softwareStatus = runSoftwareDiagnostics()
    
    // Data integrity diagnostics
    report.dataStatus = runDataIntegrityCheck()
    
    // Network connectivity diagnostics
    report.networkStatus = runNetworkDiagnostics()
    
    // Performance diagnostics
    report.performanceStatus = runPerformanceDiagnostics()
    
    report
  }
  
  private def runHardwareDiagnostics(): HardwareStatus = {
    val status = HardwareStatus()
    
    // CPU test
    status.cpuStatus = testCPUPerformance()
    
    // Memory test  
    status.memoryStatus = testMemoryIntegrity()
    
    // Storage test
    status.storageStatus = testStorageHealth()
    
    // Temperature monitoring
    status.temperatureStatus = checkTemperatures()
    
    status
  }
  
  private def testCPUPerformance(): ComponentStatus = {
    val startTime = System.currentTimeMillis()
    
    // Run CPU-intensive calculation
    val result = performCPUBenchmark()
    
    val duration = System.currentTimeMillis() - startTime
    
    duration match {
      case d if d < 1000 => ComponentStatus.Good
      case d if d < 3000 => ComponentStatus.Warning("CPU performance below expected")
      case _ => ComponentStatus.Critical("CPU performance severely degraded")
    }
  }
  
  def generateUserFriendlyReport(report: DiagnosticReport): String = {
    val sb = new StringBuilder()
    
    sb.append("=== System Health Check ===\n\n")
    
    // Overall status
    val overallStatus = determineOverallStatus(report)
    sb.append(s"Overall Status: ${overallStatus.description}\n\n")
    
    // Detailed findings
    if (report.hasIssues) {
      sb.append("Issues Found:\n")
      report.issues.foreach { issue =>
        sb.append(s"• ${issue.description}\n")
        sb.append(s"  Solution: ${issue.recommendedAction}\n\n")
      }
    } else {
      sb.append("✓ No issues detected - system is running normally\n\n")
    }
    
    // Maintenance recommendations
    sb.append("Maintenance Recommendations:\n")
    getMaintenanceRecommendations(report).foreach { rec =>
      sb.append(s"• $rec\n")
    }
    
    sb.toString()
  }
}

// Automated problem resolution
class AutoProblemResolver {
  
  def attemptAutomaticResolution(issue: SystemIssue): ResolutionResult = {
    issue match {
      case _: LowDiskSpaceIssue =>
        resolveStorageIssue()
      case _: HighMemoryUsageIssue =>
        resolveMemoryIssue()
      case _: SlowPerformanceIssue =>
        resolvePerformanceIssue()
      case _: NetworkConnectivityIssue =>
        resolveNetworkIssue()
      case _: DataCorruptionIssue =>
        resolveDataIssue()
      case _ =>
        ResolutionResult.CannotResolve("Manual intervention required")
    }
  }
  
  private def resolveStorageIssue(): ResolutionResult = {
    try {
      // Clean temporary files
      val tempCleaned = cleanTemporaryFiles()
      
      // Clean application cache
      val cacheCleaned = cleanApplicationCache()
      
      // Clean old backups (keep only recent 5)
      val backupsCleaned = cleanOldBackups()
      
      val totalFreed = tempCleaned + cacheCleaned + backupsCleaned
      
      if (totalFreed > 100 * 1024 * 1024) { // 100MB
        ResolutionResult.Success(s"Freed ${formatBytes(totalFreed)} of disk space")
      } else {
        ResolutionResult.PartialSuccess("Some disk space freed, but more cleanup needed")
      }
      
    } catch {
      case e: Exception =>
        ResolutionResult.Failed(s"Could not clean disk space: ${e.getMessage}")
    }
  }
  
  private def resolveMemoryIssue(): ResolutionResult = {
    try {
      // Close non-essential windows
      val windowsClosed = windowManager.closeNonEssentialWindows()
      
      // Clear unnecessary caches
      cacheManager.clearNonEssentialCaches()
      
      // Force garbage collection
      System.gc()
      
      // Wait and check if memory pressure reduced
      Thread.sleep(2000)
      val currentMemoryUsage = memoryMonitor.getCurrentUsage()
      
      if (currentMemoryUsage < 80) {
        ResolutionResult.Success("Memory pressure reduced")
      } else {
        ResolutionResult.PartialSuccess("Some memory freed, consider restarting application")
      }
      
    } catch {
      case e: Exception =>
        ResolutionResult.Failed(s"Could not resolve memory issue: ${e.getMessage}")
    }
  }
}

// User-friendly troubleshooting guides
class TroubleshootingGuides {
  
  def getGuideForIssue(issue: SystemIssue): TroubleshootingGuide = {
    issue match {
      case _: PowerIssue => createPowerTroubleshootingGuide()
      case _: NetworkIssue => createNetworkTroubleshootingGuide() 
      case _: PerformanceIssue => createPerformanceTroubleshootingGuide()
      case _: DisplayIssue => createDisplayTroubleshootingGuide()
      case _ => createGeneralTroubleshootingGuide()
    }
  }
  
  private def createPowerTroubleshootingGuide(): TroubleshootingGuide = {
    TroubleshootingGuide(
      title = "Power and Electrical Issues",
      steps = List(
        TroubleshootingStep(
          description = "Check power connections",
          actions = List(
            "Verify power cable is securely connected",
            "Check that UPS (if present) is on and charged",
            "Look for loose connections at power strip or wall outlet"
          )
        ),
        TroubleshootingStep(
          description = "Check for voltage issues", 
          actions = List(
            "If lights are dim or flickering, voltage may be low",
            "If you hear electrical humming, voltage may be high",
            "Consider using a voltage stabilizer for protection"
          )
        ),
        TroubleshootingStep(
          description = "Protect from power surges",
          actions = List(
            "Use a good quality surge protector",
            "Unplug during thunderstorms if possible",
            "Consider a UPS with surge protection"
          )
        )
      ),
      emergencyActions = List(
        "If you smell burning, immediately unplug the computer",
        "If screen flickers rapidly, power off immediately",
        "Save your work frequently during storms"
      )
    )
  }
}
```

---

## 🔗 Related Patterns

- **Offline-First-Desktop-Architecture.md** - Data persistence during infrastructure failures
- **Caribbean-Hardware-Integration-Patterns.md** - Specific hardware adaptations for Caribbean environments
- **Desktop-Application-Performance-Patterns.md** - Performance optimization for limited hardware
- **Caribbean-Desktop-Deployment-Strategies.md** - Installation and update strategies for limited connectivity

---

## 📊 Resilience Metrics and Monitoring

### Key Resilience Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Power Failure Recovery Time** | < 30 seconds | Time from power restoration to full operation |
| **Data Loss Rate** | 0% | Percentage of transactions lost during failures |
| **Hardware Failure Prediction** | 2 weeks advance | Time between prediction and actual failure |
| **Environmental Adaptation** | < 5 minutes | Time to adapt to environmental changes |
| **Self-Diagnostic Accuracy** | > 95% | Percentage of correctly identified issues |

### Environmental Monitoring Dashboard

```scala
// Real-time resilience dashboard
class ResilienceDashboard extends VBox {
  
  private val powerStatusIndicator = new PowerStatusIndicator()
  private val temperatureGauge = new TemperatureGauge()
  private val hardwareHealthPanel = new HardwareHealthPanel()
  
  def initialize(): Unit = {
    getChildren.addAll(
      createHeader("System Resilience Status"),
      powerStatusIndicator,
      temperatureGauge, 
      hardwareHealthPanel,
      createActionButtons()
    )
    
    startRealTimeUpdates()
  }
  
  private def startRealTimeUpdates(): Unit = {
    val timeline = new Timeline(
      new KeyFrame(Duration.seconds(5), _ => updateDashboard())
    )
    timeline.setCycleCount(Timeline.INDEFINITE)
    timeline.play()
  }
}
```

---

**Last Updated**: January 17, 2026  
**Maintained By**: Infrastructure Architect + Caribbean Operations Specialist  
**Review Frequency**: Quarterly (before hurricane season) and after major weather events  
**Version**: 1.0.0

---

**Key Insight**: Caribbean resilience requires **proactive adaptation rather than reactive recovery**. Monitor environmental conditions, predict failures before they occur, and automatically adjust system behavior to prevent problems rather than simply recovering from them. The goal is **graceful degradation under stress, not just survival**.