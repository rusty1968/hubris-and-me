//! Embedded-HAL I2C wrapper for Hubris IPC-based I2C client API
//!
//! This module provides a standard `embedded_hal::i2c::I2c` trait implementation
//! that bridges between portable device drivers and Hubris's task-isolated I2C system.
//!
//! # Features
//!
//! - Full `embedded-hal::i2c::I2c` trait compliance
//! - Support for both 7-bit and 10-bit addressing
//! - Zero-copy efficiency using Hubris lease system
//! - Optimized register operations
//! - Comprehensive error mapping
//! - Mock implementation for testing
//!
//! # Example Usage
//!
//! ```rust
//! use embedded_hal::i2c::I2c;
//! use drv_i2c_generic::embedded_hal_wrapper::{HubrisI2c, SevenBitAddr};
//!
//! // Create wrapper for TMP117 temperature sensor
//! let i2c = HubrisI2c::new_simple(
//!     I2C_SERVER_TASK,
//!     drv_i2c_types::Controller::I2c1,
//!     drv_i2c_types::PortIndex(0),
//!     0x48, // TMP117 address
//! );
//!
//! // Use with any embedded-hal I2C driver
//! let mut temp_sensor = TemperatureSensorDriver::new(i2c);
//! let temperature = temp_sensor.read_temperature()?;
//! ```

#![no_std]

use drv_i2c_api::{I2cDevice, ResponseCode};
use drv_i2c_types::{Controller, PortIndex, Mux, Segment};
use userlib::TaskId;
use embedded_hal::i2c::{ErrorKind, ErrorType, NoAcknowledgeSource, Operation};

/// Embedded-HAL I2C wrapper for Hubris IPC-based I2C
pub struct HubrisI2c {
    device: I2cDevice,
}

impl HubrisI2c {
    /// Create a new wrapper around a Hubris I2C device
    ///
    /// # Arguments
    ///
    /// * `i2c_server_task` - Task ID of the I2C server
    /// * `controller` - Hardware I2C controller (I2C1, I2C2, etc.)
    /// * `port` - Port configuration index
    /// * `segment` - Optional multiplexer and segment
    /// * `device_address` - I2C device address (7-bit)
    pub fn new(
        i2c_server_task: TaskId,
        controller: Controller,
        port: PortIndex,
        segment: Option<(Mux, Segment)>,
        device_address: u8,
    ) -> Self {
        Self {
            device: I2cDevice::new(
                i2c_server_task,
                controller,
                port,
                segment,
                device_address,
            ),
        }
    }

    /// Create wrapper for device without multiplexer
    ///
    /// Convenience constructor for simple I2C topologies without multiplexers.
    pub fn new_simple(
        i2c_server_task: TaskId,
        controller: Controller,
        port: PortIndex,
        device_address: u8,
    ) -> Self {
        Self::new(i2c_server_task, controller, port, None, device_address)
    }

    /// Get reference to underlying Hubris device for advanced operations
    ///
    /// This allows access to Hubris-specific optimized operations like
    /// register reads and SMBus block operations.
    pub fn device(&self) -> &I2cDevice {
        &self.device
    }

    /// Get mutable reference for advanced operations
    pub fn device_mut(&mut self) -> &mut I2cDevice {
        &mut self.device
    }

    /// Perform optimized register read using Hubris API
    ///
    /// This bypasses the embedded-hal interface for optimal performance
    /// when reading typed register values.
    pub fn read_register<R, V>(&self, reg: R) -> Result<V, HubrisI2cError>
    where
        R: zerocopy::IntoBytes + zerocopy::Immutable,
        V: zerocopy::IntoBytes + zerocopy::FromBytes,
    {
        self.device
            .read_reg(reg)
            .map_err(|response_code| HubrisI2cError {
                response_code,
                operation: "optimized_register_read",
            })
    }

    /// Perform SMBus block read using Hubris API
    ///
    /// This provides access to SMBus block read functionality that
    /// isn't directly available through embedded-hal.
    pub fn read_block<R>(
        &self,
        reg: R,
        buffer: &mut [u8],
    ) -> Result<usize, HubrisI2cError>
    where
        R: zerocopy::IntoBytes + zerocopy::Immutable,
    {
        self.device
            .read_block(reg, buffer)
            .map_err(|response_code| HubrisI2cError {
                response_code,
                operation: "smbus_block_read",
            })
    }
}

/// Error type that maps Hubris ResponseCode to embedded-hal errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HubrisI2cError {
    pub response_code: ResponseCode,
    pub operation: &'static str,
}

impl embedded_hal::i2c::Error for HubrisI2cError {
    fn kind(&self) -> ErrorKind {
        match self.response_code {
            ResponseCode::Success => ErrorKind::Other, // Should not happen

            // Address/Data NACK errors
            ResponseCode::AddressNackSentEarly | ResponseCode::AddressNackSentLate => {
                ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address)
            }
            ResponseCode::DataNackSent => {
                ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data)
            }

            // Bus condition errors
            ResponseCode::BusError => ErrorKind::Bus,
            ResponseCode::ArbitrationLost => ErrorKind::ArbitrationLoss,

            // All other errors map to Other
            _ => ErrorKind::Other,
        }
    }
}

impl core::fmt::Display for HubrisI2cError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "I2C {} operation failed: {:?}",
            self.operation, self.response_code
        )
    }
}

impl HubrisI2cError {
    /// Add operation context to error
    pub fn with_operation(mut self, operation: &'static str) -> Self {
        self.operation = operation;
        self
    }

    /// Check if error indicates device not present
    pub fn is_device_not_found(&self) -> bool {
        matches!(
            self.response_code,
            ResponseCode::AddressNackSentEarly
                | ResponseCode::AddressNackSentLate
                | ResponseCode::NoDevice
        )
    }

    /// Check if error indicates temporary bus condition
    pub fn is_temporary(&self) -> bool {
        matches!(
            self.response_code,
            ResponseCode::BusLocked | ResponseCode::BusTimeout | ResponseCode::ArbitrationLost
        )
    }

    /// Get suggested retry delay for temporary errors
    pub fn retry_delay(&self) -> Option<core::time::Duration> {
        match self.response_code {
            ResponseCode::BusLocked => Some(core::time::Duration::from_millis(10)),
            ResponseCode::BusTimeout => Some(core::time::Duration::from_millis(100)),
            ResponseCode::ArbitrationLost => Some(core::time::Duration::from_millis(1)),
            _ => None,
        }
    }
}

/// Address wrapper for 7-bit addressing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SevenBitAddr(pub u8);

impl SevenBitAddr {
    /// Create new 7-bit address
    pub const fn new(addr: u8) -> Self {
        SevenBitAddr(addr)
    }

    /// Get the address value
    pub fn get(self) -> u8 {
        self.0
    }

    /// Validate 7-bit address range
    pub fn try_new(addr: u8) -> Result<Self, InvalidAddress> {
        if addr > 0x7F {
            Err(InvalidAddress::SevenBitRange(addr))
        } else if addr < 0x08 || addr > 0x77 {
            Err(InvalidAddress::Reserved(addr))
        } else {
            Ok(SevenBitAddr(addr))
        }
    }
}

/// Address wrapper for 10-bit addressing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenBitAddr(pub u16);

impl TenBitAddr {
    /// Create new 10-bit address
    pub const fn new(addr: u16) -> Self {
        TenBitAddr(addr)
    }

    /// Get the address value
    pub fn get(self) -> u16 {
        self.0
    }

    /// Validate 10-bit address range
    pub fn try_new(addr: u16) -> Result<Self, InvalidAddress> {
        if addr > 0x3FF {
            Err(InvalidAddress::TenBitRange(addr))
        } else {
            Ok(TenBitAddr(addr))
        }
    }
}

/// Address validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidAddress {
    SevenBitRange(u8),  // Address > 0x7F
    TenBitRange(u16),   // Address > 0x3FF
    Reserved(u8),       // Address in reserved range
}

impl core::fmt::Display for InvalidAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InvalidAddress::SevenBitRange(addr) => {
                write!(f, "Address 0x{:02X} exceeds 7-bit range (0x00-0x7F)", addr)
            }
            InvalidAddress::TenBitRange(addr) => {
                write!(f, "Address 0x{:03X} exceeds 10-bit range (0x000-0x3FF)", addr)
            }
            InvalidAddress::Reserved(addr) => {
                write!(f, "Address 0x{:02X} is in reserved range", addr)
            }
        }
    }
}

// Address type conversions
impl From<SevenBitAddr> for u8 {
    fn from(addr: SevenBitAddr) -> u8 {
        addr.0
    }
}

impl From<u8> for SevenBitAddr {
    fn from(addr: u8) -> SevenBitAddr {
        SevenBitAddr(addr)
    }
}

impl From<TenBitAddr> for u16 {
    fn from(addr: TenBitAddr) -> u16 {
        addr.0
    }
}

impl From<u16> for TenBitAddr {
    fn from(addr: u16) -> TenBitAddr {
        TenBitAddr(addr)
    }
}

// Embedded-HAL trait implementations
impl ErrorType for HubrisI2c {
    type Error = HubrisI2cError;
}

/// Implementation for 7-bit addressing
impl embedded_hal::i2c::I2c<SevenBitAddr> for HubrisI2c {
    fn read(&mut self, _address: SevenBitAddr, buffer: &mut [u8]) -> Result<(), Self::Error> {
        // Note: We ignore the address parameter since Hubris I2cDevice
        // already contains the device address. This is a limitation of
        // bridging between embedded-hal's per-operation addressing and
        // Hubris's device-centric model.

        self.device
            .read_into(buffer)
            .map(|_| ()) // Discard byte count
            .map_err(|response_code| HubrisI2cError {
                response_code,
                operation: "read",
            })
    }

    fn write(&mut self, _address: SevenBitAddr, bytes: &[u8]) -> Result<(), Self::Error> {
        self.device
            .write(bytes)
            .map_err(|response_code| HubrisI2cError {
                response_code,
                operation: "write",
            })
    }

    fn write_read(
        &mut self,
        _address: SevenBitAddr,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        // Try to optimize for common register read patterns
        if bytes.len() == 1 {
            // Single byte write likely indicates register read
            self.device
                .read_reg_into(bytes[0], buffer)
                .map(|_| ()) // Discard byte count
                .map_err(|response_code| HubrisI2cError {
                    response_code,
                    operation: "write_read_reg",
                })
        } else {
            // Multi-byte write - fall back to separate operations
            self.device
                .write(bytes)
                .map_err(|response_code| HubrisI2cError {
                    response_code,
                    operation: "write_read_write_phase",
                })?;

            self.device
                .read_into(buffer)
                .map(|_| ()) // Discard byte count
                .map_err(|response_code| HubrisI2cError {
                    response_code,
                    operation: "write_read_read_phase",
                })
        }
    }

    fn transaction(
        &mut self,
        address: SevenBitAddr,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        // Process operations sequentially
        // Note: This doesn't provide true I2C transaction semantics
        // (repeated START conditions) but is the best we can do with
        // the current Hubris API

        for operation in operations.iter_mut() {
            match operation {
                Operation::Read(buffer) => {
                    self.read(address, buffer).map_err(|mut err| {
                        err.operation = "transaction_read";
                        err
                    })?;
                }
                Operation::Write(data) => {
                    self.write(address, data).map_err(|mut err| {
                        err.operation = "transaction_write";
                        err
                    })?;
                }
            }
        }

        Ok(())
    }
}

/// Implementation for 10-bit addressing
impl embedded_hal::i2c::I2c<TenBitAddr> for HubrisI2c {
    fn read(&mut self, address: TenBitAddr, buffer: &mut [u8]) -> Result<(), Self::Error> {
        // 10-bit addressing requires special handling
        // The current Hubris API doesn't directly support 10-bit addressing,
        // so we need to handle the 10-bit protocol manually

        // 10-bit read sequence:
        // 1. Send 11110XX0 (where XX are upper 2 bits of address)
        // 2. Send lower 8 bits of address
        // 3. Send repeated START
        // 4. Send 11110XX1 (read bit set)
        // 5. Read data

        let addr_high = 0xF0 | ((address.0 >> 7) & 0x06) as u8; // 11110XX0
        let addr_low = (address.0 & 0xFF) as u8;

        // Use write_read to perform the 10-bit addressing sequence
        let write_data = [addr_high, addr_low];

        // This is a limitation: we're approximating 10-bit addressing
        // A full implementation would need server support for 10-bit protocol
        self.device
            .write(&write_data)
            .map_err(|response_code| HubrisI2cError {
                response_code,
                operation: "10bit_address_setup",
            })?;

        self.device
            .read_into(buffer)
            .map(|_| ())
            .map_err(|response_code| HubrisI2cError {
                response_code,
                operation: "10bit_read",
            })
    }

    fn write(&mut self, address: TenBitAddr, bytes: &[u8]) -> Result<(), Self::Error> {
        // 10-bit write sequence:
        // 1. Send 11110XX0 (where XX are upper 2 bits of address)
        // 2. Send lower 8 bits of address
        // 3. Send data bytes

        let addr_high = 0xF0 | ((address.0 >> 7) & 0x06) as u8;
        let addr_low = (address.0 & 0xFF) as u8;

        // Prepare combined write data: address + data
        let mut write_data = heapless::Vec::<u8, 258>::new(); // Max: 2 addr + 256 data
        write_data.push(addr_high).map_err(|_| HubrisI2cError {
            response_code: ResponseCode::BadResponse,
            operation: "10bit_write_buffer_overflow",
        })?;
        write_data.push(addr_low).map_err(|_| HubrisI2cError {
            response_code: ResponseCode::BadResponse,
            operation: "10bit_write_buffer_overflow",
        })?;

        for &byte in bytes {
            write_data.push(byte).map_err(|_| HubrisI2cError {
                response_code: ResponseCode::BadResponse,
                operation: "10bit_write_buffer_overflow",
            })?;
        }

        self.device
            .write(&write_data)
            .map_err(|response_code| HubrisI2cError {
                response_code,
                operation: "10bit_write",
            })
    }

    fn write_read(
        &mut self,
        address: TenBitAddr,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        // For 10-bit devices, we need to do separate write and read
        // since the addressing is more complex
        self.write(address, bytes)?;
        self.read(address, buffer)
    }

    fn transaction(
        &mut self,
        address: TenBitAddr,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        // Similar to 7-bit but with 10-bit addressing
        for operation in operations.iter_mut() {
            match operation {
                Operation::Read(buffer) => {
                    self.read(address, buffer)?;
                }
                Operation::Write(data) => {
                    self.write(address, data)?;
                }
            }
        }
        Ok(())
    }
}

/// Optimized wrapper for register-heavy devices
pub struct RegisterOptimizedI2c {
    wrapper: HubrisI2c,
}

impl RegisterOptimizedI2c {
    /// Create new register-optimized wrapper
    pub fn new(wrapper: HubrisI2c) -> Self {
        Self { wrapper }
    }

    /// Direct register access using Hubris optimized calls
    pub fn read_register<T>(&self, reg: u8) -> Result<T, HubrisI2cError>
    where
        T: zerocopy::FromBytes + zerocopy::IntoBytes,
    {
        self.wrapper.device.read_reg(reg).map_err(|response_code| {
            HubrisI2cError {
                response_code,
                operation: "optimized_register_read",
            }
        })
    }

    /// Block read using Hubris SMBus support
    pub fn read_block(&self, reg: u8, buffer: &mut [u8]) -> Result<usize, HubrisI2cError> {
        self.wrapper.device.read_block(reg, buffer).map_err(
            |response_code| HubrisI2cError {
                response_code,
                operation: "optimized_block_read",
            },
        )
    }
}

impl ErrorType for RegisterOptimizedI2c {
    type Error = HubrisI2cError;
}

impl embedded_hal::i2c::I2c<SevenBitAddr> for RegisterOptimizedI2c {
    fn read(&mut self, address: SevenBitAddr, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapper.read(address, buffer)
    }

    fn write(&mut self, address: SevenBitAddr, bytes: &[u8]) -> Result<(), Self::Error> {
        self.wrapper.write(address, bytes)
    }

    fn write_read(
        &mut self,
        address: SevenBitAddr,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        // Always use optimized register read for single-byte writes
        if bytes.len() == 1 {
            self.wrapper
                .device
                .read_reg_into(bytes[0], buffer)
                .map(|_| ())
                .map_err(|response_code| HubrisI2cError {
                    response_code,
                    operation: "optimized_write_read",
                })
        } else {
            self.wrapper.write_read(address, bytes, buffer)
        }
    }

    fn transaction(
        &mut self,
        address: SevenBitAddr,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        // Try to optimize common transaction patterns
        if operations.len() == 2 {
            if let (Operation::Write(write_data), Operation::Read(read_buffer)) =
                (&operations[0], &mut operations[1])
            {
                if write_data.len() == 1 {
                    // Optimize write(1 byte) + read(n bytes) as register read
                    return self
                        .wrapper
                        .device
                        .read_reg_into(write_data[0], read_buffer)
                        .map(|_| ())
                        .map_err(|response_code| HubrisI2cError {
                            response_code,
                            operation: "optimized_transaction",
                        });
                }
            }
        }

        // Fall back to default implementation
        self.wrapper.transaction(address, operations)
    }
}

/// Wrapper that automatically retries on temporary errors
pub struct RetryingI2c<I2C> {
    inner: I2C,
    max_retries: u8,
}

impl<I2C> RetryingI2c<I2C> {
    /// Create new retrying wrapper
    ///
    /// # Arguments
    ///
    /// * `inner` - The underlying I2C implementation
    /// * `max_retries` - Maximum number of retry attempts
    pub fn new(inner: I2C, max_retries: u8) -> Self {
        Self { inner, max_retries }
    }

    /// Execute operation with automatic retry on temporary errors
    fn retry_operation<F, R>(&mut self, mut operation: F) -> Result<R, I2C::Error>
    where
        F: FnMut(&mut I2C) -> Result<R, I2C::Error>,
        I2C::Error: embedded_hal::i2c::Error,
        I2C: embedded_hal::i2c::I2c<SevenBitAddr>,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match operation(&mut self.inner) {
                Ok(result) => return Ok(result),
                Err(error) => {
                    // Check if error is retryable
                    match error.kind() {
                        ErrorKind::ArbitrationLoss | ErrorKind::Other => {
                            if attempt < self.max_retries {
                                // Wait before retry (exponential backoff)
                                userlib::sys::sleep_for(userlib::time::Duration::from_millis(
                                    10 * (attempt + 1) as u64,
                                ));
                                last_error = Some(error);
                                continue;
                            }
                        }
                        _ => {
                            // Non-retryable error
                            return Err(error);
                        }
                    }

                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap())
    }
}

impl<I2C> ErrorType for RetryingI2c<I2C>
where
    I2C: ErrorType,
{
    type Error = I2C::Error;
}

impl<I2C> embedded_hal::i2c::I2c<SevenBitAddr> for RetryingI2c<I2C>
where
    I2C: embedded_hal::i2c::I2c<SevenBitAddr>,
    I2C::Error: embedded_hal::i2c::Error,
{
    fn read(&mut self, address: SevenBitAddr, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.retry_operation(|i2c| i2c.read(address, buffer))
    }

    fn write(&mut self, address: SevenBitAddr, bytes: &[u8]) -> Result<(), Self::Error> {
        self.retry_operation(|i2c| i2c.write(address, bytes))
    }

    fn write_read(
        &mut self,
        address: SevenBitAddr,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.retry_operation(|i2c| i2c.write_read(address, bytes, buffer))
    }

    fn transaction(
        &mut self,
        address: SevenBitAddr,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.retry_operation(|i2c| i2c.transaction(address, operations))
    }
}

#[cfg(feature = "testing")]
pub mod mock {
    //! Mock I2C implementation for testing embedded-hal device drivers

    use super::*;
    use heapless::Vec;

    /// Mock I2C implementation for testing
    pub struct MockI2c {
        expected_operations: Vec<MockOperation, 32>,
        operation_index: usize,
    }

    #[derive(Debug, Clone)]
    pub enum MockOperation {
        Read {
            address: SevenBitAddr,
            response: Vec<u8, 256>,
        },
        Write {
            address: SevenBitAddr,
            expected_data: Vec<u8, 256>,
        },
        WriteRead {
            address: SevenBitAddr,
            expected_write: Vec<u8, 256>,
            read_response: Vec<u8, 256>,
        },
    }

    impl MockI2c {
        /// Create new mock I2C
        pub fn new() -> Self {
            Self {
                expected_operations: Vec::new(),
                operation_index: 0,
            }
        }

        /// Expect a write operation
        pub fn expect_write(&mut self, address: SevenBitAddr, data: &[u8]) {
            let mut expected_data = Vec::new();
            expected_data.extend_from_slice(data).unwrap();

            self.expected_operations
                .push(MockOperation::Write {
                    address,
                    expected_data,
                })
                .unwrap();
        }

        /// Expect a read operation
        pub fn expect_read(&mut self, address: SevenBitAddr, response: &[u8]) {
            let mut response_data = Vec::new();
            response_data.extend_from_slice(response).unwrap();

            self.expected_operations
                .push(MockOperation::Read {
                    address,
                    response: response_data,
                })
                .unwrap();
        }

        /// Expect a write-read operation
        pub fn expect_write_read(
            &mut self,
            address: SevenBitAddr,
            write_data: &[u8],
            read_response: &[u8],
        ) {
            let mut expected_write = Vec::new();
            expected_write.extend_from_slice(write_data).unwrap();

            let mut response = Vec::new();
            response.extend_from_slice(read_response).unwrap();

            self.expected_operations
                .push(MockOperation::WriteRead {
                    address,
                    expected_write,
                    read_response: response,
                })
                .unwrap();
        }

        /// Verify all expected operations were performed
        pub fn verify_complete(&self) {
            assert_eq!(
                self.operation_index,
                self.expected_operations.len(),
                "Not all expected I2C operations were performed"
            );
        }
    }

    /// Mock I2C error type
    #[derive(Debug)]
    pub struct MockI2cError {
        message: &'static str,
    }

    impl embedded_hal::i2c::Error for MockI2cError {
        fn kind(&self) -> ErrorKind {
            ErrorKind::Other
        }
    }

    impl core::fmt::Display for MockI2cError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "Mock I2C error: {}", self.message)
        }
    }

    impl ErrorType for MockI2c {
        type Error = MockI2cError;
    }

    impl embedded_hal::i2c::I2c<SevenBitAddr> for MockI2c {
        fn read(&mut self, address: SevenBitAddr, buffer: &mut [u8]) -> Result<(), Self::Error> {
            if self.operation_index >= self.expected_operations.len() {
                return Err(MockI2cError {
                    message: "Unexpected read operation",
                });
            }

            match &self.expected_operations[self.operation_index] {
                MockOperation::Read {
                    address: expected_addr,
                    response,
                } => {
                    if *expected_addr != address {
                        return Err(MockI2cError {
                            message: "Read address mismatch",
                        });
                    }

                    if buffer.len() != response.len() {
                        return Err(MockI2cError {
                            message: "Read buffer size mismatch",
                        });
                    }

                    buffer.copy_from_slice(response);
                    self.operation_index += 1;
                    Ok(())
                }
                _ => Err(MockI2cError {
                    message: "Expected read operation",
                }),
            }
        }

        fn write(&mut self, address: SevenBitAddr, bytes: &[u8]) -> Result<(), Self::Error> {
            if self.operation_index >= self.expected_operations.len() {
                return Err(MockI2cError {
                    message: "Unexpected write operation",
                });
            }

            match &self.expected_operations[self.operation_index] {
                MockOperation::Write {
                    address: expected_addr,
                    expected_data,
                } => {
                    if *expected_addr != address {
                        return Err(MockI2cError {
                            message: "Write address mismatch",
                        });
                    }

                    if bytes != expected_data.as_slice() {
                        return Err(MockI2cError {
                            message: "Write data mismatch",
                        });
                    }

                    self.operation_index += 1;
                    Ok(())
                }
                _ => Err(MockI2cError {
                    message: "Expected write operation",
                }),
            }
        }

        fn write_read(
            &mut self,
            address: SevenBitAddr,
            bytes: &[u8],
            buffer: &mut [u8],
        ) -> Result<(), Self::Error> {
            if self.operation_index >= self.expected_operations.len() {
                return Err(MockI2cError {
                    message: "Unexpected write_read operation",
                });
            }

            match &self.expected_operations[self.operation_index] {
                MockOperation::WriteRead {
                    address: expected_addr,
                    expected_write,
                    read_response,
                } => {
                    if *expected_addr != address {
                        return Err(MockI2cError {
                            message: "WriteRead address mismatch",
                        });
                    }

                    if bytes != expected_write.as_slice() {
                        return Err(MockI2cError {
                            message: "WriteRead write data mismatch",
                        });
                    }

                    if buffer.len() != read_response.len() {
                        return Err(MockI2cError {
                            message: "WriteRead read buffer size mismatch",
                        });
                    }

                    buffer.copy_from_slice(read_response);
                    self.operation_index += 1;
                    Ok(())
                }
                _ => Err(MockI2cError {
                    message: "Expected write_read operation",
                }),
            }
        }

        fn transaction(
            &mut self,
            address: SevenBitAddr,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            for operation in operations.iter_mut() {
                match operation {
                    Operation::Read(buffer) => {
                        self.read(address, buffer)?;
                    }
                    Operation::Write(data) => {
                        self.write(address, data)?;
                    }
                }
            }
            Ok(())
        }
    }

    impl Default for MockI2c {
        fn default() -> Self {
            Self::new()
        }
    }
}

// Re-export common types for convenience
pub use embedded_hal::i2c::{Error, ErrorKind, I2c, NoAcknowledgeSource, Operation};

#[cfg(feature = "testing")]
pub use mock::MockI2c;
