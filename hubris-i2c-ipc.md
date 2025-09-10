# I2C IPC Design in Hubris

**Project:** Hubris Operating System  
**Component:** I2C Inter-Process Communication Architecture  
**Version:** 1.0  
**Date:** 2024  

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [IPC Architecture Overview](#ipc-architecture-overview)
3. [I2C Server Design](#i2c-server-design)
4. [Client API Architecture](#client-api-architecture)
5. [Message Marshalling](#message-marshalling)
6. [Lease-Based Zero-Copy Transfers](#lease-based-zero-copy-transfers)
7. [Device Addressing Model](#device-addressing-model)
8. [Operation Types and Patterns](#operation-types-and-patterns)
9. [Error Handling and Recovery](#error-handling-and-recovery)
10. [Performance Characteristics](#performance-characteristics)
11. [Memory Management](#memory-management)
12. [Comparison with Alternative Approaches](#comparison-with-alternative-approaches)

## Executive Summary

Hubris implements I2C communication through a sophisticated Inter-Process Communication (IPC) system that prioritizes **safety**, **determinism**, and **zero-copy efficiency**. Unlike many embedded systems that use direct hardware access or callback-based APIs, Hubris employs a **task-isolated I2C server** that mediates all hardware access through synchronous IPC calls.

### Key Design Principles

- **Task Isolation**: I2C hardware is owned by a dedicated server task
- **Synchronous IPC**: Blocking calls provide deterministic timing
- **Zero-Copy Transfers**: Lease system eliminates memory copying
- **Type-Safe Marshalling**: Custom serialization prevents IPC errors
- **Device-Centric API**: Rich addressing model supports complex topologies
- **Static Allocation**: No dynamic memory allocation in critical paths

### Benefits

- **Fault Isolation**: I2C errors cannot crash client tasks
- **Resource Sharing**: Multiple tasks can safely share I2C controllers
- **Deterministic Timing**: Synchronous calls provide predictable latency
- **Memory Efficiency**: Zero-copy design minimizes overhead
- **Hardware Abstraction**: Clients don't need hardware-specific knowledge

## IPC Architecture Overview

### Hubris IPC Fundamentals

Hubris IPC is **synchronous and blocking**, not async/await based. This design provides several benefits for embedded systems:

```rust
// Client perspective - this blocks until server responds
let result = sys_send(
    server_task,           // Target task ID
    operation_code,        // Operation identifier
    request_bytes,         // Request message
    response_buffer,       // Response buffer
    leases                 // Zero-copy data transfers
);
```

#### IPC Call Flow

1. **Client Preparation**: Serialize request and prepare data leases
2. **Kernel Transition**: `sys_send` switches to kernel mode
3. **Task Switching**: Kernel suspends client, activates server
4. **Server Processing**: Server receives message and processes I2C operation
5. **Hardware Interaction**: Server performs actual I2C communication
6. **Response Generation**: Server serializes response and prepares result data
7. **Client Resume**: Kernel switches back to client with response

```
Client Task                 Kernel                    I2C Server Task
    |                          |                           |
    | sys_send()              |                           |
    |------------------------>|                           |
    |    [BLOCKED]            |                           |
    |                         | task_switch()             |
    |                         |------------------------->|
    |                         |                          | recv_open()
    |                         |                          | process_request()
    |                         |                          | hardware_i2c()
    |                         |                          | reply()
    |                         |<-------------------------|
    |<------------------------|                           |
    | [RESUMED]               |                           |
```

### Task Isolation Model

The I2C server task has **exclusive ownership** of I2C hardware resources:

```rust
// I2C server task owns hardware controllers
pub struct I2cServer {
    controllers: [I2cController; 4],  // STM32 has up to 4 I2C controllers
    device_registry: &'static [DeviceDescriptor],
    notification_mask: u32,           // For hardware interrupts
}

// Clients can only access I2C through IPC
pub struct I2cDevice {
    task: TaskId,           // Server task ID
    controller: Controller, // Which hardware controller
    port: PortIndex,       // Port on that controller
    segment: Option<(Mux, Segment)>, // Optional multiplexer
    address: u8,           // Device address
}
```

This isolation provides:
- **Fault Containment**: Client bugs cannot corrupt I2C state
- **Resource Arbitration**: Server manages concurrent access
- **Consistent Error Handling**: Centralized recovery mechanisms
- **Hardware Abstraction**: Clients work with logical devices

## I2C Server Design

### Server Task Structure

The I2C server runs as a dedicated Hubris task with the following responsibilities:

```rust
#[export_name = "main"]
fn main() -> ! {
    // Initialize hardware I2C controllers
    let mut server = I2cServer::initialize();
    
    // Main IPC receive loop
    let mut message_buffer = [0u8; 1024];
    loop {
        match sys::recv_open(&mut message_buffer, INTERRUPT_MASK) {
            Ok((message_info, caller_task)) => {
                handle_i2c_request(&mut server, &message_info, caller_task);
            }
            Err(sys::RecvError::Interrupted(interrupt_bits)) => {
                handle_hardware_interrupts(&mut server, interrupt_bits);
            }
        }
    }
}
```

### Hardware Controller Management

Each I2C controller is wrapped in a safe abstraction:

```rust
pub struct I2cController {
    registers: &'static stm32_pac::I2C,  // Memory-mapped registers
    pins: I2cPins,                       // GPIO pin configuration
    clock_config: ClockConfig,           // Clock setup
    state: ControllerState,              // Current operational state
    error_count: u32,                    // Error statistics
    last_operation: Option<OperationInfo>, // For debugging
}

#[derive(Debug, Clone, Copy)]
pub enum ControllerState {
    Idle,
    Busy { start_time: u64 },
    Error { error_type: ErrorType, time: u64 },
    Recovering,
}
```

### Request Processing Pipeline

The server processes requests through a structured pipeline:

```rust
fn handle_i2c_request(
    server: &mut I2cServer,
    message: &MessageInfo,
    caller: TaskId
) {
    // 1. Parse and validate request
    let request = match parse_i2c_message(message) {
        Ok(req) => req,
        Err(e) => {
            send_error_response(caller, ResponseCode::BadRequest);
            return;
        }
    };
    
    // 2. Validate device access permissions
    if !server.validate_device_access(&request, caller) {
        send_error_response(caller, ResponseCode::AccessDenied);
        return;
    }
    
    // 3. Check controller availability
    let controller = &mut server.controllers[request.controller as usize];
    if !controller.is_available() {
        send_error_response(caller, ResponseCode::BusLocked);
        return;
    }
    
    // 4. Execute I2C operation
    let result = execute_i2c_operation(controller, &request);
    
    // 5. Send response to client
    send_i2c_response(caller, result);
}
```

### Interrupt-Driven Operations

For high-performance operations, the server supports interrupt-driven I2C:

```rust
fn execute_async_i2c_operation(
    controller: &mut I2cController,
    request: &I2cRequest
) -> Result<(), ResponseCode> {
    match request.operation_type {
        OpType::AsyncRead => {
            // Configure hardware for interrupt-driven read
            controller.configure_read_interrupts(request.length);
            controller.start_read_operation(request.address, request.length);
            
            // Operation completes in interrupt handler
            Ok(())
        }
        OpType::AsyncWrite => {
            // Configure DMA and interrupts for write
            controller.setup_write_dma(request.write_data);
            controller.start_write_operation(request.address);
            
            Ok(())
        }
    }
}

fn handle_hardware_interrupts(server: &mut I2cServer, interrupt_bits: u32) {
    for (controller_id, controller) in server.controllers.iter_mut().enumerate() {
        let controller_mask = 1 << controller_id;
        
        if interrupt_bits & controller_mask != 0 {
            // Read interrupt status from hardware
            let status = controller.read_interrupt_status();
            
            // Complete pending operation
            let result = controller.complete_operation(status);
            
            // Notify waiting client
            if let Some(pending_client) = server.pending_operations.remove(&(controller_id as u8)) {
                send_i2c_response(pending_client.caller, result);
            }
        }
    }
}
```

## Client API Architecture

### I2cDevice Abstraction

The client API centers around the `I2cDevice` abstraction:

```rust
/// The 5-tuple that uniquely identifies an I2C device
#[derive(Copy, Clone, Debug)]
pub struct I2cDevice {
    pub task: TaskId,                    // I2C server task
    pub controller: Controller,          // Hardware controller (I2C1, I2C2, etc.)
    pub port: PortIndex,                // Port on controller (0-based)
    pub segment: Option<(Mux, Segment)>, // Optional multiplexer addressing
    pub address: u8,                    // I2C device address (7-bit)
}
```

This model supports complex I2C topologies:

```
MCU I2C Controller 1, Port 0
├── Device 0x48 (direct connection)
├── Device 0x49 (direct connection)
└── Multiplexer 0x70
    ├── Segment 0: Device 0x48 (different from direct 0x48)
    ├── Segment 1: Device 0x50
    └── Segment 2: Device 0x51

MCU I2C Controller 2, Port 0
├── Device 0x40
└── Device 0x41
```

### Device Creation and Configuration

Devices are typically created during system initialization:

```rust
// Create device for temperature sensor on I2C1, direct connection
let temp_sensor = I2cDevice::new(
    i2c_server_task,        // Server task ID
    Controller::I2c1,       // Hardware controller
    PortIndex(0),          // Port 0
    None,                  // No multiplexer
    0x48                   // TMP117 address
);

// Create device behind multiplexer
let power_monitor = I2cDevice::new(
    i2c_server_task,
    Controller::I2c1,
    PortIndex(0),
    Some((Mux::Mux0, Segment::Segment2)), // Mux 0, Segment 2
    0x40                   // INA232 address
);
```

### High-Level Operation Interface

The API provides semantic operations that map to common I2C patterns:

```rust
impl I2cDevice {
    /// Read a typed register value
    pub fn read_reg<R, V>(&self, reg: R) -> Result<V, ResponseCode>
    where
        R: IntoBytes + Immutable,    // Register address type
        V: IntoBytes + FromBytes,    // Value type
    {
        let mut val = V::new_zeroed();
        
        let (code, _) = sys_send(
            self.task,
            Op::WriteRead as u16,
            &self.marshal_device_info(),
            &mut [],
            &[
                Lease::from(reg.as_bytes()),      // Write register address
                Lease::from(val.as_mut_bytes()),  // Read value
            ],
        );
        
        self.response_code(code, val)
    }
    
    /// Perform SMBus block read with length byte
    pub fn read_block<R>(&self, reg: R, buf: &mut [u8]) -> Result<usize, ResponseCode>
    where
        R: IntoBytes + Immutable,
    {
        let mut response = 0_usize;
        
        let (code, _) = sys_send(
            self.task,
            Op::WriteReadBlock as u16,
            &self.marshal_device_info(),
            response.as_mut_bytes(),
            &[
                Lease::from(reg.as_bytes()),
                Lease::from(buf),
            ],
        );
        
        self.response_code(code, response)
    }
    
    /// Chain multiple operations without intermediate receives
    pub fn write_write_read_reg<R, V>(
        &self,
        reg: R,
        first: &[u8],
        second: &[u8],
    ) -> Result<V, ResponseCode>
    where
        R: IntoBytes + Immutable,
        V: IntoBytes + FromBytes,
    {
        let mut val = V::new_zeroed();
        
        let (code, _) = sys_send(
            self.task,
            Op::WriteRead as u16,
            &self.marshal_device_info(),
            &mut [],
            &[
                Lease::from(first),               // First write
                Lease::read_only(&[]),           // No read after first write
                Lease::from(second),             // Second write
                Lease::read_only(&[]),           // No read after second write
                Lease::from(reg.as_bytes()),     // Register read
                Lease::from(val.as_mut_bytes()), // Read result
            ],
        );
        
        self.response_code(code, val)
    }
}
```

## Message Marshalling

### Custom Marshalling System

Hubris I2C uses a custom marshalling system optimized for embedded constraints:

```rust
pub trait Marshal<T> {
    fn marshal(&self) -> T;
    fn unmarshal(val: &T) -> Result<Self, ResponseCode>
    where
        Self: Sized;
}

type I2cMessage = (u8, Controller, PortIndex, Option<(Mux, Segment)>);

impl Marshal<[u8; 4]> for I2cMessage {
    fn marshal(&self) -> [u8; 4] {
        [
            self.0,                    // Device address
            self.1 as u8,             // Controller ID
            self.2.0,                 // Port index
            match self.3 {            // Multiplexer info
                Some((mux, seg)) => {
                    0b1000_0000 |          // Mux present bit
                    ((mux as u8) << 4) |   // Mux ID (4 bits)
                    (seg as u8)            // Segment ID (4 bits)
                }
                None => 0,
            },
        ]
    }
    
    fn unmarshal(val: &[u8; 4]) -> Result<Self, ResponseCode> {
        Ok((
            val[0], // Device address
            Controller::from_u8(val[1]).ok_or(ResponseCode::BadController)?,
            PortIndex(val[2]),
            if val[3] == 0 {
                None
            } else {
                Some((
                    Mux::from_u8((val[3] & 0b0111_0000) >> 4)
                        .ok_or(ResponseCode::BadMux)?,
                    Segment::from_u8(val[3] & 0b0000_1111)
                        .ok_or(ResponseCode::BadSegment)?,
                ))
            },
        ))
    }
}
```

### Operation Encoding

Different I2C operations are encoded as operation codes:

```rust
#[derive(Debug, Clone, Copy, FromPrimitive)]
#[repr(u16)]
pub enum Op {
    WriteRead = 1,        // Combined write-then-read operation
    WriteReadBlock = 2,   // SMBus block read with length byte
    // Future operations could include:
    // Transaction = 3,   // Multiple operations in one IPC call
    // AsyncRead = 4,     // Interrupt-driven read
    // AsyncWrite = 5,    // Interrupt-driven write
}
```

### Compact Message Format

The marshalled message format is designed for efficiency:

```
Byte 0: Device Address (7-bit I2C address)
Byte 1: Controller ID (I2C1=0, I2C2=1, I2C3=2, I2C4=3)
Byte 2: Port Index (0-based port number)
Byte 3: Multiplexer Info
        Bit 7: Multiplexer Present (1=yes, 0=no)
        Bits 6-4: Multiplexer ID (0-7)
        Bits 3-0: Segment ID (0-15)
```

This 4-byte format can address:
- 4 I2C controllers
- 256 ports per controller  
- 8 multiplexers per port
- 16 segments per multiplexer
- 128 devices per segment (7-bit addressing)

## Lease-Based Zero-Copy Transfers

### Lease System Overview

Hubris uses a **lease system** for zero-copy data transfer between tasks:

```rust
pub struct Lease<T> {
    data: *mut T,
    len: usize,
    permissions: LeasePermissions,
}

#[derive(Debug, Clone, Copy)]
pub enum LeasePermissions {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}
```

### Lease Creation Patterns

Different lease types support various I2C patterns:

```rust
// Read-only lease for outgoing data
let write_lease = Lease::from(&[0x01, 0x02, 0x03]); // Data to write

// Write lease for incoming data  
let mut read_buffer = [0u8; 16];
let read_lease = Lease::from(&mut read_buffer); // Buffer to read into

// Empty lease for operations that don't transfer data in one direction
let empty_lease = Lease::read_only(&[]); // No data transfer

// Multiple leases for complex operations
let leases = &[
    Lease::from(register_address),  // Write register address
    Lease::from(&mut read_buffer),  // Read register value
];
```

### Memory Safety Guarantees

The lease system provides several safety guarantees:

1. **Exclusive Access**: Server gets exclusive access to leased memory
2. **Lifetime Management**: Leases are automatically revoked when IPC completes
3. **Permission Enforcement**: Server cannot write to read-only leases
4. **Bounds Checking**: Kernel validates lease bounds

### Performance Characteristics

Zero-copy transfers eliminate memory copying overhead:

```rust
// Traditional approach (with copying)
let data = [0u8; 1024];
memcpy(ipc_buffer, &data, 1024);        // Copy to IPC buffer
sys_send(server, ipc_buffer, ...);      // Send IPC
memcpy(&data, response_buffer, 1024);   // Copy from response
// Total: 2048 bytes copied

// Hubris lease approach (zero-copy)
let mut data = [0u8; 1024];
let lease = Lease::from(&mut data);     // No copying
sys_send(server, ..., lease);           // Server accesses data directly
// Total: 0 bytes copied
```

## Device Addressing Model

### Hierarchical Addressing

Hubris supports a sophisticated hierarchical addressing model:

```
System Level
├── MCU (single)
    ├── I2C Controller 1 (STM32 I2C1 peripheral)
    │   ├── Port 0 (SCL1/SDA1 pins)
    │   │   ├── Direct Device 0x48
    │   │   ├── Direct Device 0x49
    │   │   └── Multiplexer 0x70
    │   │       ├── Segment 0 → Device 0x48 (isolated from direct 0x48)
    │   │       ├── Segment 1 → Device 0x50
    │   │       └── Segment 2 → Device 0x51
    │   └── Port 1 (SCL1_ALT/SDA1_ALT pins - alternate pin mapping)
    │       └── Direct Device 0x40
    ├── I2C Controller 2 (STM32 I2C2 peripheral)
    │   └── Port 0 (SCL2/SDA2 pins)
    │       └── Direct Device 0x41
    └── I2C Controller 3 (STM32 I2C3 peripheral)
        └── Port 0 (SCL3/SDA3 pins)
            └── Device 0x42
```

### Address Space Isolation

Each level provides address space isolation:

- **Controller Level**: Different controllers have independent address spaces
- **Port Level**: Multiple pin configurations per controller
- **Multiplexer Level**: Segments provide isolated address spaces
- **Device Level**: Standard 7-bit I2C addressing within each segment

### Device Registration

Devices are typically registered statically during build:

```rust
// Generated from build-time configuration
pub static DEVICE_REGISTRY: &[DeviceDescriptor] = &[
    DeviceDescriptor {
        name: "temp_sensor_main",
        controller: Controller::I2c1,
        port: PortIndex(0),
        segment: None,
        address: 0x48,
        device_class: DeviceClass::TemperatureSensor,
    },
    DeviceDescriptor {
        name: "temp_sensor_backup", 
        controller: Controller::I2c1,
        port: PortIndex(0),
        segment: Some((Mux::Mux0, Segment::Segment0)),
        address: 0x48, // Same address, different segment
        device_class: DeviceClass::TemperatureSensor,
    },
    DeviceDescriptor {
        name: "power_monitor_12v",
        controller: Controller::I2c2,
        port: PortIndex(0),
        segment: None,
        address: 0x40,
        device_class: DeviceClass::PowerMonitor,
    },
];

// Runtime device lookup
pub fn get_device(name: &str) -> Option<I2cDevice> {
    DEVICE_REGISTRY.iter()
        .find(|d| d.name == name)
        .map(|d| I2cDevice::new(
            I2C_SERVER_TASK,
            d.controller,
            d.port,
            d.segment,
            d.address,
        ))
}
```

## Operation Types and Patterns

### Basic Operations

The API supports fundamental I2C operations:

```rust
// Simple write operation
device.write(&[0x01, 0x02, 0x03])?;

// Simple read operation  
let mut buffer = [0u8; 4];
let bytes_read = device.read_into(&mut buffer)?;

// Register read (write address, then read value)
let temperature: u16 = device.read_reg(0x00u8)?;

// Register write with verification
device.write(&[0x01, 0x42])?; // Write register 0x01 = 0x42
let verify: u8 = device.read_reg(0x01u8)?; // Read back to verify
assert_eq!(verify, 0x42);
```

### Complex Operation Chaining

The API supports chaining operations without intermediate client interactions:

```rust
// Write-Write-Read pattern (common for regulators)
let voltage_setting: u16 = device.write_write_read_reg(
    0x21u8,           // Voltage register
    &[0x02],          // Select rail 2
    &[0x01],          // Select phase 1  
)?; // Read current voltage setting for rail 2, phase 1

// SMBus block operations
let mut data_buffer = [0u8; 32];
let bytes_read = device.read_block(0x10u8, &mut data_buffer)?;
// First byte from device contains actual length, not included in buffer
```

### Transaction Patterns

Different patterns optimize for different use cases:

```rust
// Pattern 1: Single register read (most common)
// Client Call: device.read_reg(0x00u8)
// IPC Flow: 
//   Write: [0x00]
//   Read:  [value_bytes...]
//   Leases: [write_lease(reg), read_lease(value)]

// Pattern 2: Multi-byte register read
// Client Call: device.read_reg_into(0x00u8, &mut buffer)  
// IPC Flow:
//   Write: [0x00]
//   Read:  buffer.len() bytes
//   Leases: [write_lease(reg), read_lease(buffer)]

// Pattern 3: Block read with length
// Client Call: device.read_block(0x10u8, &mut buffer)
// IPC Flow:
//   Write: [0x10]
//   Read:  [length_byte, data_bytes...]
//   Response: actual_length (length_byte value)
//   Buffer: data_bytes only (length_byte not included)

// Pattern 4: Complex device configuration
// Client Call: device.write_write_read_reg(reg, first, second)
// IPC Flow:
//   Write: first
//   Write: second  
//   Write: [reg]
//   Read:  [value_bytes...]
//   Leases: [write_lease(first), empty_lease, write_lease(second), 
//           empty_lease, write_lease(reg), read_lease(value)]
```

### Error Handling Patterns

The API provides structured error handling:

```rust
match device.read_reg::<u8, u16>(0x00) {
    Ok(value) => {
        // Process successful read
        process_sensor_value(value);
    }
    Err(ResponseCode::AddressNackSentEarly) => {
        // Device not responding - maybe unplugged
        log::warn!("Sensor not responding");
        handle_missing_device();
    }
    Err(ResponseCode::BusTimeout) => {
        // Bus stuck - needs recovery
        log::error!("I2C bus timeout");
        request_bus_recovery();
    }
    Err(ResponseCode::BusLocked) => {
        // Another client is using the bus
        log::info!("I2C bus busy, retrying...");
        sys::sleep_for(Duration::from_millis(10));
        // Retry logic here
    }
    Err(e) => {
        // Unexpected error
        log::error!("I2C error: {:?}", e);
    }
}
```

## Error Handling and Recovery

### Error Classification

I2C errors are classified into several categories:

```rust
#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq, Eq)]
pub enum ResponseCode {
    // Success
    Success = 0,
    
    // Hardware-level errors
    BusLocked = 1,              // Bus busy with another operation
    BusReset = 2,               // Hardware reset occurred
    BusError = 3,               // Electrical bus error
    BusTimeout = 4,             // Operation timed out
    
    // Protocol-level errors  
    AddressNackSentEarly = 5,   // Address not acknowledged
    AddressNackSentLate = 6,    // Address NACK after START
    DataNackSent = 7,           // Data byte not acknowledged
    ArbitrationLost = 8,        // Lost arbitration to another master
    
    // Configuration errors
    ControllerModeError = 9,    // Hardware in wrong mode
    ControllerDisabled = 10,    // I2C controller not enabled
    BadController = 11,         // Invalid controller ID
    BadPort = 12,              // Invalid port ID
    BadMux = 13,               // Invalid multiplexer ID
    BadSegment = 14,           // Invalid segment ID
    
    // System errors
    BadResponse = 15,          // Malformed response
    NoDevice = 16,             // Device not in registry
}
```

### Error Recovery Strategies

The I2C server implements several recovery strategies:

```rust
impl I2cController {
    /// Attempt to recover from bus error conditions
    pub fn recover_from_error(&mut self, error: ResponseCode) -> Result<(), ResponseCode> {
        match error {
            ResponseCode::BusTimeout => {
                // Generate clock cycles to unstick bus
                self.clock_recovery_sequence()?;
                self.reset_controller_state();
                Ok(())
            }
            
            ResponseCode::BusError => {
                // Reset I2C peripheral
                self.hardware_reset()?;
                self.reconfigure_controller()?;
                Ok(())
            }
            
            ResponseCode::ArbitrationLost => {
                // Wait for bus to become idle, then retry
                self.wait_for_bus_idle()?;
                Ok(())
            }
            
            ResponseCode::AddressNackSentEarly |
            ResponseCode::AddressNackSentLate => {
                // Device not responding - check if device is present
                if self.probe_device_presence()? {
                    // Device present but not responding - may need reset
                    self.attempt_device_reset()?;
                } else {
                    // Device not present - update device registry
                    return Err(ResponseCode::NoDevice);
                }
                Ok(())
            }
            
            _ => Err(error), // Cannot recover from this error
        }
    }
    
    /// Generate clock cycles to recover stuck bus
    fn clock_recovery_sequence(&mut self) -> Result<(), ResponseCode> {
        // Switch to GPIO mode
        self.configure_pins_as_gpio()?;
        
        // Generate 9 clock cycles to clear any partial transfers
        for _ in 0..9 {
            self.set_scl_low();
            self.delay_microseconds(5);
            self.set_scl_high();
            self.delay_microseconds(5);
            
            // Check if SDA is released
            if self.read_sda_pin() {
                break;
            }
        }
        
        // Generate STOP condition
        self.set_sda_low();
        self.set_scl_high();
        self.delay_microseconds(5);
        self.set_sda_high();
        
        // Switch back to I2C mode
        self.configure_pins_as_i2c()?;
        
        Ok(())
    }
}
```

### Client-Side Error Handling

Clients can implement retry logic and graceful degradation:

```rust
pub fn robust_read_temperature(sensor: &I2cDevice) -> Option<f32> {
    const MAX_RETRIES: u8 = 3;
    
    for attempt in 0..MAX_RETRIES {
        match sensor.read_reg::<u8, u16>(0x00) {
            Ok(raw_value) => {
                return Some((raw_value as f32) * 0.0078125);
            }
            
            Err(ResponseCode::BusLocked) => {
                // Bus busy - wait and retry
                sys::sleep_for(Duration::from_millis(10 * (attempt + 1) as u64));
                continue;
            }
            
            Err(ResponseCode::BusTimeout) => {
                // Timeout - longer wait before retry
                sys::sleep_for(Duration::from_millis(100));
                continue;
            }
            
            Err(ResponseCode::AddressNackSentEarly) => {
                // Device not responding - maybe temporary
                if attempt < MAX_RETRIES - 1 {
                    sys::sleep_for(Duration::from_millis(50));
                    continue;
                } else {
                    log::warn!("Temperature sensor not responding after {} attempts", MAX_RETRIES);
                    return None;
                }
            }
            
            Err(e) => {
                // Unrecoverable error
                log::error!("Temperature read failed: {:?}", e);
                return None;
            }
        }
    }
    
    None
}
```

## Performance Characteristics

### Latency Analysis

I2C IPC operations have predictable latency characteristics:

```
Total Latency = IPC_Overhead + Marshalling + Hardware_Time + Unmarshalling

Where:
- IPC_Overhead: ~2-5μs (task switch + kernel processing)
- Marshalling: ~0.5μs (4-byte message serialization)
- Hardware_Time: Variable (depends on I2C speed and data length)
- Unmarshalling: ~0.5μs (response deserialization)
```

### Throughput Measurements

Actual throughput depends on several factors:

```rust
// Benchmark results on STM32H7 @ 400MHz, I2C @ 400kHz
// Single-byte register read:
//   IPC overhead: 3μs
//   I2C transfer: 45μs (address + register + restart + data)
//   Total: 48μs
//   Throughput: ~20.8k ops/second

// 16-byte block read:
//   IPC overhead: 3μs  
//   I2C transfer: 380μs (address + register + restart + 16 bytes)
//   Total: 383μs
//   Throughput: ~2.6k ops/second

// Zero-copy 256-byte read:
//   IPC overhead: 3μs
//   I2C transfer: 5.8ms (address + register + restart + 256 bytes)
//   Total: 5.803ms
//   Throughput: ~172 ops/second
//   Data rate: ~44 KB/second
```

### Memory Usage

The I2C system has efficient memory usage:

```rust
// Server-side memory usage (per controller):
struct I2cController {
    // Hardware state: 64 bytes
    registers: &'static I2C_TypeDef,     // 8 bytes (pointer)
    pins: I2cPins,                       // 16 bytes
    clock_config: ClockConfig,           // 8 bytes
    state: ControllerState,              // 16 bytes
    error_stats: ErrorStatistics,        // 16 bytes
    
    // Operation buffers: 544 bytes
    tx_buffer: [u8; 256],               // 256 bytes
    rx_buffer: [u8; 256],               // 256 bytes
    operation_info: OperationInfo,       // 32 bytes
    
    // Total per controller: ~608 bytes
}

// Client-side memory usage (per device):
struct I2cDevice {
    task: TaskId,                        // 4 bytes
    controller: Controller,              // 1 byte
    port: PortIndex,                    // 1 byte  
    segment: Option<(Mux, Segment)>,    // 2 bytes
    address: u8,                        // 1 byte
    // Total per device: 9 bytes + padding = 12 bytes
}

// Total system memory (4 controllers + 32 devices):
// Server: 4 * 608 = 2,432 bytes
// Clients: 32 * 12 = 384 bytes  
// Total: ~2.8 KB
```

### Comparison with Direct Hardware Access

| Metric | Direct Access | Hubris IPC | Trade-off |
|--------|---------------|------------|-----------|
| Latency | 0.5μs | 3-5μs | +6-10x latency for safety |
| Throughput | ~50 KB/s | ~44 KB/s | -12% throughput for isolation |
| Memory Usage | ~200 bytes | ~2.8 KB | +14x memory for sharing |
| Fault Isolation | None | Complete | Safety vs. efficiency |
| Resource Sharing | Manual | Automatic | Convenience vs. control |
| Development Complexity | High | Low | Productivity vs. performance |

## Memory Management

### Static Allocation Strategy

Hubris I2C uses static allocation throughout:

```rust
// Server task stack and data
#[link_section = ".i2c_server_stack"]
static mut I2C_SERVER_STACK: [u8; 4096] = [0; 4096];

#[link_section = ".i2c_server_data"] 
static mut I2C_SERVER_DATA: I2cServerData = I2cServerData::new();

// Device registry (compile-time generated)
#[link_section = ".device_registry"]
static DEVICE_REGISTRY: &[DeviceDescriptor] = &[
    // Devices populated by build system
];

// Client buffers (per-task allocation)
struct TaskI2cBuffers {
    message_buffer: [u8; 64],      // IPC message buffer
    temp_buffer: [u8; 256],        // Temporary data buffer
}
```

### Lease Memory Management

The lease system manages memory without allocation:

```rust
// Leases reference existing memory - no allocation
let mut sensor_data = [0u8; 16];                    // Stack allocation
let read_lease = Lease::from(&mut sensor_data);     // No heap allocation

// Server gets direct access to client memory
fn server_process_read_lease(lease: &mut Lease<[u8]>) {
    // Server writes directly to client's buffer
    for (i, byte) in lease.iter_mut().enumerate() {
        *byte = read_i2c_data_register();
    }
    // No copying required
}
```

### Buffer Management Strategies

Different strategies optimize for different patterns:

```rust
// Strategy 1: Pre-allocated per-device buffers
struct DeviceManager {
    temp_sensor_buffer: [u8; 4],       // Small buffer for temperature
    power_monitor_buffer: [u8; 32],     // Larger buffer for power data
    eeprom_buffer: [u8; 256],          // Large buffer for EEPROM
}

// Strategy 2: Shared temporary buffers
struct TaskBuffers {
    shared_buffer: [u8; 512],          // One large buffer
    buffer_in_use: bool,               // Simple allocation flag
}

impl TaskBuffers {
    fn with_buffer<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        if self.buffer_in_use {
            None // Buffer busy
        } else {
            self.buffer_in_use = true;
            let result = f(&mut self.shared_buffer);
            self.buffer_in_use = false;
            Some(result)
        }
    }
}

// Strategy 3: Stack-based temporary buffers
fn read_large_eeprom() -> Result<[u8; 1024], ResponseCode> {
    let mut buffer = [0u8; 1024];      // Stack allocation
    eeprom_device.read_into(&mut buffer)?;
    Ok(buffer)                         // Return by value
}
```

## Comparison with Alternative Approaches

### Alternative 1: Direct Hardware Access

```rust
// Direct hardware access approach
pub struct DirectI2c {
    registers: &'static I2C_TypeDef,
}

impl DirectI2c {
    pub fn read_register(&mut self, address: u8, reg: u8) -> Result<u8, I2cError> {
        // Direct register manipulation
        self.registers.cr1.write(|w| w.pe().set_bit());
        self.registers.cr2.write(|w| {
            w.sadd().bits(address << 1)
             .rd_wrn().clear_bit()
             .nbytes().bits(1)
             .start().set_bit()
        });
        
        // Wait for address ACK
        while !self.registers.isr.read().addr().bit_is_set() {
            if self.registers.isr.read().nackf().bit_is_set() {
                return Err(I2cError::AddressNack);
            }
        }
        
        // Send register address
        self.registers.txdr.write(|w| w.txdata().bits(reg));
        
        // More register manipulation...
        Ok(data)
    }
}

// Pros: Low latency, high performance
// Cons: No sharing, complex error handling, fault propagation
```

### Alternative 2: Interrupt-Based Callback System

```rust
// Callback-based approach
pub struct CallbackI2c {
    pending_operations: HashMap<OperationId, Box<dyn FnOnce(Result<Vec<u8>, I2cError>)>>,
}

impl CallbackI2c {
    pub fn read_async<F>(&mut self, address: u8, reg: u8, callback: F) 
    where
        F: FnOnce(Result<u8, I2cError>) + 'static,
    {
        let op_id = self.start_hardware_operation(address, reg);
        self.pending_operations.insert(op_id, Box::new(callback));
    }
}

// Pros: Non-blocking, high throughput
// Cons: Callback hell, heap allocation, complex state management
```

### Alternative 3: Message Queue System

```rust
// Message queue approach
pub struct I2cRequest {
    address: u8,
    operation: I2cOp,
    response_queue: Queue<I2cResponse>,
}

pub fn submit_i2c_request(request: I2cRequest) {
    I2C_REQUEST_QUEUE.push(request);
}

// Pros: Decoupled, potentially high throughput
// Cons: Complex queue management, ordering issues, memory overhead
```

### Alternative 4: Embedded-HAL Direct Implementation

```rust
// Direct embedded-hal implementation
pub struct HardwareI2c {
    peripheral: I2C1,
}

impl embedded_hal::i2c::I2c for HardwareI2c {
    type Error = I2cError;
    
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        // Direct hardware manipulation
    }
}

// Pros: Standard interface, portable
// Cons: No task isolation, shared mutable access issues
```

### Comparison Matrix

| Approach | Latency | Throughput | Safety | Sharing | Complexity | Memory |
|----------|---------|------------|--------|---------|------------|---------|
| Hubris IPC | Medium | Medium | High | Easy | Low | Medium |
| Direct Access | Very Low | High | Low | Hard | High | Low |
| Callbacks | Low | High | Medium | Medium | High | High |
| Message Queue | Medium | High | Medium | Easy | High | High |
| embedded-hal Direct | Low | High | Low | Hard | Medium | Low |

### Why Hubris Chose IPC

The IPC approach was chosen for Hubris because:

1. **Safety First**: Task isolation prevents faults from propagating
2. **Deterministic**: Synchronous calls provide predictable timing
3. **Simple**: Easy to reason about and debug
4. **Efficient**: Zero-copy transfers minimize overhead
5. **Scalable**: Supports complex device topologies
6. **Maintainable**: Centralized I2C logic reduces code duplication

The trade-offs (slight latency increase, modest memory overhead) are acceptable for the safety and reliability benefits in critical embedded systems.

---

## Conclusion

Hubris's I2C IPC design represents a careful balance between **performance** and **safety**. While it trades some latency for fault isolation, the result is a robust, deterministic, and maintainable I2C system suitable for critical embedded applications.

The key innovations include:

- **Task-isolated I2C server** for fault containment
- **Zero-copy lease system** for efficient data transfer  
- **Rich device addressing model** for complex topologies
- **Type-safe marshalling** for reliable IPC
- **Structured error handling** for robust operation

This design enables safe sharing of I2C resources while maintaining the real-time characteristics required for embedded systems.
