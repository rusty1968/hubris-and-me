# Hubris System Control Driver Development Guide 

This document explains the system control driver's role in Hubris and provides guidance for implementing clock and reset management in new microcontroller ports using standardized OpenProt traits.

## Architectural Role and Dependencies

### Dependency Hierarchy
```
┌─────────────────────────┐
│   Application Tasks     │
├─────────────────────────┤
│ Communication Drivers   │  ← SPI, I2C, UART servers
│   (spi-server, etc.)    │
├─────────────────────────┤
│    System Driver        │  ← drv-chipfamily-sys
│   (Clock, GPIO, Reset)  │
├─────────────────────────┤
│  Abstraction Layer      │  ← ClockControl, ResetControl traits
├─────────────────────────┤
│     Hardware PAC        │  ← Register-level access
│  (Peripheral Access)    │
└─────────────────────────┘
```

## What is the System Control Driver?

The **system control driver** is the foundational driver that provides essential chip-level services to all other drivers and application tasks. It serves as the "hardware abstraction coordinator" for the entire system, managing shared resources that multiple tasks need to access.

In Hubris, the system control driver is typically named `drv-{chipfamily}-sys` (e.g., `drv-stm32xx-sys`, `drv-ast1060-sys`) and runs as a high-priority server task that other drivers communicate with via IPC.

## Core Responsibilities

### 1. Clock Management

The system driver controls peripheral clock signals, which is fundamental since most microcontroller peripherals require active clock signals to operate. This functionality leverages standardized OpenProt traits implemented by external HAL crates.

**Key Operations:**
- **`enable_clock(clock_id)`**: Provides clock signal to a specific peripheral, making it operational
- **`disable_clock(clock_id)`**: Stops clock signal to a peripheral, putting it in a non-operational state (saves power but peripheral retains state)
- **`set_clock_frequency(clock_id, frequency_hz)`**: Configure peripheral clock frequency
- **`get_clock_frequency(clock_id)`**: Query current peripheral clock frequency
- **`configure_clock(clock_id, config)`**: Advanced clock configuration with custom parameters

### 2. Reset Control

Reset management ensures peripherals start in a known state and can be reliably reinitialized. This uses the OpenProt `ResetControl` trait implemented by HAL crates.

**Key Operations:**
- **`reset_assert(reset_id)`**: Places a peripheral in reset state for initialization
- **`reset_deassert(reset_id)`**: Releases a peripheral from reset, making it operational
- **`reset_pulse(reset_id, duration)`**: Performs a complete reset cycle with specified timing
- **`reset_is_asserted(reset_id)`**: Query current reset state

## OpenProt Standardized Traits

The system control driver leverages standardized traits from the **OpenProt project** that provide a common interface for clock and reset control across all chip families. This allows the Hubris system driver code to be virtually identical regardless of the target hardware.

### The OpenProt Traits

```rust
// From the OpenProt project
use system_control_traits::{ClockControl, ResetControl, Error, ErrorKind};

/// Common trait for clock control operations
pub trait ClockControl: ErrorType {
    type ClockId: Clone + PartialEq;
    type ClockConfig: PartialEq;

    fn enable(&mut self, clock_id: &Self::ClockId) -> Result<(), Self::Error>;
    fn disable(&mut self, clock_id: &Self::ClockId) -> Result<(), Self::Error>;
    fn set_frequency(&mut self, clock_id: &Self::ClockId, frequency_hz: u64) -> Result<(), Self::Error>;
    fn get_frequency(&self, clock_id: &Self::ClockId) -> Result<u64, Self::Error>;
    fn configure(&mut self, clock_id: &Self::ClockId, config: Self::ClockConfig) -> Result<(), Self::Error>;
    fn get_config(&self, clock_id: &Self::ClockId) -> Result<Self::ClockConfig, Self::Error>;
}

/// Common trait for reset control operations  
pub trait ResetControl: ErrorType {
    type ResetId: Clone + PartialEq;

    fn reset_assert(&mut self, reset_id: &Self::ResetId) -> Result<(), Self::Error>;
    fn reset_deassert(&mut self, reset_id: &Self::ResetId) -> Result<(), Self::Error>;
    fn reset_pulse(&mut self, reset_id: &Self::ResetId, duration: Duration) -> Result<(), Self::Error>;
    fn reset_is_asserted(&self, reset_id: &Self::ResetId) -> Result<bool, Self::Error>;
}
```

### Benefits of OpenProt Standardization

**Identical Driver Code:**
- System driver implementation is virtually the same across all chip families
- Only the mapping layer and HAL instantiation differ between chips
- Core clock/reset logic uses standard trait methods

**Vendor Ecosystem:**
- Chip vendors implement OpenProt traits in their HAL crates
- HAL crates can be shared across different RTOS/bare-metal projects  
- Leverages existing Rust embedded ecosystem

**Type Safety:**
- Compile-time guarantees about clock/reset operations
- Platform-specific identifiers prevent cross-chip confusion
- Rich error handling with standardized error kinds

## Implementation Architecture

### 1. Unified System Driver Implementation

With OpenProt traits, the system driver code becomes virtually identical across chip families:

```rust
#![no_std]
#![no_main]

use idol_runtime::{RequestError, NotificationHandler};
use userlib::*;
use system_control_traits::{ClockControl, ResetControl, Error, ErrorKind};
use sys_api::*;

// Platform-specific HAL imports (conditional compilation)
#[cfg(feature = "aspeed-ast2600")]
use aspeed_rust::SystemController;
#[cfg(feature = "aspeed-ast2600")]
use aspeed_mapping::AspeedMapping;

#[cfg(feature = "stm32f407")]
use stm32f4xx_hal::SystemController;
#[cfg(feature = "stm32f407")]  
use stm32_mapping::Stm32Mapping;

#[cfg(feature = "rp2040")]
use rp2040_hal::SystemController;
#[cfg(feature = "rp2040")]
use rp_mapping::RpMapping;

// Include generated server stub
include!(concat!(env!("OUT_DIR"), "/server_stub.rs"));

// The core server implementation is IDENTICAL for all chip families
struct SysServer {
    controller: SystemController,  // Different type per chip, same trait interface
    mapping: &'static dyn ClockResetMapping,
    initialized: bool,
}

impl SysServer {
    fn new() -> Self {
        // Only this instantiation differs between chips
        #[cfg(feature = "aspeed-ast2600")]
        let (controller, mapping) = (SystemController::new(), &AspeedMapping);
        
        #[cfg(feature = "stm32f407")]
        let (controller, mapping) = (SystemController::new(), &Stm32Mapping);
        
        #[cfg(feature = "rp2040")]
        let (controller, mapping) = (SystemController::new(), &RpMapping);
        
        Self {
            controller,
            mapping,
            initialized: false,
        }
    }
    
    // This init() method is IDENTICAL across all chip families
    fn init(&mut self) -> Result<(), SysError> {
        if self.initialized {
            return Ok(());
        }
        
        // All chips use the same initialization sequence
        self.configure_system_clocks()?;
        self.enable_essential_clocks()?;
        
        self.initialized = true;
        Ok(())
    }
    
    // IDENTICAL implementation - uses OpenProt trait methods
    fn configure_system_clocks(&mut self) -> Result<(), SysError> {
        // Get platform-specific configuration through mapping layer
        let main_clock = self.mapping.get_main_system_clock_id();
        let target_freq = self.mapping.get_optimal_system_frequency();
        let main_config = self.mapping.get_main_clock_config();
        
        // Use standard trait methods - same for all chips!
        self.controller.set_frequency(&main_clock, target_freq)
            .map_err(|e| SysError::from_trait_error(e))?;
            
        self.controller.configure(&main_clock, main_config)
            .map_err(|e| SysError::from_trait_error(e))?;
            
        Ok(())
    }
    
    // IDENTICAL clock enable implementation across all chips
    fn enable_clock_internal(&mut self, generic_clock_id: u32) -> Result<(), SysError> {
        // Map generic ID to platform-specific type
        let platform_clock_id = self.mapping.map_clock_id(generic_clock_id)
            .ok_or(SysError::ClockNotFound)?;
            
        // Use standard OpenProt trait method - same for all chips!
        self.controller.enable(&platform_clock_id)
            .map_err(|e| SysError::from_trait_error(e))
    }
    
    // IDENTICAL reset implementation across all chips
    fn reset_assert_internal(&mut self, generic_reset_id: u32) -> Result<(), SysError> {
        let platform_reset_id = self.mapping.map_reset_id(generic_reset_id)
            .ok_or(SysError::InvalidResetId)?;
            
        // Use standard OpenProt trait method - same for all chips!
        self.controller.reset_assert(&platform_reset_id)
            .map_err(|e| SysError::from_trait_error(e))
    }
    
    // All other methods are IDENTICAL - they all use the OpenProt traits!
    fn set_frequency_internal(&mut self, generic_clock_id: u32, frequency_hz: u64) -> Result<(), SysError> {
        let platform_clock_id = self.mapping.map_clock_id(generic_clock_id)
            .ok_or(SysError::ClockNotFound)?;
            
        self.controller.set_frequency(&platform_clock_id, frequency_hz)
            .map_err(|e| SysError::from_trait_error(e))
    }
    
    fn reset_pulse_internal(&mut self, generic_reset_id: u32, duration_ms: u32) -> Result<(), SysError> {
        let platform_reset_id = self.mapping.map_reset_id(generic_reset_id)
            .ok_or(SysError::InvalidResetId)?;
            
        let duration = core::time::Duration::from_millis(duration_ms as u64);
        self.controller.reset_pulse(&platform_reset_id, duration)
            .map_err(|e| SysError::from_trait_error(e))
    }
}

// The IPC implementation is also IDENTICAL across all chips
impl idl::InOrderSysImpl for SysServer {
    fn enable_clock(
        &mut self,
        _msg: &RecvMessage,
        clock_id: u32,
    ) -> Result<(), RequestError<SysError>> {
        self.enable_clock_internal(clock_id)
            .map_err(RequestError::Runtime)
    }
    
    fn reset_assert(
        &mut self,
        _msg: &RecvMessage,
        reset_id: u32,
    ) -> Result<(), RequestError<SysError>> {
        self.reset_assert_internal(reset_id)
            .map_err(RequestError::Runtime)
    }
    
    fn set_clock_frequency(
        &mut self,
        _msg: &RecvMessage,
        clock_id: u32,
        frequency_hz: u64,
    ) -> Result<(), RequestError<SysError>> {
        self.set_frequency_internal(clock_id, frequency_hz)
            .map_err(RequestError::Runtime)
    }
    
    fn reset_pulse(
        &mut self,
        _msg: &RecvMessage,
        reset_id: u32,
        duration_ms: u32,
    ) -> Result<(), RequestError<SysError>> {
        self.reset_pulse_internal(reset_id, duration_ms)
            .map_err(RequestError::Runtime)
    }
    
    // ... all other IPC methods are identical too
}

#[no_mangle]
fn main() -> ! {
    let mut server = SysServer::new();
    
    // Initialize using OpenProt traits - same for all chips
    server.init().expect("System driver initialization failed");
    
    // Main server loop - identical for all chips
    loop {
        idol_runtime::dispatch(&mut server, INCOMING);
    }
}
```

### 2. Chip-Agnostic IPC Interface

The IPC interface remains generic across all chip families:

```idol
// sys.idol - Generic interface for all chip families
Interface(
    name: "Sys",
    ops: {
        "enable_clock": (
            args: { "clock_id": "u32" },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        "disable_clock": (
            args: { "clock_id": "u32" },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        "set_clock_frequency": (
            args: { 
                "clock_id": "u32",
                "frequency_hz": "u64" 
            },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        "get_clock_frequency": (
            args: { "clock_id": "u32" },
            reply: Result(ok: "u64", err: CLike("SysError")),
            idempotent: true,
        ),
        "reset_assert": (
            args: { "reset_id": "u32" },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        "reset_deassert": (
            args: { "reset_id": "u32" },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        "reset_pulse": (
            args: { 
                "reset_id": "u32",
                "duration_ms": "u32" 
            },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
    }
)
```

### 3. The Mapping Layer

The mapping layer translates between generic Hubris constants and platform-specific HAL types:

```rust
// Common API constants - shared across all chip families
pub mod sys_api {
    // Standard peripheral categories
    pub const CLOCK_GPIO_BASE: u32 = 0x0100_0000;
    pub const CLOCK_SPI_BASE: u32 = 0x0200_0000;
    pub const CLOCK_I2C_BASE: u32 = 0x0300_0000;
    
    pub const RESET_GPIO_BASE: u32 = 0x1100_0000;
    pub const RESET_SPI_BASE: u32 = 0x1200_0000;
    
    // Common identifiers - same across all chips
    pub const CLOCK_GPIOA: u32 = CLOCK_GPIO_BASE + 0;
    pub const CLOCK_SPI1: u32 = CLOCK_SPI_BASE + 0;
    pub const CLOCK_I2C1: u32 = CLOCK_I2C_BASE + 0;
    
    pub const RESET_GPIOA: u32 = RESET_GPIO_BASE + 0;
    pub const RESET_SPI1: u32 = RESET_SPI_BASE + 0;
    pub const RESET_I2C1: u32 = RESET_I2C_BASE + 0;
}

// Platform-specific mapping trait
pub trait ClockResetMapping {
    type ClockId: Clone + PartialEq;
    type ResetId: Clone + PartialEq;
    type ClockConfig: PartialEq;
    
    fn map_clock_id(&self, generic_id: u32) -> Option<Self::ClockId>;
    fn map_reset_id(&self, generic_id: u32) -> Option<Self::ResetId>;
    
    // Platform-specific configuration helpers
    fn get_main_system_clock_id(&self) -> Self::ClockId;
    fn get_optimal_system_frequency(&self) -> u64;
    fn get_main_clock_config(&self) -> Self::ClockConfig;
    fn get_essential_clock_list(&self) -> &[Self::ClockId];
}
```

**ASPEED Mapping Implementation:**
```rust
// aspeed_mapping.rs
use aspeed_rust::syscon::{ClockId as AspeedClockId, ResetId as AspeedResetId};
use aspeed_rust::clocking::ClockConfig as AspeedClockConfig;

pub struct AspeedMapping;

impl ClockResetMapping for AspeedMapping {
    type ClockId = AspeedClockId;
    type ResetId = AspeedResetId;
    type ClockConfig = AspeedClockConfig;
    
    fn map_clock_id(&self, generic_id: u32) -> Option<Self::ClockId> {
        match generic_id {
            CLOCK_GPIOA => Some(AspeedClockId::Gpio),      // ASPEED has unified GPIO clock
            CLOCK_SPI1 => Some(AspeedClockId::Spi1),
            CLOCK_I2C1 => Some(AspeedClockId::I2c1),
            _ => None,
        }
    }
    
    fn map_reset_id(&self, generic_id: u32) -> Option<Self::ResetId> {
        match generic_id {
            RESET_GPIOA => Some(AspeedResetId::Gpio),
            RESET_SPI1 => Some(AspeedResetId::Spi1),
            RESET_I2C1 => Some(AspeedResetId::I2c1),
            _ => None,
        }
    }
    
    fn get_main_system_clock_id(&self) -> Self::ClockId {
        AspeedClockId::SystemClock
    }
    
    fn get_optimal_system_frequency(&self) -> u64 {
        800_000_000  // 800 MHz for AST2600
    }
    
    fn get_main_clock_config(&self) -> Self::ClockConfig {
        AspeedClockConfig {
            source: aspeed_rust::clocking::ClockSource::Crystal25Mhz,
            enable_spread_spectrum: false,
        }
    }
    
    fn get_essential_clock_list(&self) -> &[Self::ClockId] {
        &[AspeedClockId::Gpio, AspeedClockId::Uart1, AspeedClockId::Wdt]
    }
}
```

**STM32 Mapping Implementation:**
```rust
// stm32_mapping.rs  
use stm32f4xx_hal::rcc::{ClockId as Stm32ClockId, ResetId as Stm32ResetId};
use stm32f4xx_hal::rcc::ClockConfig as Stm32ClockConfig;

pub struct Stm32Mapping;

impl ClockResetMapping for Stm32Mapping {
    type ClockId = Stm32ClockId;
    type ResetId = Stm32ResetId;
    type ClockConfig = Stm32ClockConfig;
    
    fn map_clock_id(&self, generic_id: u32) -> Option<Self::ClockId> {
        match generic_id {
            CLOCK_GPIOA => Some(Stm32ClockId::GpioA),      // STM32 has separate GPIO clocks
            CLOCK_SPI1 => Some(Stm32ClockId::Spi1),
            CLOCK_I2C1 => Some(Stm32ClockId::I2c1),
            _ => None,
        }
    }
    
    fn get_optimal_system_frequency(&self) -> u64 {
        168_000_000  // 168 MHz for STM32F407
    }
    
    fn get_main_clock_config(&self) -> Self::ClockConfig {
        Stm32ClockConfig {
            source: stm32f4xx_hal::rcc::ClockSource::Hse,
            pll_m: 8,
            pll_n: 336,
            pll_p: 2,
            pll_q: 7,
        }
    }
    
    // ... other implementations
}
```

### 4. Multi-Platform Build Configuration

```toml
# Cargo.toml
[dependencies]
# OpenProt standardized traits
system-control-traits = { git = "https://github.com/rusty1968/proposed_traits", branch = "caliptra-test" }

# Chip-specific HAL crates (conditional)
aspeed-rust = { git = "https://github.com/AspeedTech-BMC/aspeed-rust", optional = true }
stm32f4xx-hal = { version = "0.14", optional = true, features = ["stm32f407"] }
rp2040-hal = { version = "0.8", optional = true }

# Common Hubris dependencies
userlib = { path = "../../lib/userlib" }
sys_api = { path = "../common/sys-api" }

[features]
default = []
aspeed-ast2600 = ["aspeed-rust"]
stm32f407 = ["stm32f4xx-hal"]
rp2040 = ["rp2040-hal"]
```

## Client Usage

Driver code remains portable across all chip families:

```rust
// SPI driver - identical for all chips
use sys_api::*;

impl SpiServer {
    fn init() -> Result<Self, SpiError> {
        let sys = Sys::from(SYS.get_task_id());
        
        // Uses generic constants - same code for all chips
        sys.enable_clock(CLOCK_SPI1)?;
        sys.set_clock_frequency(CLOCK_SPI1, 10_000_000)?; // 10MHz
        sys.reset_deassert(RESET_SPI1)?;
        
        Ok(SpiServer::new())
    }
}
```

## Key Benefits of OpenProt Integration

**Code Reuse:**
- 99% of system driver code is identical across chip families
- Only mapping layer and HAL instantiation differ
- Dramatically reduces maintenance burden

**Vendor Ecosystem:**
- Chip vendors implement OpenProt traits in their HAL crates
- Community can contribute to standardized interfaces
- Easier to add new chip family support

**Type Safety:**
- Compile-time verification of clock/reset operations
- Platform-specific types prevent cross-chip errors
- Rich error handling with standardized error kinds

**Testing:**
- Common test suite can verify all chip implementations
- Mock implementations for unit testing
- Consistent behavior verification across platforms
