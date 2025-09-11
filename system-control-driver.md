# Hubris System Control Driver Architecture

**Project:** Hubris Operating System  
**Component:** System Control Driver Architecture  
**Version:** 1.0  
**Date:** September 8, 2025  

## Overview

This document illustrates the architecture of a Hubris system control driver, responsible for managing system-level functions including clock control, power management, reset sequencing, and hardware configuration. The system control driver serves as the central authority for low-level system operations.

## System Control Architecture Block Diagram

```mermaid
graph TB
    subgraph "Application Layer"
        KERNEL[Kernel Task]
        POWER[Power Manager Task]
        THERMAL[Thermal Manager Task]
        CLOCK[Clock Manager Task]
        BOOT[Boot Sequencer Task]
        DEBUG[Debug/Test Tasks]
    end

    subgraph "System API Layer"
        SYSAPI[drv-stm32xx-sys-api<br/>• Direct Function Calls<br/>• GPIO Control Functions<br/>• Clock Control Functions<br/>• Power Management Functions<br/>• Hardware Abstraction]
    end

    subgraph "System Control Server Layer"
        SYSCTRL[sys-control-server<br/>• Clock Tree Management<br/>• Power Domain Control<br/>• Reset Sequencing<br/>• Pin Multiplexing<br/>• Interrupt Routing<br/>• System State Management<br/>• Hardware Abstraction]
    end

    subgraph "Hardware Driver Layer"
        CLKDRV[clock-driver<br/>• PLL Configuration<br/>• Clock Gating<br/>• Frequency Scaling<br/>• Clock Tree Setup]
        PWRDRV[power-driver<br/>• Voltage Regulation<br/>• Power Domain Control<br/>• Sleep/Wake Management<br/>• Power Sequencing]
        RSTDRV[reset-driver<br/>• Reset Generation<br/>• Reset Sequencing<br/>• Reset Status<br/>• Watchdog Control]
        GPIODRV[gpio-driver<br/>• Pin Configuration<br/>• Alternate Functions<br/>• Pin Muxing<br/>• Electrical Config]
    end

    subgraph "Register Abstraction Layer"
        RCC[RCC Registers<br/>• Clock Control<br/>• Reset Control<br/>• PLL Configuration]
        PWR[PWR Registers<br/>• Power Control<br/>• Voltage Scaling<br/>• Sleep Modes]
        SYSCFG[SYSCFG Registers<br/>• System Configuration<br/>• Pin Remapping<br/>• Interrupt Config]
        GPIO[GPIO Registers<br/>• Pin Control<br/>• Alternate Function<br/>• Pull-up/Pull-down]
    end

    subgraph "Hardware Layer"
        STM32SYS[STM32 System Hardware<br/>• Clock Generation Unit<br/>• Power Management Unit<br/>• Reset and Clock Control<br/>• GPIO Banks<br/>• System Configuration]
    end

    subgraph "External Components"
        CRYSTAL[External Crystals<br/>• HSE Crystal<br/>• LSE Crystal<br/>• Backup Domain]
        POWER_IC[Power Management ICs<br/>• Voltage Regulators<br/>• Power Supervisors<br/>• Battery Backup]
        RESET_IC[Reset Controllers<br/>• Watchdog ICs<br/>• Power-on Reset<br/>• Brown-out Detection]
    end

    %% Application to API connections
    KERNEL --> SYSAPI
    POWER --> SYSAPI
    THERMAL --> SYSAPI
    CLOCK --> SYSAPI
    BOOT --> SYSAPI
    DEBUG --> SYSAPI

    %% API to Server connection
    SYSAPI --> SYSCTRL

    %% Server to Driver connections
    SYSCTRL --> CLKDRV
    SYSCTRL --> PWRDRV
    SYSCTRL --> RSTDRV
    SYSCTRL --> GPIODRV

    %% Driver to Register connections
    CLKDRV --> RCC
    PWRDRV --> PWR
    RSTDRV --> RCC
    GPIODRV --> GPIO
    SYSCTRL --> SYSCFG

    %% Register to Hardware connections
    RCC --> STM32SYS
    PWR --> STM32SYS
    SYSCFG --> STM32SYS
    GPIO --> STM32SYS

    %% Hardware to External connections
    STM32SYS --> CRYSTAL
    STM32SYS --> POWER_IC
    STM32SYS --> RESET_IC

    %% Styling
    classDef appLayer fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef apiLayer fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef serverLayer fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef driverLayer fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef registerLayer fill:#fce4ec,stroke:#880e4f,stroke-width:2px
    classDef hardwareLayer fill:#f1f8e9,stroke:#33691e,stroke-width:2px
    classDef externalLayer fill:#e0f2f1,stroke:#004d40,stroke-width:2px

    class KERNEL,POWER,THERMAL,CLOCK,BOOT,DEBUG appLayer
    class SYSAPI apiLayer
    class SYSCTRL serverLayer
    class CLKDRV,PWRDRV,RSTDRV,GPIODRV driverLayer
    class RCC,PWR,SYSCFG,GPIO registerLayer
    class STM32SYS hardwareLayer
    class CRYSTAL,POWER_IC,RESET_IC externalLayer
```

## Clock Management Subsystem

```mermaid
graph TB
    subgraph "Clock Tree Management"
        subgraph "Clock Sources"
            HSI[HSI<br/>Internal RC<br/>16 MHz]
            HSE[HSE<br/>External Crystal<br/>8-25 MHz]
            LSI[LSI<br/>Internal RC<br/>32 kHz]
            LSE[LSE<br/>External Crystal<br/>32.768 kHz]
        end
        
        subgraph "PLL Systems"
            PLL1[PLL1<br/>Main System PLL<br/>Up to 400 MHz]
            PLL2[PLL2<br/>Peripheral PLL<br/>Audio/Video]
            PLL3[PLL3<br/>Peripheral PLL<br/>USB/Ethernet]
        end
        
        subgraph "System Clocks"
            SYSCLK[SYSCLK<br/>System Clock<br/>CPU Core]
            HCLK[HCLK<br/>AHB Clock<br/>Bus Matrix]
            PCLK1[PCLK1<br/>APB1 Clock<br/>Low Speed Peripherals]
            PCLK2[PCLK2<br/>APB2 Clock<br/>High Speed Peripherals]
        end
        
        subgraph "Peripheral Clocks"
            I2CCLK[I2C Clocks]
            SPICLK[SPI Clocks]
            UARTCLK[UART Clocks]
            ADCCLK[ADC Clocks]
            TIMCLK[Timer Clocks]
        end
    end

    %% Clock source connections
    HSI --> PLL1
    HSE --> PLL1
    HSE --> PLL2
    HSE --> PLL3
    LSI --> SYSCLK
    LSE --> SYSCLK

    %% PLL to system clocks
    PLL1 --> SYSCLK
    SYSCLK --> HCLK
    HCLK --> PCLK1
    HCLK --> PCLK2

    %% System to peripheral clocks
    PCLK1 --> I2CCLK
    PCLK1 --> UARTCLK
    PCLK2 --> SPICLK
    PLL2 --> ADCCLK
    PCLK1 --> TIMCLK

    classDef clockSource fill:#e3f2fd,stroke:#1565c0,stroke-width:2px
    classDef pllSystem fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    classDef sysClock fill:#e8f5e8,stroke:#388e3c,stroke-width:2px
    classDef periClock fill:#fce4ec,stroke:#ad1457,stroke-width:2px

    class HSI,HSE,LSI,LSE clockSource
    class PLL1,PLL2,PLL3 pllSystem
    class SYSCLK,HCLK,PCLK1,PCLK2 sysClock
    class I2CCLK,SPICLK,UARTCLK,ADCCLK,TIMCLK periClock
```

## Power Management Subsystem

```mermaid
graph TB
    subgraph "Power Management Architecture"
        subgraph "Power Domains"
            VDD[VDD<br/>Main Digital Supply<br/>1.8V/3.3V]
            VDDA[VDDA<br/>Analog Supply<br/>ADC/DAC]
            VDDIO[VDDIO<br/>I/O Supply<br/>GPIO Banks]
            VBAT[VBAT<br/>Backup Domain<br/>RTC/LSE]
        end
        
        subgraph "Power States"
            RUN[Run Mode<br/>Full Operation<br/>All Clocks Active]
            SLEEP[Sleep Mode<br/>CPU Stopped<br/>Peripherals Active]
            STOP[Stop Mode<br/>Most Clocks Off<br/>RAM Retained]
            STANDBY[Standby Mode<br/>Minimal Power<br/>RAM Lost]
        end
        
        subgraph "Voltage Scaling"
            SCALE0[Scale 0<br/>Highest Performance<br/>1.3V Core]
            SCALE1[Scale 1<br/>High Performance<br/>1.2V Core]
            SCALE2[Scale 2<br/>Balanced<br/>1.1V Core]
            SCALE3[Scale 3<br/>Low Power<br/>1.0V Core]
        end
        
        subgraph "Wake Sources"
            EXTI[External Interrupts]
            RTC_WAKE[RTC Alarms]
            WDG_WAKE[Watchdog Reset]
            PIN_WAKE[Wakeup Pins]
        end
    end

    %% Power domain relationships
    VDD --> RUN
    VDD --> SLEEP
    VDD --> STOP
    VBAT --> STANDBY

    %% Power state transitions
    RUN --> SLEEP
    SLEEP --> STOP
    STOP --> STANDBY
    
    %% Wake source connections
    EXTI --> RUN
    RTC_WAKE --> RUN
    WDG_WAKE --> RUN
    PIN_WAKE --> RUN

    %% Voltage scaling in run mode
    RUN --> SCALE0
    RUN --> SCALE1
    RUN --> SCALE2
    RUN --> SCALE3

    classDef powerDomain fill:#ffebee,stroke:#c62828,stroke-width:2px
    classDef powerState fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef voltageScale fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    classDef wakeSource fill:#e1f5fe,stroke:#0277bd,stroke-width:2px

    class VDD,VDDA,VDDIO,VBAT powerDomain
    class RUN,SLEEP,STOP,STANDBY powerState
    class SCALE0,SCALE1,SCALE2,SCALE3 voltageScale
    class EXTI,RTC_WAKE,WDG_WAKE,PIN_WAKE wakeSource
```

## System Control Data Flow

```mermaid
sequenceDiagram
    participant App as Application Task
    participant API as drv-stm32xx-sys-api
    participant Server as sys-control-server
    participant ClkDrv as clock-driver
    participant PwrDrv as power-driver
    participant HW as STM32 Hardware

    Note over App,HW: System Clock Configuration Flow

    App->>API: Configure system clock
    API->>Server: ClockConfigRequest
    
    Note over Server: Validate clock configuration
    
    Server->>ClkDrv: Configure PLL
    ClkDrv->>HW: Write RCC registers
    
    Server->>ClkDrv: Switch system clock
    ClkDrv->>HW: Update SYSCLK source
    
    Server->>PwrDrv: Adjust voltage scaling
    PwrDrv->>HW: Write PWR registers
    
    Note over HW: Clock stabilization delay
    
    HW-->>ClkDrv: Clock ready interrupt
    ClkDrv-->>Server: Configuration complete
    Server-->>API: Success response
    API-->>App: Clock configured

    Note over App,HW: Power Mode Transition Flow

    App->>API: Enter low power mode
    API->>Server: PowerModeRequest
    
    Server->>ClkDrv: Disable unused clocks
    Server->>PwrDrv: Configure wake sources
    Server->>PwrDrv: Enter sleep mode
    
    PwrDrv->>HW: Write PWR control registers
    
    Note over HW: System enters sleep mode
    Note over HW: Wake event occurs
    
    HW-->>PwrDrv: Wake interrupt
    PwrDrv-->>Server: System wake
    Server->>ClkDrv: Restore clocks
    Server-->>API: Wake complete
    API-->>App: System active
```

## System Control Crate Structure

```mermaid
graph TB
    subgraph "Crate Organization"
        subgraph "API Crate"
            SYSAPI_CRATE[drv-stm32xx-sys-api<br/>• System call wrappers<br/>• Hardware abstractions<br/>• Type definitions<br/>• Direct function calls<br/>• No IPC messages]
        end
        
        subgraph "Server Crate"
            SERVER_CRATE[sys-control-server<br/>• main() function<br/>• IPC handling<br/>• State management<br/>• Coordination logic<br/>• Policy enforcement]
        end
        
        subgraph "Driver Crates"
            CLK_CRATE[clock-driver<br/>• PLL management<br/>• Clock tree control<br/>• Frequency calculation<br/>• Clock gating]
            PWR_CRATE[power-driver<br/>• Voltage scaling<br/>• Power mode control<br/>• Wake source config<br/>• Power sequencing]
            GPIO_CRATE[gpio-driver<br/>• Pin configuration<br/>• Alternate functions<br/>• Pin muxing<br/>• Electrical settings]
        end
        
        subgraph "HAL Crate"
            HAL_CRATE[stm32-hal<br/>• Register definitions<br/>• Register access<br/>• Hardware abstraction<br/>• Device specifics]
        end
    end

    SYSAPI_CRATE --> SERVER_CRATE
    SERVER_CRATE --> CLK_CRATE
    SERVER_CRATE --> PWR_CRATE
    SERVER_CRATE --> GPIO_CRATE
    CLK_CRATE --> HAL_CRATE
    PWR_CRATE --> HAL_CRATE
    GPIO_CRATE --> HAL_CRATE

    classDef apiCrate fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    classDef serverCrate fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef driverCrate fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    classDef halCrate fill:#e1f5fe,stroke:#0277bd,stroke-width:2px

    class SYSAPI_CRATE apiCrate
    class SERVER_CRATE serverCrate
    class CLK_CRATE,PWR_CRATE,GPIO_CRATE driverCrate
    class HAL_CRATE halCrate
```

## Configuration and Build System

```mermaid
graph TB
    subgraph "Build-Time Configuration"
        subgraph "Configuration Sources"
            APP_TOML[app.toml<br/>• System configuration<br/>• Clock settings<br/>• Power policies<br/>• Pin assignments]
            BOARD_CFG[board.toml<br/>• Hardware specifics<br/>• Crystal frequencies<br/>• Power supply config<br/>• Pin constraints]
            CHIP_CFG[chip.toml<br/>• MCU capabilities<br/>• Clock limits<br/>• Power features<br/>• GPIO counts]
        end
        
        subgraph "Code Generation"
            BUILD_RS[build.rs<br/>• Parse configurations<br/>• Validate settings<br/>• Generate code<br/>• Create constants]
        end
        
        subgraph "Generated Code"
            SYS_CONFIG[sys_config.rs<br/>• Clock tree constants<br/>• Power domain settings<br/>• Pin assignments<br/>• Interrupt mappings]
            BOARD_DEF[board_def.rs<br/>• Hardware definitions<br/>• Resource allocation<br/>• Capability flags<br/>• Constraint checks]
        end
    end

    APP_TOML --> BUILD_RS
    BOARD_CFG --> BUILD_RS
    CHIP_CFG --> BUILD_RS
    
    BUILD_RS --> SYS_CONFIG
    BUILD_RS --> BOARD_DEF

    classDef configFile fill:#e8eaf6,stroke:#3f51b5,stroke-width:2px
    classDef buildScript fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    classDef generatedCode fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px

    class APP_TOML,BOARD_CFG,CHIP_CFG configFile
    class BUILD_RS buildScript
    class SYS_CONFIG,BOARD_DEF generatedCode
```

## Memory and Resource Management

```mermaid
graph TB
    subgraph "System Control Memory Layout"
        subgraph "Flash Memory"
            SERVER_CODE[Server Code<br/>~12KB<br/>• IPC handling<br/>• State machines<br/>• Policy logic]
            DRIVER_CODE[Driver Code<br/>~8KB<br/>• Register access<br/>• Hardware control<br/>• Error handling]
            CONFIG_DATA[Configuration<br/>~2KB<br/>• Clock settings<br/>• Power policies<br/>• Pin mappings]
        end
        
        subgraph "RAM Memory"
            SERVER_STACK[Server Stack<br/>~1KB<br/>• Call stack<br/>• Local variables<br/>• IPC buffers]
            SYSTEM_STATE[System State<br/>~512B<br/>• Clock status<br/>• Power state<br/>• Configuration cache]
            DRIVER_STATE[Driver State<br/>~256B<br/>• Hardware status<br/>• Register shadows<br/>• Error counters]
        end
        
        subgraph "Hardware Resources"
            CLOCK_REGS[Clock Registers<br/>• RCC block<br/>• PLL configuration<br/>• Clock enables<br/>• Status flags]
            POWER_REGS[Power Registers<br/>• PWR block<br/>• Voltage scaling<br/>• Sleep control<br/>• Wake sources]
            GPIO_REGS[GPIO Registers<br/>• GPIO banks<br/>• Pin configuration<br/>• Alternate functions<br/>• Pull resistors]
            SYSCFG_REGS[SYSCFG Registers<br/>• System config<br/>• Pin remapping<br/>• Interrupt routing<br/>• Memory mapping]
        end
    end

    classDef flashMem fill:#e1f5fe,stroke:#0277bd,stroke-width:2px
    classDef ramMem fill:#e8f5e8,stroke:#388e3c,stroke-width:2px
    classDef hwRegs fill:#fff3e0,stroke:#f57c00,stroke-width:2px

    class SERVER_CODE,DRIVER_CODE,CONFIG_DATA flashMem
    class SERVER_STACK,SYSTEM_STATE,DRIVER_STATE ramMem
    class CLOCK_REGS,POWER_REGS,GPIO_REGS,SYSCFG_REGS hwRegs
```

## Key System Control Features

### **Clock Management**
- **PLL Configuration**: Multiple PLLs for different subsystems
- **Clock Tree Control**: Hierarchical clock distribution
- **Dynamic Frequency Scaling**: Runtime frequency adjustment
- **Clock Gating**: Power optimization through selective clock disable

### **Power Management**
- **Voltage Scaling**: Dynamic voltage adjustment for performance/power trade-off
- **Sleep Modes**: Multiple low-power states with different wake latencies
- **Power Domain Control**: Independent control of power islands
- **Wake Source Management**: Configurable wake-up triggers

### **GPIO and Pin Control**
- **Pin Multiplexing**: Dynamic alternate function assignment
- **Electrical Configuration**: Drive strength, pull-up/down, speed settings
- **Pin Remapping**: Software-configurable pin assignments
- **Interrupt Configuration**: External interrupt source management

### **Reset and System Control**
- **Reset Generation**: Software and hardware reset control
- **Reset Sequencing**: Ordered reset of subsystems
- **Watchdog Management**: System reliability through watchdog timers
- **System Configuration**: Memory mapping and system feature control

## Architectural Benefits

### **Centralized Control**
- **Single Authority**: One server manages all system-level resources
- **Coordination**: Prevents conflicts between subsystem configurations
- **Policy Enforcement**: Consistent application of power and clock policies

### **Safety and Security**
- **Controlled Access**: All system control goes through validated server
- **Resource Protection**: Prevents unauthorized hardware reconfiguration
- **State Validation**: Ensures system remains in valid configurations

### **Flexibility and Maintainability**
- **Modular Design**: Separate drivers for different subsystems
- **Configuration-Driven**: Build-time configuration for different targets
- **Extensible**: Easy to add new power states or clock configurations

This system control architecture provides **comprehensive management of critical system resources** while maintaining the security, determinism, and reliability characteristics essential for mission-critical embedded systems.

## System API Architecture Clarification

### **Important: This IS IPC - Task-to-task communication**

The `drv-stm32xx-sys-api` crate uses **IPC for task-to-task communication** to access system hardware services.

```rust
// From the actual I2C server code:
use drv_stm32xx_sys_api::{Sys, PinSet, OutputType, Speed, Pull};

// Gets a task ID - this is talking to another task via IPC
let sys = SYS.get_task_id();
let sys = Sys::from(sys);

// These function calls hide IPC messages underneath
sys.gpio_configure_alternate(pin, OutputType::OpenDrain, Speed::Low, Pull::None, function);
sys.gpio_set(scl_pin);
sys.gpio_reset(scl_pin);
```

### **What this ACTUALLY is:**

Looking at the pattern `SYS.get_task_id()`, this proves:

1. **SYS** is a **task slot** (like `I2C_SERVER`)
2. **Sys::from(task_id)** creates an **IPC client** 
3. **gpio_configure_alternate()** etc. are **IPC calls** wrapped as function calls

### **Corrected Architecture Pattern:**

```mermaid
graph TB
    subgraph "Actual Hubris System Architecture"
        subgraph "Application/Driver Tasks"
            I2CSERVER[I2C Server Task]
            OTHERDRV[Other Driver Tasks]
        end
        
        subgraph "System Task"
            SYSTASK[SYS Task<br/>• GPIO control<br/>• Clock management<br/>• Pin configuration<br/>• Hardware access]
        end
        
        subgraph "System API Crate"
            SYSAPI_IMPL[drv-stm32xx-sys-api<br/>• Sys client struct<br/>• IPC message wrappers<br/>• Hardware abstractions<br/>• Task-to-task communication]
        end
        
        subgraph "Hardware"
            STMHW[STM32 Hardware<br/>• GPIO registers<br/>• RCC registers<br/>• PWR registers]
        end
    end

    I2CSERVER --> SYSAPI_IMPL
    OTHERDRV --> SYSAPI_IMPL
    SYSAPI_IMPL -.->|IPC Messages| SYSTASK
    SYSTASK --> STMHW

    classDef taskLayer fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef systemTask fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef apiLayer fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef hwLayer fill:#fff3e0,stroke:#e65100,stroke-width:2px

    class I2CSERVER,OTHERDRV taskLayer
    class SYSTASK systemTask
    class SYSAPI_IMPL apiLayer
    class STMHW hwLayer
```

### **Likely Implementation Pattern:**

```rust
// drv-stm32xx-sys-api probably works like this:
pub struct Sys {
    task_id: TaskId,  // The SYS task ID
}

impl Sys {
    pub fn from(task_id: TaskId) -> Self {
        Self { task_id }
    }
    
    /// Send IPC message to SYS task for GPIO configuration
    pub fn gpio_configure_alternate(&self, pin: PinSet, ...) {
        let request = GpioConfigRequest {
            pin,
            mode: AlternateFunction,
            output_type: OpenDrain,
            // ...
        };
        
        // This is likely an IPC send to the SYS task
        let response = sys::send(self.task_id, &request);
        // Handle response...
    }
    
    /// Send IPC message to SYS task for GPIO control
    pub fn gpio_set(&self, pin: PinSet) {
        let request = GpioSetRequest { pin };
        sys::send(self.task_id, &request);
    }
}
```

### **Key Pattern Recognition:**

| Aspect | `drv-i2c-api` (Client-Server IPC) | `drv-stm32xx-sys-api` (System Service IPC) |
|--------|----------------------------------|---------------------------------------------|
| **Pattern** | `I2C_SERVER.get_task_id()` | `SYS.get_task_id()` |
| **Client Creation** | Client for I2C operations | Client for system operations |
| **Message Types** | I2C read/write requests | GPIO/clock/power requests |
| **Server Task** | I2C server task | SYS server task |
| **Purpose** | Peripheral communication | System resource management |

### **Both are IPC - Different Purposes:**

- **`drv-i2c-api`**: Application tasks → I2C server → Hardware peripherals
- **`drv-stm32xx-sys-api`**: Driver tasks → SYS server → System hardware

### **Conclusion:**

The `drv-stm32xx-sys-api` is **also IPC**, just like `drv-i2c-api`. The key insight is that `SYS.get_task_id()` indicates there's a **system server task** that manages GPIO, clocks, and power - similar to how there's an I2C server task for I2C operations.

This maintains Hubris's principle of **controlled hardware access** through dedicated server tasks rather than direct kernel system calls.
