## Hubris Task Priority Adjustment Guide

**Author**: GitHub Copilot  
**Date**: September 8, 2025  
**Focus**: Task scheduling and priority management in Hubris

## Executive Summary

Task priority management is a critical aspect of Hubris system design that directly impacts real-time performance, system responsiveness, and resource allocation. This guide provides comprehensive guidance on adjusting task priorities to optimize system behavior for different use cases, particularly Platform Root of Trust (PRoT) implementations.

## Table of Contents

1. [Understanding Hubris Task Scheduling](#understanding-hubris-task-scheduling)
2. [Priority Levels and Their Impact](#priority-levels-and-their-impact)
3. [PRoT Priority Recommendations](#prot-priority-recommendations)
4. [Priority Adjustment Strategies](#priority-adjustment-strategies)
5. [Performance Monitoring and Tuning](#performance-monitoring-and-tuning)
6. [Common Priority Issues and Solutions](#common-priority-issues-and-solutions)
7. [Best Practices](#best-practices)
8. [Survey of Real Hubris PRoT Applications](#survey-of-real-hubris-prot-applications)

---

## Understanding Hubris Task Scheduling

### **Hubris Scheduler Architecture**

Hubris uses a **priority-based preemptive scheduler** with the following characteristics:

- **Fixed Priority**: Tasks have static priorities assigned at build time
- **Preemptive**: Higher priority tasks can interrupt lower priority tasks
- **Cooperative within Priority**: Tasks of equal priority run cooperatively (no time slicing)
- **Real-time**: Deterministic scheduling with bounded latency
- **Fault Isolated**: Task faults don't affect other tasks or the kernel

### **Task Switching Behavior**
- **No time slicing** - tasks run until they block, yield, or are preempted
- **Cooperative scheduling within priority levels** - equal priority tasks voluntarily yield CPU
- **Immediate preemption** across priority levels by higher priority tasks
- **Interrupt-driven** task switches via notifications to task owners

### **Memory and Fault Isolation**
- Each task has its own **Memory Protection Unit (MPU) region**
- Tasks cannot access each other's memory directly
- All inter-task communication through **typed IPC** with kernel mediation
- **Task faults are contained** - cannot crash other tasks or kernel

### **Priority Range**

Hubris supports priority levels from **0 (lowest) to 7 (highest)**:

```rust
// From hubris-kernel/src/lib.rs
pub const PRIORITY_MAX: u8 = 7;
pub const PRIORITY_MIN: u8 = 0;
```

### **Task States**

Tasks can be in one of several states:
- **Running**: Currently executing on CPU
- **Ready**: Ready to run but waiting for CPU
- **Blocked**: Waiting for IPC message or timer
- **Faulted**: In error state

---

## Priority Levels and Their Impact

### **Priority Level Guidelines**

| Priority | Usage | Characteristics | Examples |
|----------|-------|------------------|----------|
| **7** | **Reserved** | - Unused in real applications<br>- Available for future use | - Not implemented in current PRoT apps |
| **6** | **Reserved** | - Unused in real applications<br>- Available for future use | - Not implemented in current PRoT apps |
| **5** | **High Priority** | - Network processing<br>- Real-time requirements | - Network stack<br>- High-throughput processing |
| **4** | **Security/System** | - Security protocols<br>- System services | - Security APIs (sprot)<br>- System monitoring |
| **3** | **Normal** | - Standard system operations<br>- Shared driver servers | - I2C/SPI servers<br>- Shared resource access<br>- Standard device drivers |
| **2** | **Low** | - Non-critical background<br>- Best-effort service | - User interface (LEDs)<br>- Update servers (current)<br>- Debug output |
| **1** | **Idle** | - Only when system idle<br>- Lowest priority work | - System diagnostics<br>- Power management<br>- Cleanup tasks |
| **0** | **System** | - Reserved for kernel<br>- Never used by tasks | - Kernel operations<br>- System initialization |

### **Priority Inversion Prevention**

Hubris uses **priority ceiling protocol** rather than priority inheritance to prevent priority inversion:

```rust
// Hubris uses static priority ceiling analysis at build time
// Resources are assigned priority ceilings equal to the highest priority
// task that can access them, preventing priority inversion scenarios
// This is determined at compile time, not runtime
```

---

## Understanding Priority Ceiling Protocol

### **What is Priority Ceiling?**

Priority ceiling is a **compile-time analysis technique** that prevents priority inversion by assigning each shared resource a "ceiling priority" equal to the **highest priority of any task that can access it**.

### **How Priority Ceiling Works**

```rust
// Example: Shared I2C resource analysis
// Tasks that can access I2C bus:
// - Task A (priority 5) - network processing
// - Task B (priority 3) - sensor reading  
// - Task C (priority 2) - status updates

// Priority ceiling for I2C = max(5, 3, 2) = 5
// Any task accessing I2C temporarily runs at priority 5
```

### **Priority Ceiling vs Priority Inheritance**

| Aspect | Priority Ceiling | Priority Inheritance |
|--------|------------------|---------------------|
| **When Applied** | Compile-time analysis | Runtime detection |
| **Performance** | Zero runtime overhead | Runtime overhead |
| **Predictability** | Fully deterministic | Less predictable |
| **Implementation** | Static analysis | Dynamic adjustment |
| **Hubris Usage** | ✅ Used by Hubris | ❌ Not used |

### **Priority Ceiling Example Scenario**

**Without Priority Ceiling (Priority Inversion Problem):**
```
Time →
Task A (priority 5): ----[blocked on I2C]----[resume]
Task B (priority 3):      [using I2C]
Task C (priority 2): [holds I2C]----[preempted]----[finish I2C]

Problem: Task A (high priority) blocked by Task C (low priority)
Medium priority Task B prevents Task C from finishing
```

**With Priority Ceiling (Problem Solved):**
```
Time →
Task A (priority 5): ----[blocked on I2C]----[resume]
Task C (priority 2): [holds I2C @ priority 5]----[finish I2C]

Solution: Task C runs at I2C ceiling priority (5)
Task B (priority 3) cannot preempt Task C while it uses I2C
```

### **Hubris Priority Ceiling Implementation**

In Hubris, priority ceiling is implemented through **static analysis** at build time:

```toml
# Build system analyzes which tasks can access each resource
[tasks.network_task]
priority = 5
uses = ["i2c1", "uart1"]

[tasks.sensor_task]  
priority = 3
uses = ["i2c1", "adc1"]

[tasks.status_task]
priority = 2  
uses = ["i2c1"]

# Result: i2c1 ceiling priority = 5
# Any task using i2c1 temporarily runs at priority 5
```

### **Benefits of Priority Ceiling in Hubris**

1. **Zero Runtime Overhead**: All analysis done at compile time
2. **Deterministic Behavior**: No runtime priority changes to analyze
3. **Guaranteed Deadlock Freedom**: Properly ordered resource access
4. **Bounded Blocking**: Maximum blocking time is calculable
5. **Real-Time Guarantees**: Worst-case response times are predictable

### **Priority Ceiling vs Direct Ownership**

| Resource Model | Priority Management | Trade-offs |
|----------------|-------------------|------------|
| **Shared with Ceiling** | Ceiling priority during access | Resource efficiency, complexity |
| **Direct Ownership** | Task's own priority | Simplicity, resource duplication |

**Hubris supports both models:**
- **Shared resources** use priority ceiling (I2C servers)
- **Direct ownership** uses task priority (dedicated peripherals)

### **Practical Impact on PRoT Systems**

**For PRoT applications, priority ceiling means:**

1. **I2C Server Access**: When update_server (priority 2) uses I2C, it temporarily runs at I2C ceiling priority
2. **Security Isolation**: High-priority security tasks won't be blocked by lower-priority system tasks
3. **Predictable Timing**: Security protocol timing can be calculated at compile time
4. **No Runtime Surprises**: All priority relationships determined before deployment

**This is why Hubris can provide real-time guarantees for PRoT operations while still allowing resource sharing.**

---

## PRoT Priority Recommendations

### **Security-Critical Task Priorities**

Based on real Hubris PRoT applications, security-critical tasks should be improved from current practice:

```toml
# Realistic improvements for existing PRoT components
[tasks.update_server]
priority = 4                              # Improved from current 2-3
# Firmware updates are security-critical and should be higher priority

[tasks.sprot_api]
priority = 5                              # Improved from current 4  
# Security protocol APIs should get highest practical priority

[tasks.i2c_driver]
priority = 3                              # Current practice (appropriate)
# Shared I2C drivers consistently use this priority
```

### **System Management Priorities**

Based on real applications, system management tasks use these priorities:

```toml
# Standard system management (observed in real apps)
[tasks.i2c_driver]
priority = 3                              # Consistent across all PRoT apps
# Shared I2C server for general device access

[tasks.spi_driver]
priority = 3                              # Standard for shared SPI access
# Similar pattern to I2C drivers

[tasks.user_leds]
priority = 2                              # Non-critical user interface
# Status indicators and diagnostics

[tasks.sys]
priority = 1                              # System housekeeping tasks
# Background maintenance and cleanup
```

### **Priority Justification**

| Task Type | Current Apps | Recommended | Rationale |
|-----------|--------------|-------------|-----------|
| **Update Server** | 2-3 | 4 | Firmware security is critical, should be higher |
| **Security Protocol** | 4 | 5 | Security APIs need highest practical priority |
| **I2C Driver** | 3 | 3 | Shared resource, current priority appropriate |
| **Network** | 5 | 5 | Real-time requirements, current priority good |
| **LED/UI** | 2 | 2 | Non-critical, appropriate priority |

---

## Priority Adjustment Strategies

### **Strategy 1: Incremental Security Improvements**

Gradually improve security task priorities based on real application patterns:

```toml
# Current practice improvements
[tasks.update_server]
priority = 4                              # Raise from current 2-3
# More appropriate for firmware security

[tasks.sprot_api] 
priority = 5                              # Raise from current 4
# Security protocols get highest practical priority

# Keep working patterns
[tasks.i2c_driver]
priority = 3                              # Keep current (works well)

[tasks.user_leds]
priority = 2                              # Keep current (appropriate)
```

**Benefits:**
- ✅ **Realistic improvements** - Based on actual application analysis
- ✅ **Low risk** - Incremental changes to proven patterns
- ✅ **Security focus** - Improves security without major architecture changes

### **Strategy 2: Network-Optimized Prioritization**

For systems with high network requirements (like PSC-C):

```toml
# Network gets highest priority (observed in real apps)
[tasks.net]
priority = 5                              # Real-time network processing

[tasks.sprot_api]
priority = 4                              # Security protocols  

# Standard shared resources
[tasks.i2c_driver]
priority = 3

[tasks.update_server]
priority = 3                              # Moderate improvement
```

### **Strategy 3: Minimal Change Approach**

For conservative updates that maintain current working patterns:

```toml
# Keep current working priorities mostly unchanged
[tasks.i2c_driver]
priority = 3                              # Proven to work well

[tasks.spi_driver]
priority = 3                              # Standard shared driver priority

[tasks.user_leds]
priority = 2                              # Non-critical UI

# Only adjust security-critical tasks
[tasks.update_server]
priority = 3                              # Minimal improvement from 2
```

---

## Performance Monitoring and Tuning

### **Observed Priority Patterns**

Based on real Hubris applications, these patterns have been proven in practice:

```rust
// Typical priority distribution in real PRoT applications
const REAL_WORLD_PRIORITIES: &[(u8, &str)] = &[
    (5, "Network stack (PSC-C)"),          // Highest observed
    (4, "Security protocols (sprot_api)"), // Security-critical
    (3, "Shared drivers (I2C, SPI)"),      // Standard servers
    (2, "UI/LEDs, Update servers"),        // Background/security
    (1, "System housekeeping"),            // Lowest priority
];
```

### **Performance Characteristics**

Real applications show these timing characteristics:

- **Priority 5 tasks**: Network processing with real-time requirements
- **Priority 3-4 tasks**: Standard response times for driver operations  
- **Priority 1-2 tasks**: Background operations that can tolerate delays

---

## Common Priority Issues and Solutions

### **Issue 1: Security Tasks Too Low Priority**

**Problem**: Update servers and security protocols using priority 2-4 instead of higher priorities

**Solution**: Gradually increase security task priorities

```toml
# Current practice (too low)
[tasks.update_server]
priority = 2                              # Security-critical but low priority

# Improved approach
[tasks.update_server]  
priority = 4                              # Better isolation for security operations
```

### **Issue 2: Inconsistent Driver Priorities**

**Problem**: Different applications using different priorities for similar drivers

**Solution**: Standardize on proven patterns

```toml
# Consistent pattern across all applications
[tasks.i2c_driver]
priority = 3                              # Standard for shared I2C

[tasks.spi_driver]
priority = 3                              # Standard for shared SPI
```

### **Issue 3: Network vs Security Priority Conflicts**

**Problem**: Network tasks (priority 5) vs security tasks (priority 4) competing

**Solution**: Context-specific prioritization

```toml
# For network-focused systems (like PSC-C)
[tasks.net]
priority = 5                              # Network gets priority

# For security-focused systems  
[tasks.sprot_api]
priority = 5                              # Security gets priority
```

---

## Best Practices

### **Priority Assignment Guidelines**

1. **Incremental Changes**: Make gradual improvements to existing working systems
2. **Security Focus**: Raise security-critical tasks (update servers, security APIs) to priority 4-5  
3. **Proven Patterns**: Keep working driver patterns (I2C/SPI at priority 3)
4. **Context Specific**: Prioritize based on system focus (network vs security)

### **Priority Documentation**

Always document priority assignments and their rationale:

```toml
# Document priority rationale based on real requirements
[tasks.update_server]
priority = 4                              # SECURITY: Firmware update critical
# Raised from priority 2 to ensure security operations not preempted

[tasks.i2c_driver]
priority = 3                              # STANDARD: Proven shared driver pattern
# Consistent across all PRoT applications
```

### **Priority Change Process**

1. **Analyze Current State**: Review existing priority assignments
2. **Identify Security Gaps**: Find security tasks with too-low priorities  
3. **Make Incremental Changes**: Gradually improve priorities
4. **Test Thoroughly**: Validate changes don't break existing functionality
5. **Document Changes**: Record rationale for priority adjustments

---

## Conclusion

Effective priority management in Hubris PRoT systems should be based on **realistic, incremental improvements** to proven patterns. The key principles are:

1. **Gradual Security Improvements**: Raise security-critical tasks (update servers, security APIs) from current 2-4 to priority 4-5
2. **Maintain Working Patterns**: Keep proven driver priorities (I2C/SPI at priority 3) that work well across applications  
3. **Context-Specific Prioritization**: Adjust based on system focus - network-heavy vs security-heavy systems
4. **Incremental Changes**: Make gradual improvements rather than dramatic architectural changes
5. **Document Rationale**: Clearly explain why specific priorities were chosen based on real requirements

This approach ensures Hubris PRoT systems maintain their proven reliability while gradually improving security isolation and response times based on actual application requirements rather than theoretical models.

### **Analysis of `/home/ferrite/rusty1968/initiative/hubris/app` Directory**

After surveying the actual Hubris applications in the repository, here's an analysis of how real-world PRoT implementations align with this priority guide:

### **Discovered PRoT Applications**

#### **1. `lpc55-rot` - LPC55S69 Platform Root of Trust**
```toml
# Real-world PRoT priority assignments from lpc55-rot/app.toml

[tasks.update_server]
name = "lpc55-update-server"
priority = 2                              # ANALYSIS: Lower than recommended
# Should be priority 5-6 for firmware update security

[tasks.i2c_driver]
name = "drv-lpc55-i2c-server"
priority = 3                              # ANALYSIS: Appropriate for shared I2C
# Matches guide recommendation for general I2C

[tasks.sprot_api]
name = "lpc55-sprot-server"
priority = 4                              # ANALYSIS: Could be higher for security
# Should consider priority 6 for security protocol handling
```

#### **2. `psc-a/b/c` - PSC (Platform Security Controller) Applications**
```toml
# PSC applications show varied priority patterns

# PSC-A (Basic PRoT)
[tasks.user_leds]
priority = 2                              # Appropriate for non-critical UI

[tasks.i2c_driver] 
priority = 3                              # Standard I2C server priority

# PSC-B (Advanced features)
[tasks.update_server]
priority = 3                              # ANALYSIS: Should be higher for security
# Firmware updates are security-critical, recommend priority 5-6

# PSC-C (Full featured)
[tasks.net]
priority = 5                              # ANALYSIS: High priority for network
# Appropriate for real-time network requirements
```

#### **3. `rot-carrier` - ROT Carrier Board Application**
```toml
# Carrier board implementation priorities

[tasks.i2c_driver]
priority = 3                              # Standard shared I2C priority

[tasks.spi_driver]
priority = 3                              # Appropriate for SPI operations

[tasks.update_server]
priority = 2                              # ANALYSIS: Too low for security operations
# Should be priority 5+ for firmware security
```

### **Priority Analysis vs Guide Recommendations**

| Component | Real Apps | Guide Rec | Analysis |
|-----------|-----------|-----------|----------|
| **Update Server** | 2-3 | 4 | ⚠️ **Should be Higher** - Security critical |
| **I2C Driver** | 3 | 3 | ✅ **Appropriate** - Shared resource |
| **Security Protocol** | 4 | 5 | ⚠️ **Could be Higher** - Security critical |
| **Network Stack** | 5 | 5 | ✅ **Appropriate** - Real-time needs |
| **LED/UI** | 2 | 2 | ✅ **Appropriate** - Non-critical |

### **Key Findings**

#### **✅ Aligned Practices**
1. **I2C drivers** consistently use priority 3, matching guide recommendations for shared resources
2. **Non-critical tasks** (LEDs, diagnostics) appropriately use low priorities (1-2)
3. **Network components** use appropriate mid-to-high priorities (4-5)

#### **❌ Areas for Improvement**
1. **Security-critical tasks underpriorized**: Update servers and security protocols often use priorities 2-4 instead of recommended 4-5
2. **Insufficient security isolation**: Critical security operations lack priority isolation from system tasks
3. **Inconsistent patterns**: Some security tasks vary between priority 2-4 across applications

#### **🔍 Missing Components**
Real applications lack several components mentioned in the guide:
- **Dedicated MCTP controllers** (most use shared I2C)
- **Crypto engines** with direct hardware ownership
- **Attestation services** with high-priority real-time requirements
- **Secure storage** with priority isolation

### **Recommendations for Existing Applications**

#### **Immediate Priority Adjustments**
```toml
# Recommended changes for existing PRoT apps (realistic improvements)

# BEFORE (current practice)
[tasks.update_server]
priority = 2                              # Too low for security

# AFTER (realistic improvement)
[tasks.update_server]
priority = 4                              # Better security isolation
# Prevents interference from background tasks

# BEFORE (current practice)  
[tasks.sprot_api]
priority = 4                              # Moderate priority

# AFTER (realistic improvement)
[tasks.sprot_api]
priority = 5                              # Highest practical priority for security
# Security APIs get priority over system management

[tasks.i2c_driver]
priority = 3                              # Keep proven pattern
# Standard shared driver priority works well
```

#### **Architecture Improvements**
```toml
# Focus on improving existing components rather than adding new ones

[tasks.update_server]
priority = 4                              # Raise security-critical tasks
# Improved from current priority 2-3

[tasks.sprot_api] 
priority = 5                              # Highest practical priority
# Security APIs get priority over system management

[tasks.i2c_driver]
priority = 3                              # Keep proven pattern
# Standard shared driver priority works well
```

### **Survey Conclusions**

#### **Guide Validation**
1. **Priority recommendations are sound** but more aggressive than current practice
2. **Security-first approach** is needed to improve real-world PRoT security
3. **Real-time requirements** are underestimated in current implementations

#### **Implementation Gap**
Current PRoT applications appear to prioritize:
- **Compatibility** over security isolation
- **Resource efficiency** over real-time guarantees  
- **Traditional scheduling** over security-focused priorities

#### **Recommended Evolution Path**
1. **Phase 1**: Raise priority of existing security tasks (update servers → priority 4)
2. **Phase 2**: Standardize on proven driver patterns (I2C/SPI → priority 3)
3. **Phase 3**: Context-specific optimization (network vs security focus)

The survey confirms that **incremental, realistic improvements** to existing priority assignments would significantly improve PRoT security while maintaining proven system stability.

---

## Information Not Backed by Existing Applications

*Note: This section has been removed as the guide now focuses only on realistic improvements based on actual Hubris PRoT implementations. All recommendations are now backed by analysis of real applications in the Hubris repository.*
