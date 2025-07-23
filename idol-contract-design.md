# Expressing Hardware Abstraction Traits in Idol: ClockControl Example

When migrating from a trait-based hardware abstraction layer to a Hubris-based system using Idol, you'll need to transform your trait interfaces into Idol interface definitions. Let's use the `ClockControl` trait as a concrete example.

## Original Trait Definition

```rust
/// Trait for clock control operations.
/// Abstracts enabling, disabling, and configuring clocks for peripherals or system components.
pub trait ClockControl: ErrorType {
    /// Type for identifying a clock (e.g., peripheral ID, clock name, or register offset).
    type ClockId: Clone + PartialEq;
    /// Type for configuring a clock.
    type ClockConfig: PartialEq;

    fn enable(&mut self, clock_id: &Self::ClockId) -> Result<(), Self::Error>;
    fn disable(&mut self, clock_id: &Self::ClockId) -> Result<(), Self::Error>;
    fn set_frequency(&mut self, clock_id: &Self::ClockId, frequency_hz: u64) -> Result<(), Self::Error>;
    fn get_frequency(&self, clock_id: &Self::ClockId) -> Result<u64, Self::Error>;
    fn configure(&mut self, clock_id: &Self::ClockId, config: Self::ClockConfig) -> Result<(), Self::Error>;
    fn get_config(&self, clock_id: &Self::ClockId) -> Result<Self::ClockConfig, Self::Error>;
}
```

## Transformed Idol Interface

```ron
Interface(
    name: "ClockControl",
    ops: {
        "enable": (
            args: {
                "clock_id": (type: "ClockId"),
            },
            reply: Result(
                ok: "()",
                err: CLike("ClockError"),
            ),
        ),
        "disable": (
            args: {
                "clock_id": (type: "ClockId"),
            },
            reply: Result(
                ok: "()",
                err: CLike("ClockError"),
            ),
        ),
        "set_frequency": (
            args: {
                "clock_id": (type: "ClockId"),
                "frequency_hz": (type: "u64"),
            },
            reply: Result(
                ok: "()",
                err: CLike("ClockError"),
            ),
        ),
        "get_frequency": (
            args: {
                "clock_id": (type: "ClockId"),
            },
            reply: Result(
                ok: "u64",
                err: CLike("ClockError"),
            ),
        ),
        "configure": (
            args: {
                "clock_id": (type: "ClockId"),
            },
            leases: {
                "config": (type: "ClockConfig", read: true),
            },
            reply: Result(
                ok: "()",
                err: CLike("ClockError"),
            ),
        ),
        "get_config": (
            args: {
                "clock_id": (type: "ClockId"),
            },
            leases: {
                "config_out": (type: "ClockConfig", write: true),
            },
            reply: Result(
                ok: "()",
                err: CLike("ClockError"),
            ),
        ),
    },
)
```

## Key Transformation Decisions

### 1. Handling Associated Types

The original trait uses associated types for `ClockId` and `ClockConfig`. In Idol, we need to make these concrete types:

- `ClockId`: Define a concrete type that represents all possible clock identifiers.
  ```rust
  // In a shared crate accessible to both client and server
  #[derive(Copy, Clone, PartialEq, Eq, zerocopy::FromBytes, zerocopy::AsBytes)]
  #[repr(C)]
  pub enum ClockId {
      Cpu = 0,
      Peripheral = 1,
      Usb = 2,
      // ... other clock identifiers
  }
  ```

- `ClockConfig`: For complex configuration types, define a concrete structure:
  ```rust
  #[derive(Copy, Clone, PartialEq, Eq, zerocopy::FromBytes, zerocopy::AsBytes)]
  #[repr(C)]
  pub struct ClockConfig {
      pub source: u8,
      pub divider: u32,
      pub multiplier: u32,
      // ... other configuration fields
  }
  ```

### 2. Self Reference Elimination

The trait methods use `&mut self` and `&self`, but in Idol, there's no implicit "self" - the server maintains state. Instead, we keep track of which client is calling which operations.

### 3. Complex Type Handling

For complex types like `ClockConfig`, we use memory leases:
- For input configurations: Use a read-only lease
- For output configurations: Use a write-only lease

### 4. Error Handling

The trait uses `Self::Error`, but in Idol we define a concrete error type:
```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum ClockError {
    InvalidClock = 0,
    InvalidFrequency = 1,
    InvalidConfig = 2,
    HardwareFailure = 3,
    // ... other error cases
}
```

## Client Usage Example

With the generated Idol client code, usage would look like:

```rust
// Create client instance
let clock_control = ClockControl::from(server_task_id);

// Enable a clock
let result = clock_control.enable(ClockId::Peripheral)?;

// Set frequency
let result = clock_control.set_frequency(ClockId::Cpu, 100_000_000)?;

// Get frequency
let freq = clock_control.get_frequency(ClockId::Usb)?;

// Configure a clock
let config = ClockConfig {
    source: 1,
    divider: 4,
    multiplier: 16,
};
let result = clock_control.configure(ClockId::Peripheral, &config)?;

// Get configuration
let mut current_config = ClockConfig::default();
let result = clock_control.get_config(ClockId::Cpu, &mut current_config)?;
```

## Server Implementation Example

With the generated server trait, implementation would look like:

```rust
struct ClockControlServer {
    // Server state
    clocks: [ClockState; NUM_CLOCKS],
}

impl InOrderClockControlImpl for ClockControlServer {
    fn enable(&mut self, clock_id: ClockId) -> Result<(), ClockError> {
        // Implementation that accesses hardware
        match clock_id {
            ClockId::Cpu => {
                // Enable CPU clock
                Ok(())
            }
            ClockId::Peripheral => {
                // Enable peripheral clock
                Ok(())
            }
            // ...
        }
    }
    
    fn disable(&mut self, clock_id: ClockId) -> Result<(), ClockError> {
        // Implementation
        Ok(())
    }
    
    fn set_frequency(&mut self, clock_id: ClockId, frequency_hz: u64) -> Result<(), ClockError> {
        // Implementation
        Ok(())
    }
    
    fn get_frequency(&mut self, clock_id: ClockId) -> Result<u64, ClockError> {
        // Implementation
        Ok(100_000_000)
    }
    
    fn configure(&mut self, clock_id: ClockId, config: &ClockConfig) -> Result<(), ClockError> {
        // Implementation
        Ok(())
    }
    
    fn get_config(&mut self, clock_id: ClockId, config_out: &mut ClockConfig) -> Result<(), ClockError> {
        // Fill the output buffer with current configuration
        *config_out = ClockConfig {
            source: 1,
            divider: 4,
            multiplier: 16,
        };
        Ok(())
    }
}
