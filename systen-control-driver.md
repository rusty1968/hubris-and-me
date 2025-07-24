
# Hubris System Driver Development Guide

This document explains the critical role of system drivers in Hubris and provides guidance for implementing them in new microcontroller ports, incorporating the new abstraction layer.

## What is the System Driver?

The **system driver** is the foundational driver that provides essential chip-level services to all other drivers and application tasks. It serves as the "hardware abstraction coordinator" for the entire system, managing shared resources that multiple tasks need to access.

In Hubris, the system driver is typically named `drv-{chipfamily}-sys` (e.g., `drv-stm32xx-sys`, `drv-ast1060-sys`) and runs as a high-priority server task that other drivers communicate with via IPC.

## Core Responsibilities

### 1. Clock and Reset Management

The system driver controls peripheral power and reset states, which is fundamental since most microcontroller peripherals are powered down by default. This functionality is now abstracted through the `ClockControl` and `ResetControl` traits.

**Key Operations:**
- **`enable_clock(clock_id)`**: Powers on a specific peripheral by enabling its clock signal
- **`disable_clock(clock_id)`**: Powers down a peripheral to save energy
- **`reset_assert(reset_id)`**: Places a peripheral in reset state for initialization
- **`reset_deassert(reset_id)`**: Releases a peripheral from reset, making it operational

**Example Usage with Abstraction Layer:**

```rust
// SPI driver initialization sequence
let sys = Sys::from(SYS.get_task_id());

// Power on the SPI1 peripheral
sys.enable_clock(&ClockId::Spi1)?;

// Release from reset state
sys.reset_deassert(&ResetId::Spi1)?;

// SPI1 is now ready for configuration and use
```

### 2. GPIO Control and Configuration

GPIO (General Purpose Input/Output) management is shared across many drivers for pin configuration, digital I/O, and alternate function assignment.

**Key Operations:**
- **`gpio_configure(port, pins, config)`**: Configure pin modes, pull resistors, drive strength, alternate functions
- **`gpio_set_reset(port, set_mask, reset_mask)`**: Set and clear digital output pins atomically
- **`gpio_read_input(port)`**: Read current state of input pins
- **`gpio_toggle(port, pins)`**: Toggle output pin states

**Configuration Types:**

```rust
// Pin configuration examples
GpioConfig::Input(Pull::Up)           // Input with pull-up resistor
GpioConfig::Output(OutputType::PushPull) // Push-pull output
GpioConfig::AlternateFunction(5)      // Alternate function 5 (e.g., SPI)
GpioConfig::Analog                    // Analog mode for ADC
```

### 3. Interrupt Multiplexing (EXTI)

The system driver manages GPIO-based interrupts, routing hardware interrupt sources to specific tasks via the notification system.

**Key Operations:**
- **`gpio_irq_configure(mask, sensitivity)`**: Configure interrupt edge detection (rising/falling/both)
- **`gpio_irq_control(mask, operation)`**: Enable, disable, or check interrupt status

**Interrupt Flow:**
1. Task configures GPIO pin as input
2. Task configures interrupt sensitivity via sys driver
3. Task enables interrupt and waits for notification
4. Task handles interrupt and re-enables for next event


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

**Every peripheral driver depends on the system driver** because:
- Peripherals need clocks enabled before they can be accessed
- Pin configuration is required for communication interfaces
- Reset control is needed for reliable initialization
- Shared GPIO resources must be coordinated

### Example: SPI Driver Dependency with Abstraction Layer

```rust
// In SPI driver initialization
impl SpiServer {
    fn init() -> Result<Self, SpiError> {
        let sys = Sys::from(SYS.get_task_id());
        
        // Enable SPI peripheral clock using abstraction
        sys.enable_clock(&ClockId::Spi1)?;
        
        // Configure SPI pins with alternate functions
        sys.gpio_configure(Port::A, pins![5, 6, 7], &[
            GpioConfig::AlternateFunction(5), // SCK
            GpioConfig::AlternateFunction(5), // MOSI  
            GpioConfig::AlternateFunction(5), // MISO
        ])?;
        
        // Release peripheral from reset using abstraction
        sys.reset_deassert(&ResetId::Spi1)?;
        
        // Now SPI registers can be safely accessed
        let spi = unsafe { &(*pac::SPI1::ptr()) };
        // ... SPI configuration
        
        Ok(SpiServer { /* ... */ })
    }
}
```

## Implementation Architecture with Abstraction Layer

### 1. Hardware Abstraction Layer

The new abstraction layer defines traits for common hardware operations that are implemented by platform-specific code:

```rust
// Implementation of abstraction traits for STM32 platform
impl ClockControl for Stm32Sys {
    type ClockId = Stm32Clock;
    type ClockConfig = Stm32ClockConfig;
    type Error = Stm32SysError;
    
    fn enable(&mut self, clock_id: &Self::ClockId) -> Result<(), Self::Error> {
        match clock_id {
            Stm32Clock::Spi1 => {
                let rcc = unsafe { &(*pac::RCC::ptr()) };
                rcc.apb2enr.modify(|_, w| w.spi1en().set_bit());
                Ok(())
            },
            Stm32Clock::I2c1 => {
                // Enable I2C1 clock
                // ...
                Ok(())
            },
            _ => Err(Stm32SysError::ClockNotFound),
        }
    }
    
    // Other method implementations...
}

impl ResetControl for Stm32Sys {
    type ResetId = Stm32Reset;
    type Error = Stm32SysError;
    
    fn reset_assert(&mut self, reset_id: &Self::ResetId) -> Result<(), Self::Error> {
        // Implementation specific to STM32
        // ...
        Ok(())
    }
    
    // Other method implementations...
}

// Error handling with standardized kinds
impl Error for Stm32SysError {
    fn kind(&self) -> ErrorKind {
        match self {
            Stm32SysError::ClockNotFound => ErrorKind::ClockNotFound,
            Stm32SysError::ResetFailed => ErrorKind::HardwareFailure,
            // Map other specific errors to standard kinds
        }
    }
}
```

### 2. IPC Interface Definition

The system driver's API is defined using Hubris's Idol interface definition language, now exposing the abstraction layer's methods:

```idol
// stm32xx-sys.idol
Interface(
    name: "Sys",
    ops: {
        "enable_clock
        ```idol
// stm32xx-sys.idol (continued)
        "enable_clock": (
            args: { "clock_id": "ClockId" },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        "disable_clock": (
            args: { "clock_id": "ClockId" },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        "set_clock_frequency": (
            args: { 
                "clock_id": "ClockId",
                "frequency_hz": "u64" 
            },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        "get_clock_frequency": (
            args: { "clock_id": "ClockId" },
            reply: Result(ok: "u64", err: CLike("SysError")),
            idempotent: true,
        ),
        "configure_clock": (
            args: { 
                "clock_id": "ClockId",
                "config": "ClockConfig" 
            },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        "reset_assert": (
            args: { "reset_id": "ResetId" },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        "reset_deassert": (
            args: { "reset_id": "ResetId" },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        "reset_pulse": (
            args: { 
                "reset_id": "ResetId",
                "duration_ms": "u32" 
            },
            reply: Result(ok: "()", err: CLike("SysError")),
            idempotent: true,
        ),
        // ... other operations for GPIO, etc.
    }
)
```

### 3. Server Implementation

The system driver runs as an IPC server task that implements the abstraction traits and processes requests from other drivers:

```rust
#![no_std]
#![no_main]

use idol_runtime::{RequestError, NotificationHandler};
use userlib::*;
use sys_abstractions::{ClockControl, ResetControl, Error, ErrorKind};

// Include generated server stub
include!(concat!(env!("OUT_DIR"), "/server_stub.rs"));

struct ServerImpl {
    // Hardware-specific fields
    rcc: &'static pac::rcc::RegisterBlock,
    gpio_ports: [&'static pac::gpioa::RegisterBlock; 8],
    // ... other hardware references
}

// Implementation of the abstraction traits
impl ClockControl for ServerImpl {
    type ClockId = ChipClock;
    type ClockConfig = ChipClockConfig;
    type Error = SysError;
    
    fn enable(&mut self, clock_id: &Self::ClockId) -> Result<(), Self::Error> {
        // Implementation specific to the chip
        match clock_id {
            ChipClock::Spi1 => {
                self.rcc.apb2enr.modify(|_, w| w.spi1en().set_bit());
                Ok(())
            },
            // Handle other clocks...
            _ => Err(SysError::ClockNotFound),
        }
    }
    
    // Implement other ClockControl methods...
    fn disable(&mut self, clock_id: &Self::ClockId) -> Result<(), Self::Error> {
        // Implementation
    }
    
    fn set_frequency(&mut self, clock_id: &Self::ClockId, frequency_hz: u64) -> Result<(), Self::Error> {
        // Implementation
    }
    
    fn get_frequency(&self, clock_id: &Self::ClockId) -> Result<u64, Self::Error> {
        // Implementation
    }
    
    fn configure(&mut self, clock_id: &Self::ClockId, config: Self::ClockConfig) -> Result<(), Self::Error> {
        // Implementation
    }
    
    fn get_config(&self, clock_id: &Self::ClockId) -> Result<Self::ClockConfig, Self::Error> {
        // Implementation
    }
}

impl ResetControl for ServerImpl {
    type ResetId = ChipReset;
    type Error = SysError;
    
    fn reset_assert(&mut self, reset_id: &Self::ResetId) -> Result<(), Self::Error> {
        // Implementation
        match reset_id {
            ChipReset::Spi1 => {
                self.rcc.apb2rstr.modify(|_, w| w.spi1rst().set_bit());
                Ok(())
            },
            // Handle other resets...
            _ => Err(SysError::InvalidResetId),
        }
    }
    
    // Implement other ResetControl methods...
    fn reset_deassert(&mut self, reset_id: &Self::ResetId) -> Result<(), Self::Error> {
        // Implementation
    }
    
    fn reset_pulse(&mut self, reset_id: &Self::ResetId, duration: Duration) -> Result<(), Self::Error> {
        self.reset_assert(reset_id)?;
        // Wait for specified duration
        // Implementation-specific delay
        self.reset_deassert(reset_id)
    }
    
    fn reset_is_asserted(&self, reset_id: &Self::ResetId) -> Result<bool, Self::Error> {
        // Implementation
    }
}

// Implement Error trait for SysError
impl Error for SysError {
    fn kind(&self) -> ErrorKind {
        match self {
            SysError::ClockNotFound => ErrorKind::ClockNotFound,
            SysError::InvalidResetId => ErrorKind::InvalidResetId,
            SysError::ConfigurationFailed => ErrorKind::ClockConfigurationFailed,
            // Map other specific errors...
            _ => ErrorKind::HardwareFailure,
        }
    }
}

// Implement server IPC operations
impl idl::InOrderSysImpl for ServerImpl {
    fn enable_clock(
        &mut self,
        _msg: &RecvMessage,
        clock_id: ChipClock,
    ) -> Result<(), RequestError<SysError>> {
        // Delegate to the ClockControl trait implementation
        ClockControl::enable(self, &clock_id)
            .map_err(RequestError::Runtime)
    }
    
    fn reset_assert(
        &mut self,
        _msg: &RecvMessage,
        reset_id: ChipReset,
    ) -> Result<(), RequestError<SysError>> {
        // Delegate to the ResetControl trait implementation
        ResetControl::reset_assert(self, &reset_id)
            .map_err(RequestError::Runtime)
    }
    
    // Implement other IPC operations...
}

#[no_mangle]
fn main() -> ! {
    let mut server = ServerImpl::new();
    
    // System initialization
    system_init();
    
    // Main server loop
    loop {
        idol_runtime::dispatch(&mut server, INCOMING);
    }
}

fn system_init() {
    // Basic system clock configuration
    configure_system_clock();
    
    // Enable GPIO port clocks (usually needed for all GPIO operations)
    enable_gpio_clocks();
    
    // Any other chip-specific initialization
    configure_flash_wait_states();
}
```

## Design Patterns and Principles with Abstraction Layer

### 1. Trait-Based Abstraction

The new abstraction layer leverages Rust's trait system to provide a clear interface between hardware-specific implementation and generic driver code:

```rust
// Generic code that works with any system implementation
fn configure_spi<T: ClockControl + ResetControl>(
    sys: &mut T,
    clock_id: &T::ClockId,
    reset_id: &T::ResetId
) -> Result<(), T::Error> 
where 
    T::Error: Error
{
    // Enable the clock
    sys.enable(clock_id)?;
    
    // Reset the peripheral
    sys.reset_assert(reset_id)?;
    sys.reset_deassert(reset_id)?;
    
    Ok(())
}
```







