# Trait-Based I2C Service Layer Architecture

**Project:** Hubris Operatin```rust
/// Bus recovery behavior - platform agnostic interface
pub trait BusRecovery {
    type Error: Into<ResponseCode>;
    type ControllerId: Copy + Clone + Debug + Eq + Hash;
    
    /// Attempt to recover a stuck I2C bus
    fn recover_bus(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error>;
    
    /// Check if bus appears to be stuck
    fn is_bus_stuck(&self, controller_id: Self::ControllerId) -> Result<bool, Self::Error>;
    
    /// Get recovery statistics
    fn recovery_stats(&self, controller_id: Self::ControllerId) -> BusRecoveryStats;
}

/// Power management behavior - platform agnostic interface  
pub trait PowerManagement {
    type Error: Into<ResponseCode>;
    type ControllerId: Copy + Clone + Debug + Eq + Hash;
    type PowerMode: Copy + Clone + Debug;
    
    /// Transition to specified power mode
    fn set_power_mode(&mut self, mode: Self::PowerMode) -> Result<(), Self::Error>;
    
    /// Get current power mode
    fn power_mode(&self) -> Self::PowerMode;
    
    /// Configure controller for low power mode
    fn prepare_controller_for_sleep(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error>;
    
    /// Restore controller after wake from low power
    fn restore_controller_from_sleep(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error>;
    
    /// Configure wakeup sources
    fn configure_wakeup(&mut self, controller_id: Self::ControllerId, enable: bool) -> Result<(), Self::Error>;
}

/// Multiplexer management behavior
pub trait MultiplexerManagement {
    type Error: Into<ResponseCode>;
    type ControllerId: Copy + Clone + Debug + Eq + Hash;
    type MuxAddress: Copy + Clone + Debug;
    type Segment: Copy + Clone + Debug;
    
    /// Select multiplexer segment
    fn select_mux_segment(&mut self, 
                         controller_id: Self::ControllerId,
                         mux: Self::MuxAddress, 
                         segment: Self::Segment) -> Result<(), Self::Error>;
    
    /// Get current mux state
    fn current_mux_state(&self, controller_id: Self::ControllerId) -> Option<(Self::MuxAddress, Self::Segment)>;
    
    /// Reset all multiplexers on controller
    fn reset_muxes(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error>;
}

/// Hardware feature detection and capability management
pub trait HardwareCapabilities {
    type ControllerId: Copy + Clone + Debug + Eq + Hash;
    type Features: Clone + Debug;
    
    /// Detect hardware capabilities at runtime
    fn detect_capabilities(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Get capabilities for specific controller
    fn controller_capabilities(&self, controller_id: Self::ControllerId) -> Option<&Self::Features>;
    
    /// Check if feature is supported
    fn supports_feature(&self, controller_id: Self::ControllerId, feature: &str) -> bool;
}

/// Error handling and diagnostics behavior
pub trait ErrorHandling {
    type Error: Into<ResponseCode>;
    type ControllerId: Copy + Clone + Debug + Eq + Hash;
    type ErrorStats: Clone + Debug;
    
    /// Handle hardware error interrupt
    fn handle_error(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error>;
    
    /// Get error statistics
    fn error_stats(&self, controller_id: Self::ControllerId) -> Self::ErrorStats;
    
    /// Clear error counters
    fn clear_error_stats(&mut self, controller_id: Self::ControllerId);
    
    /// Get last error details
    fn last_error(&self, controller_id: Self::ControllerId) -> Option<Self::Error>;
}omponent:** Composable I2C Service Layer Traits  
**Version:** 1.0  
**Date:** September 8, 2025  

## Overview

This document proposes refactoring the STM32-specific `Stm32I2cServer` into a composable trait-based architecture, allowing for platform independence while maintaining high-level behaviors like bus recovery and power management.

## Current Monolithic Architecture Problem

The current `Stm32I2cServer` tightly couples platform-specific details with high-level logic:

```rust
// Current monolithic approach
struct Stm32I2cServer {
    // Platform-specific components mixed with generic logic
    sys: Sys,                           // STM32-specific
    drivers: [Stm32I2cDriver; 4],      // STM32-specific
    mux_states: MuxStateTracker,        // Generic
    power_mode: PowerMode,              // Generic concept, STM32 implementation
    bus_recovery: BusRecoveryState,     // Generic concept, STM32 implementation
}
```

## Proposed Trait-Based Architecture

### **1. Core Service Layer Traits**

```rust
/// Generic I2C service layer - platform independent
pub trait I2cServiceLayer {
    type Error: Into<ResponseCode>;
    type ControllerId: Copy + Clone + Debug + Eq + Hash;  // Just the ID, not the controller
    type DriverInterface: I2cDriverInterface;
    
    /// Handle incoming IPC request
    fn handle_request(&mut self, request: I2cRequest) -> Result<I2cResponse, Self::Error>;
    
    /// Initialize the service layer
    fn initialize(&mut self) -> Result<(), Self::Error>;
    
    /// Get available controller IDs
    fn available_controller_ids(&self) -> &[Self::ControllerId];
    
    /// Shutdown gracefully
    fn shutdown(&mut self) -> Result<(), Self::Error>;
}

/// Low-level driver interface abstraction
pub trait I2cDriverInterface {
    type Error: Into<ResponseCode>;
    type ControllerId: Copy + Clone + Debug + Eq + Hash;  // Lightweight controller identifier
    
    /// Perform I2C write-read transaction
    fn write_read(&mut self, 
                  controller_id: Self::ControllerId,
                  addr: u8, 
                  write_data: &[u8], 
                  read_data: &mut [u8]) -> Result<usize, Self::Error>;
    
    /// Check if controller is ready
    fn is_ready(&self, controller_id: Self::ControllerId) -> bool;
    
    /// Reset controller to known state
    fn reset(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error>;
}
```

### **2. Composable Behavior Traits**

```rust
/// Bus recovery behavior - platform agnostic interface
pub trait BusRecovery {
    type Error: Into<ResponseCode>;
    type ControllerId: Copy + Clone + Debug + Eq + Hash;
    
    /// Attempt to recover a stuck I2C bus
    fn recover_bus(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error>;
    
    /// Check if bus appears to be stuck
    fn is_bus_stuck(&self, controller_id: Self::ControllerId) -> Result<bool, Self::Error>;
    
    /// Get recovery statistics
    fn recovery_stats(&self, controller_id: Self::ControllerId) -> BusRecoveryStats;
}

/// Power management behavior - platform agnostic interface  
pub trait PowerManagement {
    type Error: Into<ResponseCode>;
    type ControllerId: Copy + Clone + Debug + Eq + Hash;
    type PowerMode: Copy + Clone + Debug;
    
    /// Transition to specified power mode
    fn set_power_mode(&mut self, mode: Self::PowerMode) -> Result<(), Self::Error>;
    
    /// Get current power mode
    fn power_mode(&self) -> Self::PowerMode;
    
    /// Configure controller for low power mode
    fn prepare_controller_for_sleep(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error>;
    
    /// Restore controller after wake from low power
    fn restore_controller_from_sleep(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error>;
    
    /// Configure wakeup sources
    fn configure_wakeup(&mut self, controller_id: Self::ControllerId, enable: bool) -> Result<(), Self::Error>;
}

/// Multiplexer management behavior
pub trait MultiplexerManagement {
    type Error: Into<ResponseCode>;
    type ControllerId: Copy + Clone + Debug + Eq + Hash;
    type MuxAddress: Copy + Clone + Debug;
    type Segment: Copy + Clone + Debug;
    
    /// Select multiplexer segment
    fn select_mux_segment(&mut self, 
                         controller_id: Self::ControllerId,
                         mux: Self::MuxAddress, 
                         segment: Self::Segment) -> Result<(), Self::Error>;
    
    /// Get current mux state
    fn current_mux_state(&self, controller_id: Self::ControllerId) -> Option<(Self::MuxAddress, Self::Segment)>;
    
    /// Reset all multiplexers on controller
    fn reset_muxes(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error>;
}

/// Hardware feature detection and capability management
pub trait HardwareCapabilities {
    type ControllerId: Copy + Clone + Debug + Eq + Hash;
    type Features: Clone + Debug;
    
    /// Detect hardware capabilities at runtime
    fn detect_capabilities(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Get capabilities for specific controller
    fn controller_capabilities(&self, controller_id: Self::ControllerId) -> Option<&Self::Features>;
    
    /// Check if feature is supported
    fn supports_feature(&self, controller_id: Self::ControllerId, feature: &str) -> bool;
}

/// Error handling and diagnostics behavior
pub trait ErrorHandling {
    type Error: Into<ResponseCode>;
    type ControllerId: Copy + Clone + Debug + Eq + Hash;
    type ErrorStats: Clone + Debug;
    
    /// Handle hardware error interrupt
    fn handle_error(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error>;
    
    /// Get error statistics
    fn error_stats(&self, controller_id: Self::ControllerId) -> Self::ErrorStats;
    
    /// Clear error counters
    fn clear_error_stats(&mut self, controller_id: Self::ControllerId);
    
    /// Get last error details
    fn last_error(&self, controller_id: Self::ControllerId) -> Option<Self::Error>;
}
```

### **3. Generic Service Layer Implementation**

```rust
/// Generic I2C service that composes behaviors through traits
pub struct GenericI2cService<D, B, P, M, H, E> 
where
    D: I2cDriverInterface,
    B: BusRecovery<ControllerId = D::ControllerId>,
    P: PowerManagement<ControllerId = D::ControllerId>,
    M: MultiplexerManagement<ControllerId = D::ControllerId>,
    H: HardwareCapabilities<ControllerId = D::ControllerId>,
    E: ErrorHandling<ControllerId = D::ControllerId>,
{
    driver: D,
    bus_recovery: B,
    power_management: P,
    mux_management: M,
    hardware_caps: H,
    error_handling: E,
    
    // Generic state - now uses controller IDs as keys
    active_transactions: HashMap<D::ControllerId, TransactionState>,
    ipc_buffer: [u8; 1024],
}

impl<D, B, P, M, H, E> I2cServiceLayer for GenericI2cService<D, B, P, M, H, E>
where
    D: I2cDriverInterface,
    B: BusRecovery<ControllerId = D::ControllerId>,
    P: PowerManagement<ControllerId = D::ControllerId>,
    M: MultiplexerManagement<ControllerId = D::ControllerId>,
    H: HardwareCapabilities<ControllerId = D::ControllerId>,
    E: ErrorHandling<ControllerId = D::ControllerId>,
{
    type Error = ServiceError;
    type ControllerId = D::ControllerId;
    type DriverInterface = D;
    
    fn handle_request(&mut self, request: I2cRequest) -> Result<I2cResponse, Self::Error> {
        // Generic request handling logic
        match request.operation {
            I2cOperation::WriteRead { controller_id, addr, write_data, read_len, mux_info } => {
                // 1. Configure multiplexer if needed
                if let Some((mux, segment)) = mux_info {
                    self.mux_management.select_mux_segment(controller_id, mux, segment)?;
                }
                
                // 2. Check if bus recovery is needed
                if self.bus_recovery.is_bus_stuck(controller_id)? {
                    self.bus_recovery.recover_bus(controller_id)?;
                }
                
                // 3. Perform transaction
                let mut read_buffer = vec![0u8; read_len];
                let bytes_read = self.driver.write_read(
                    controller_id, 
                    addr, 
                    &write_data, 
                    &mut read_buffer
                )?;
                
                Ok(I2cResponse::Data(read_buffer[..bytes_read].to_vec()))
            }
            
            I2cOperation::SetPowerMode { mode } => {
                // Generic power mode handling
                let power_mode = self.convert_power_mode(mode)?;
                self.power_management.set_power_mode(power_mode)?;
                Ok(I2cResponse::Success)
            }
            
            I2cOperation::RecoverBus { controller_id } => {
                // Explicit bus recovery request
                self.bus_recovery.recover_bus(controller_id)?;
                Ok(I2cResponse::Success)
            }
            
            I2cOperation::GetCapabilities { controller_id } => {
                // Hardware capability query
                let caps = self.hardware_caps.controller_capabilities(controller_id)
                    .ok_or(ServiceError::UnsupportedController)?;
                Ok(I2cResponse::Capabilities(caps.clone()))
            }
        }
    }
    
    fn initialize(&mut self) -> Result<(), Self::Error> {
        // Generic initialization sequence
        self.hardware_caps.detect_capabilities()?;
        
        for &controller_id in self.available_controller_ids() {
            self.driver.reset(controller_id)?;
            self.mux_management.reset_muxes(controller_id)?;
            self.error_handling.clear_error_stats(controller_id);
        }
        
        Ok(())
    }
    
    fn available_controller_ids(&self) -> &[Self::ControllerId] {
        // Delegate to hardware capabilities
        &[] // Implementation would return discovered controller IDs
    }
}
```

### **4. STM32-Specific Trait Implementations**

```rust
/// STM32-specific bus recovery implementation
pub struct Stm32BusRecovery {
    sys: Sys,
    pin_configs: HashMap<ControllerId, Stm32PinConfig>,
    recovery_stats: HashMap<ControllerId, BusRecoveryStats>,
    // The actual controller instances with register mappings
    controllers: HashMap<ControllerId, Stm32I2cController>,
}

impl BusRecovery for Stm32BusRecovery {
    type Error = Stm32I2cError;
    type ControllerId = ControllerId;  // Simple enum: I2C1, I2C2, I2C3
    
    fn recover_bus(&mut self, controller_id: Self::ControllerId) -> Result<(), Self::Error> {
        let pin_config = self.pin_configs.get(&controller_id)
            .ok_or(Stm32I2cError::UnsupportedController)?;
        
        // Access the actual controller for register operations
        let controller = self.controllers.get_mut(&controller_id)
            .ok_or(Stm32I2cError::UnsupportedController)?;
        
        // 1. Disable I2C peripheral before GPIO manipulation
        controller.disable()?;
        
        // 2. STM32-specific GPIO bit-banging recovery
        self.sys.gpio_configure_output(
            pin_config.scl_pin,
            OutputType::OpenDrain,
            Speed::High,
            Pull::Up,
        )?;
        
        // Generate 9 clock pulses with STM32 GPIO
        for _ in 0..9 {
            self.sys.gpio_reset(pin_config.scl_pin);
            self.delay_microseconds(5);
            self.sys.gpio_set(pin_config.scl_pin);
            self.delay_microseconds(5);
            
            if self.sys.gpio_read(pin_config.sda_pin) {
                break;
            }
        }
        
        // Generate STOP condition
        self.generate_stop_condition(pin_config)?;
        
        // Restore I2C alternate function
        self.restore_i2c_pins(pin_config)?;
        
        // Update statistics
        self.recovery_stats.entry(controller)
            .or_default()
            .recovery_count += 1;
        
        Ok(())
    }
    
    fn is_bus_stuck(&self, controller: Self::Controller) -> Result<bool, Self::Error> {
        // STM32-specific bus stuck detection logic
        let pin_config = self.pin_configs.get(&controller)
            .ok_or(Stm32I2cError::UnsupportedController)?;
        
        // Check if SDA is stuck low
        let sda_state = self.sys.gpio_read(pin_config.sda_pin);
        let scl_state = self.sys.gpio_read(pin_config.scl_pin);
        
        Ok(!sda_state || !scl_state)
    }
    
    fn recovery_stats(&self, controller: Self::Controller) -> BusRecoveryStats {
        self.recovery_stats.get(&controller)
            .cloned()
            .unwrap_or_default()
    }
}

/// STM32-specific power management implementation
pub struct Stm32PowerManagement {
    sys: Sys,
    current_mode: Stm32PowerMode,
    controller_states: HashMap<Controller, ControllerPowerState>,
}

impl PowerManagement for Stm32PowerManagement {
    type Error = Stm32I2cError;
    type Controller = Controller;
    type PowerMode = Stm32PowerMode;
    
    fn set_power_mode(&mut self, mode: Self::PowerMode) -> Result<(), Self::Error> {
        match mode {
            Stm32PowerMode::Run => {
                // Enable all I2C clocks
                for &controller in &[Controller::I2C1, Controller::I2C2, Controller::I2C3] {
                    self.enable_controller_clock(controller)?;
                }
            }
            Stm32PowerMode::Sleep => {
                // Configure wakeup sources but keep clocks enabled
                for &controller in &[Controller::I2C1, Controller::I2C2, Controller::I2C3] {
                    self.configure_wakeup(controller, true)?;
                }
            }
            Stm32PowerMode::Stop => {
                // Save state and disable clocks
                for &controller in &[Controller::I2C1, Controller::I2C2, Controller::I2C3] {
                    self.prepare_controller_for_sleep(controller)?;
                    self.disable_controller_clock(controller)?;
                }
            }
        }
        
        self.current_mode = mode;
        Ok(())
    }
    
    fn power_mode(&self) -> Self::PowerMode {
        self.current_mode
    }
    
    fn prepare_controller_for_sleep(&mut self, controller: Self::Controller) -> Result<(), Self::Error> {
        // Save STM32 I2C controller state
        let state = ControllerPowerState {
            timing_config: self.read_timing_registers(controller)?,
            control_config: self.read_control_registers(controller)?,
            enabled: self.is_controller_enabled(controller)?,
        };
        
        self.controller_states.insert(controller, state);
        Ok(())
    }
    
    fn restore_controller_from_sleep(&mut self, controller: Self::Controller) -> Result<(), Self::Error> {
        // Restore STM32 I2C controller state
        if let Some(state) = self.controller_states.get(&controller) {
            self.enable_controller_clock(controller)?;
            self.restore_timing_registers(controller, &state.timing_config)?;
            self.restore_control_registers(controller, &state.control_config)?;
        }
        Ok(())
    }
    
    fn configure_wakeup(&mut self, controller: Self::Controller, enable: bool) -> Result<(), Self::Error> {
        // Configure STM32 I2C as wakeup source
        let interrupt_vector = self.get_interrupt_vector(controller)?;
        if enable {
            self.sys.configure_wakeup_interrupt(interrupt_vector);
        } else {
            self.sys.disable_wakeup_interrupt(interrupt_vector);
        }
        Ok(())
    }
}

/// STM32-specific hardware capabilities
pub struct Stm32HardwareCapabilities {
    detected_features: HashMap<Controller, Stm32Features>,
    device_id: u32,
    revision_id: u32,
}

impl HardwareCapabilities for Stm32HardwareCapabilities {
    type Controller = Controller;
    type Features = Stm32Features;
    
    fn detect_capabilities(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Read STM32 device ID and revision
        self.device_id = read_device_id()?;
        self.revision_id = read_revision_id()?;
        
        // Determine capabilities based on STM32 variant
        let base_features = match self.device_id {
            0x413 => Stm32Features {  // STM32F407
                has_fifo: false,
                max_speed: I2cSpeed::Fast400k,
                smbus_support: true,
                fast_mode_plus: false,
                analog_filter: true,
                digital_filter_stages: 15,
            },
            0x450 => Stm32Features {  // STM32H7
                has_fifo: true,
                max_speed: I2cSpeed::FastPlus1M,
                smbus_support: true,
                fast_mode_plus: true,
                analog_filter: true,
                digital_filter_stages: 15,
            },
            _ => return Err("Unsupported STM32 variant".into()),
        };
        
        // Apply features to all available controllers
        for &controller in &[Controller::I2C1, Controller::I2C2, Controller::I2C3] {
            self.detected_features.insert(controller, base_features.clone());
        }
        
        Ok(())
    }
    
    fn controller_capabilities(&self, controller: Self::Controller) -> Option<&Self::Features> {
        self.detected_features.get(&controller)
    }
    
    fn supports_feature(&self, controller: Self::Controller, feature: &str) -> bool {
        if let Some(features) = self.detected_features.get(&controller) {
            match feature {
                "fifo" => features.has_fifo,
                "fast_mode_plus" => features.fast_mode_plus,
                "smbus" => features.smbus_support,
                "analog_filter" => features.analog_filter,
                _ => false,
            }
        } else {
            false
        }
    }
}
```

### **5. Service Composition and Instantiation**

```rust
/// STM32-specific service layer type alias
pub type Stm32I2cService = GenericI2cService<
    Stm32I2cDriver,           // Driver implementation
    Stm32BusRecovery,         // Bus recovery implementation
    Stm32PowerManagement,     // Power management implementation
    Stm32MultiplexerManagement, // Mux management implementation
    Stm32HardwareCapabilities, // Hardware capabilities implementation
    Stm32ErrorHandling,       // Error handling implementation
>;

/// Factory function to create configured STM32 I2C service
pub fn create_stm32_i2c_service(sys: Sys) -> Result<Stm32I2cService, ServiceError> {
    let driver = Stm32I2cDriver::new(sys.clone())?;
    let bus_recovery = Stm32BusRecovery::new(sys.clone())?;
    let power_management = Stm32PowerManagement::new(sys.clone())?;
    let mux_management = Stm32MultiplexerManagement::new()?;
    let hardware_caps = Stm32HardwareCapabilities::new(sys.clone())?;
    let error_handling = Stm32ErrorHandling::new(sys)?;
    
    let mut service = GenericI2cService {
        driver,
        bus_recovery,
        power_management,
        mux_management,
        hardware_caps,
        error_handling,
        active_transactions: HashMap::new(),
        ipc_buffer: [0u8; 1024],
    };
    
    service.initialize()?;
    Ok(service)
}

/// Main server task using trait-based service
#[export_name = "main"]
fn main() -> ! {
    let sys = SYS.get_task_id();
    let sys = Sys::from(sys);
    
    let mut service = create_stm32_i2c_service(sys)
        .expect("Failed to create I2C service");
    
    loop {
        userlib::recv_without_notification(&mut buffer, |op, msg| {
            match op {
                Op::WriteRead => {
                    let request = parse_i2c_request(msg)?;
                    let response = service.handle_request(request)?;
                    Ok(serialize_response(response))
                }
                _ => Err(ResponseCode::BadOp),
            }
        });
    }
}
```

## Benefits of Trait-Based Architecture

### **1. Platform Independence**
- Core service logic is platform-agnostic
- Easy to port to different microcontrollers
- Clear separation between generic and platform-specific code

### **2. Composability**
- Mix and match implementations for different features
- Easy to test individual behaviors in isolation
- Modular replacement of specific functionality

### **3. Testability**
```rust
// Easy to create mock implementations for testing
struct MockBusRecovery;
impl BusRecovery for MockBusRecovery {
    // Test implementation
}

// Test with different behavior combinations
#[test]
fn test_service_with_mock_recovery() {
    let service = GenericI2cService {
        driver: MockDriver::new(),
        bus_recovery: MockBusRecovery,
        // ... other mock implementations
    };
    // Test service behavior
}
```

### **4. Extensibility**
- Add new behaviors without modifying existing code
- Support new hardware features through trait extensions
- Gradual migration path from monolithic implementation

### **5. Type Safety**
- Compile-time verification of behavior composition
- No runtime overhead for abstraction
- Clear interface contracts through trait definitions

## Migration Strategy

### **Phase 1: Extract Traits**
1. Define core traits for existing behaviors
2. Keep current `Stm32I2cServer` implementation
3. Add trait implementations alongside existing code

### **Phase 2: Implement Generic Service**
1. Create `GenericI2cService` with trait composition
2. Implement STM32-specific trait instances
3. Parallel implementation for validation

### **Phase 3: Switch Implementation**
1. Replace `Stm32I2cServer` with trait-based service
2. Update build configuration and task definitions
3. Remove old monolithic implementation

### **Phase 4: Optimize and Extend**
1. Add new platform support through trait implementations
2. Optimize individual behaviors based on profiling
3. Add advanced features through trait extensions

This trait-based architecture provides a clean separation between platform-specific hardware details and generic I2C service behaviors, making the code more maintainable, testable, and portable while preserving the performance and reliability requirements of embedded systems.
