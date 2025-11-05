# OpenPRoT Operating System Selection
## Choosing the Right Foundation for Platform Root of Trust

---

## Our Mission: Security That Cannot Fail

**Platform Root of Trust (PRoT) Requirements:**
- Hardware-enforced memory isolation
- Deterministic behavior
- Fault recovery without compromising system integrity
- Open-source, auditable foundation

> "The question isn't which OS has more features—it's which architecture makes failure impossible by design."

---

## Why This Decision Matters

**PRoT is the foundation of platform security:**
- First code to run on server boot
- Establishes trust for all subsequent operations
- Manages cryptographic attestation
- Handles secure firmware updates
- Cannot be restarted easily in production

**One wrong choice = compromised infrastructure at scale**

---

## Our Evaluation Framework

We assessed six critical dimensions:

1. 🛡️ **Memory protection and isolation** - Security boundaries
2. 🔄 **Fault tolerance and recovery** - System reliability
3. 📐 **Static vs. dynamic composition** - Predictability
4. 🎯 **System complexity and attack surface** - Maintainability
5. ⚡ **Preemptive scheduling** - Responsive behavior
6. 🔍 **Debuggability and observability** - Production monitoring

---

## The Contenders

### Two Production-Ready Rust Embedded Operating Systems

**Hubris** (Oxide Computer Company)
- Microkernel for server management
- Static task model
- MPU-enforced isolation
- Production-deployed since 2021

**Tock** (Stanford/MIT/Academia)
- General-purpose embedded OS
- Dynamic application loading
- Capsule-based architecture
- Research published 2017, production-proven since 2019+
- Notable deployments: Open Titan.

> Both use Rust for memory safety—but fundamentally different architectural philosophies

---

## Round 1: Memory Protection

### How Do We Prevent One Bug From Destroying Everything?

**Hubris: Full MPU Isolation**
- ✅ Drivers in separate MPU-protected memory spaces
- ✅ Kernel physically isolated from tasks
- ✅ Failing driver **cannot** corrupt kernel
- ✅ Component-level fault boundaries
- ✅ Hardware peripherals also isolated via MPU (see Appendix A8)

**Tock: Kernel-Space Drivers**
- ✅ Userspace processes have MPU isolation
- ⚠️ Drivers (capsules) run in kernel space with Rust safety
- ⚠️ Capsule panic affects entire kernel

**Winner: Hubris** — Finest-grained isolation for maximum fault containment

---

## Round 2: When Things Go Wrong

### Can We Recover Without Rebooting the Entire System?

**Hubris: Component-Level Recovery (Jefe Supervisor)**
- ✅ Supervisor task can restart individual crashed tasks
- ✅ In-place reinitialization (drivers, services, etc.)
- ✅ Memory isolation limits "blast radius"
- ✅ No system-wide reboot needed
- ✅ Production-proven in Oxide servers

**Tock: Process-Level Recovery**
- ✅ Can restart userspace processes independently
- ✅ MPU isolates process failures
- ⚠️ Kernel capsule panic requires full kernel restart
- ⚠️ Less granular than per-component recovery
- ✅ Production-proven architecture

**Winner: Hubris** — Finest-grained recovery for continuous operation

---

## Round 3: System Composition

### When Do We Know What's Running?

**Hubris: Compile-Time Certainty**
```toml
# app.toml - ALL tasks defined at build time
[tasks.crypto]
priority = 1
memory = "64KB"
interrupts = ["AES_IRQ"]
```
- ✅ Static assertions verify configuration
- ✅ Build fails if resources exceeded
- ✅ **Zero runtime surprises**

**Tock: Runtime Flexibility**
- ✅ Processes loaded dynamically (useful for multi-app systems)
- ✅ Grant-based allocation with deterministic cleanup
- ⚠️ More runtime complexity to validate
- ⚠️ Broader set of possible system states

**Winner: Hubris** — Static model best matches PRoT's known requirements

---

## The "Aggressively Static" Advantage

**If the build succeeds, these failures are impossible:**

❌ Resource exhaustion  
❌ Invalid task communication paths  
❌ Memory allocation failures  
❌ IRQ conflicts  
❌ Priority inversions  

> "Move errors from 2am in production to 2pm during development"

---

## Round 4: Communication Architecture

### How Do Tasks Talk to Each Other?

**Hubris: Synchronous IPC (L4-inspired)**
```rust
// Sender blocks until reply received
let result = sys_send(task_id, message);
// Either success or precise fault location
```
- ✅ No race conditions (deterministic ordering)
- ✅ Precise fault isolation (REPLY_FAULT)
- ✅ Direct memory copy (zero-copy)
- ✅ Extends Rust ownership across tasks

**Tock: Asynchronous Upcalls**
- ✅ Non-blocking design (different performance trade-offs)
- ⚠️ Callback-based event delivery
- ⚠️ More complex reasoning about event ordering
- ✅ Well-suited for dynamic process model

**Winner: Hubris** — Synchronous model simplifies reasoning for PRoT

---

## Round 5: Attack Surface & Maturity

### What Can Go Wrong?

**Hubris: Minimal by Design**
- ✅ No dynamic memory allocation
- ✅ No task creation/destruction at runtime
- ✅ No runtime resource management
- ✅ Application-specific kernel (dead code eliminated)
- ✅ **Production-deployed at Oxide Computer**

**Tock: Flexible but Broader**
- ⚠️ Dynamic application loading
- ⚠️ Grant-based allocation system
- ⚠️ General-purpose kernel (includes unused features)
- ✅ **Multiple production deployments**

**Winner: Hubris** — Minimal attack surface + production validation

---

## Round 6: Debugging and Observability

### How Do We Inspect System Behavior Without Compromising Security?

**Hubris: Kernel-Aware Debugger (Humility)**
- ✅ **NO** console interfaces in application code
- ✅ **NO** printf formatting code in system
- ✅ **NO** command parsing vulnerabilities
- ✅ External debugger with Debug Binary Interface (DBI)
- ✅ Full core dumps for post-mortem analysis
- ✅ Production-proven at Oxide

**Tock: Standard Debugging Approaches**
- ✅ Supports standard embedded debugging tools
- ✅ GDB integration for kernel debugging
- ✅ Flexible - applications choose their debug strategy
- ⚠️ No built-in kernel-aware debugging framework
- ⚠️ System observability depends on application implementation

**Winner: Hubris** — Integrated debugging architecture with security by design

---

## Visual Comparison: Architecture Philosophy

```
Hubris Philosophy: Eliminate Uncertainty
┌─────────────────────────────────────┐
│  Build Time: Validate Everything    │
│  Runtime: Execute Only              │
│  Failure: Impossible by Construction│
│  Status: Production-Proven          │
└─────────────────────────────────────┘

Tock Philosophy: Enable Flexibility
┌─────────────────────────────────────┐
│  Build Time: Prepare Framework      │
│  Runtime: Adapt and Allocate        │
│  Failure: Handle Gracefully         │
│  Status: Production-Deployed        │
└─────────────────────────────────────┘
```

**For PRoT: We choose proven "cannot fail" over flexible "handle dynamically"**

---

## Key Differentiators Summary

| Critical Feature | Hubris | Tock |
|-----------------|--------|------|
| **Memory Isolation** | All components (MPU) ✅ | Processes only (MPU) ⚠️ |
| **Fault Recovery** | Component-level ✅ | Process-level ⚠️ |
| **Composition** | Static ✅ | Dynamic ⚠️ |
| **Resource Allocation** | Compile-time ✅ | Runtime ⚠️ |
| **Scheduling** | Preemptive ✅ | Cooperative ⚠️ |
| **IPC Model** | Synchronous ✅ | Asynchronous ⚠️ |
| **Production Status** | **Deployed (2021+)** ✅ | **Deployed (2017+)** ✅ |
| **Security Audits** | Yes ✅ | Yes ✅ |
| **Best Fit** | Single-purpose infrastructure ✅ | Multi-app platforms ⚠️ |

**Legend:** ✅ = Optimal for PRoT | ⚠️ = Different trade-offs

---

## The "But What About..." Slide

**Q: Doesn't Tock have production deployments in security systems?**  
A: Yes! Tock is excellent, production-proven engineering (deployed since 2017). Different architecture philosophy optimized for multi-application embedded systems. For PRoT's single-purpose, known-at-build-time requirements, Hubris's static model is a better architectural fit.

**Q: What about RISC-V support?**  
A: Hubris designed with RISC-V in mind. Straightforward port (narrow scope, simple execution model, minimal assembly).

**Q: Isn't static composition too restrictive?**  
A: Not for PRoT. We know exactly what we need at build time. Flexibility adds risk without benefit for this use case.

**Q: What about the MPL 2.0 license?**  
A: Commercial use allowed. Modified MPL files must remain MPL and be shared. Works fine with proprietary code.

---

## Real-World Implications

### Scenario: Driver Crashes During Boot in Remote Data Center

**Hubris Response (with Jefe supervisor):**
1. 🛡️ Kernel detects fault (MPU violation)
2. 📞 Notifies jefe (supervisor task)
3. 🔄 Jefe restarts just that driver component
4. ✅ System continues booting, other components unaffected
5. ⏱️ Total impact: ~10 milliseconds

**Tock Response:**
1. 🛡️ If process fails: kernel restarts process, continues ✅
2. ⚠️ If capsule (kernel driver) panics: kernel restart required
3. ⏱️ Total impact: seconds (for kernel restart)

**Why This Matters for PRoT:**
- Remote data center deployment (no physical access)
- Every second of downtime is costly
- Component-level recovery vs. system-level recovery
- Granularity of fault isolation directly impacts availability

---

## Resource Efficiency: Both Excel

**SRAM Efficiency:**
- ✅ Hubris: Execute-in-place (XIP) from flash
- ✅ Tock: Execute-in-place (XIP) from flash
- ✅ Both: SRAM only for data/stack/heap

**The Difference:**
- Hubris: Application-specific kernel (maximum efficiency)
- Tock: General-purpose kernel (optimal efficiency)

**For constrained PRoT hardware: Every byte counts**

---

## The Critical Question

### What Happens at 2am When Something Goes Wrong?

**Hubris Design:**
- Most problems prevented at compile time
- Runtime faults isolated and recoverable
- No dynamic allocation to exhaust
- No race conditions from async messaging
- External debugging with no security compromise

**This is the PRoT requirement:** *Boring reliability over exciting flexibility*

---

## Different Tools for Different Jobs

**Tock is Excellent For:**
- ✅ Multi-application embedded platforms
- ✅ Research and educational systems
- ✅ Systems requiring runtime app loading/updates
- ✅ Scenarios where dynamic flexibility is valuable
- ✅ **Production-proven since 2017**

**Hubris is Optimal For:**
- ✅ Single-purpose security infrastructure
- ✅ Platform root of trust
- ✅ Server management controllers
- ✅ Known-at-build-time system composition
- ✅ **"Cannot fail" architectures**

---

## Our Recommendation

### **Hubris is the Right Choice for OpenPRoT**

**Not because Tock is inferior—but because:**

1. 🎯 **Architectural alignment** — Static model matches PRoT's known requirements
2. 🛡️ **Finest-grained isolation** — Component-level MPU boundaries
3. 📐 **Predictability** — Compile-time validation eliminates runtime unknowns
4. 🔄 **Component recovery** — Jefe supervisor enables per-task restart
5. 🎪 **Focused simplicity** — Designed specifically for infrastructure management
6. ✅ **Production-proven** — Deployed and validated at Oxide Computer
7. 🔒 **Security-audited** — Third-party validation completed

**Each OS excels in its domain:**
- **Tock:** Excellent for multi-app platforms with dynamic loading  
- **Hubris:** Purpose-built for single-purpose, cannot-fail infrastructure

---

## The Core Principle

> "For platform root of trust, we prioritize **avoiding complexity** over **gaining flexibility**"

**PRoT Doesn't Need:**
- Dynamic application loading
- Runtime resource allocation
- Flexible multi-tenancy
- General-purpose capabilities

**PRoT Needs:**
- Deterministic behavior
- Provable correctness
- Minimal attack surface
- Reliable fault recovery

**Hubris provides exactly what we need, nothing more**

---

## Final Thought: Philosophy Matters

### The "Aggressively Static" Philosophy

**Hubris's Design Principle:**
*"If we can check it at build time, we must check it at build time"*

**For PRoT, this means:**
- Configuration errors found in CI/CD, not in data centers
- Resource exhaustion impossible by construction
- Security analysis on fixed system composition
- Audit trail includes complete build-time validation

**This architectural philosophy makes PRoT feasible at scale**

---

## Next Steps

**Implementation Roadmap:**
1. ✅ Complete OS evaluation (this presentation)
2. 🔄 RISC-V port planning and execution
3. 🔄 HAL development on Hubris
4. 🔄 Service integration (SPDM, MCTP, PLDM)
5. 🔄 Security validation and audit
6. 🔄 Production deployment

**The foundation is solid. Time to build.**

---

## Questions?

**Key Takeaways:**
- ✅ Both OSes represent strong, production-proven engineering
- ✅ Tock: Optimized for multi-app platforms (2017+)
- ✅ Hubris: Optimized for infrastructure management (2021+)
- ✅ Architecture match matters: static PRoT requirements → static OS design
- ✅ For PRoT: Component-level recovery > Process-level recovery
- ✅ Choice driven by requirements, not superiority

**References available in full whitepaper**

---

## Thank You

**OpenPRoT: Open-Source Platform Root of Trust**

*Building secure infrastructure through principled OS selection*

---

# Appendix: Technical Deep Dives

## A1: Synchronous IPC Detailed Example

```rust
// Task A wants to read from I2C device
let request = I2cRead { addr: 0x50, len: 32 };

// This BLOCKS until driver processes and replies
let response = sys_send(i2c_driver, &request)?;

// Either:
// 1. Success: response contains data
// 2. REPLY_FAULT: driver crashed at precise location
// 3. Timeout: configurable deadline exceeded

// No race conditions, no message queue management
// Direct memory copy, zero-copy semantics
```

**Benefits for Security:**
- Deterministic behavior
- Clear error attribution
- No asynchronous state to manage

---

## A2: Static Validation Example

```toml
# app.toml configuration
[tasks.network]
priority = 3
memory = "128KB"
interrupts = ["ETH_IRQ"]

[tasks.crypto]  
priority = 2  # ERROR: Higher priority task created after
              # lower priority = inversion detected at build
```

**Build System Validates:**
- Total memory ≤ physical RAM
- No IRQ conflicts
- Priority ordering consistency
- Communication path validity
- Stack overflow impossible

---

## A3: Fault Recovery Flow

```
┌─────────────┐
│ Driver Task │ ──────┐
│ (I2C)       │       │ 1. Memory violation
└─────────────┘       ↓
                  ┌────────┐
                  │ Kernel │
                  └────────┘
                      │
                      │ 2. REPLY_FAULT
                      ↓
                ┌──────────────┐
                │ Supervisor   │
                │ Task         │
                └──────────────┘
                      │
                      │ 3. sys_restart(i2c_driver)
                      ↓
┌─────────────┐
│ Driver Task │ ◄─────┐ 4. Reset registers/stack
│ (I2C)       │       │    Clear resources
└─────────────┘       │    Task ready again
        │
        └─────── 5. Normal operation resumes
```

**Key Point:** Other tasks unaffected during recovery

---

## A4: Memory Layout Comparison

**Hubris:**
```
┌──────────────┐ 0x20000000
│ Task 1 Data  │ (Fixed, MPU-protected)
├──────────────┤
│ Task 2 Data  │ (Fixed, MPU-protected)
├──────────────┤
│ Task 3 Data  │ (Fixed, MPU-protected)
└──────────────┘

No runtime allocation
No fragmentation
Predictable usage
```

**Tock:**
```
┌──────────────┐ 0x20000000
│ Kernel       │
├──────────────┤
│ Grant Region │ (Dynamic allocation)
│  ↓ grows ↓   │
├──────────────┤
│  ↑ grows ↑   │
│ Process      │ (Dynamic, reclaimed on exit)
└──────────────┘

Flexible allocation
Deterministic reclamation
Runtime adaptation
```

---

## A5: Scheduling Comparison

**Hubris Preemptive Scheduling:**
```
Priority 1 (Highest): ████████████████
Priority 2:           ──██──██───██──
Priority 3:           ────────██─────
Priority 4 (Lowest):  ──────────────█

High priority ALWAYS preempts lower
Deterministic response times
Critical operations never blocked
```

**All tasks are preemptively scheduled:**
- Crypto task can interrupt network task mid-operation
- Real-time guarantees for security-critical operations
- Deadlines enforceable through priority

---

**Tock Cooperative Scheduling:**
```
Kernel + Capsules: ████████████████████ (cooperative)
Process A:         ──────██████──────── (preemptive)
Process B:         ────────────████──── (preemptive)

Capsules (kernel drivers) cooperatively scheduled
User processes preemptively scheduled (round-robin)
```

**Key architectural detail:**
- **Capsules run at kernel level** - cooperatively scheduled
- **Capsule must yield** for other capsules to run
- **Long-running capsule** can delay other kernel operations
- **User processes** are preemptively scheduled

**Implications for PRoT:**

**Scenario: Network capsule processing large packet**
```
Time →
Network capsule:  ████████████████████ (processing, must yield)
Crypto capsule:   ────────────────────█ (waiting for network to yield)
```

**Tock's approach:**
- ✅ Works well when capsules are well-behaved
- ✅ Simpler kernel implementation
- ⚠️ Long-running operation can delay time-critical tasks
- ⚠️ Relies on capsule developers to yield appropriately

**Hubris's approach:**
- ✅ High-priority task always runs when ready
- ✅ No dependency on task cooperation
- ✅ Deterministic worst-case response times
- ✅ Critical for time-sensitive cryptographic operations

**For PRoT:** Crypto attestation responses must be timely - preemptive scheduling ensures this even under load

---

## A6: License Implications

**Both Commercial-Friendly:**

**MPL 2.0 (Hubris):**
- ✅ Use commercially
- ✅ Mix with proprietary code
- ✅ Modified MPL files → stay MPL, must share
- ✅ Your new files → any license
- ✅ Explicit patent grant

**Apache 2.0 (Tock):**
- ✅ Use commercially  
- ✅ Mix with proprietary code
- ✅ State changes, don't need to share
- ✅ Your new files → any license
- ✅ Explicit patent grant

**Bottom Line:** Both work for commercial OpenPRoT deployment

---

## A7: RISC-V Porting Considerations

**Why Hubris is Straightforward to Port:**

1. **Narrow Target Scope**
   - Only 32-bit microcontrollers
   - Well-defined execution model
   
2. **Rust Ecosystem Support**
   - RISC-V already well-supported
   - LLVM backend mature
   
3. **Memory Safety**
   - Rust prevents most porting bugs
   - Type system catches errors
   
4. **Simple Execution Model**
   - Privileged kernel
   - Unprivileged tasks
   - Standard MPU concepts
   
5. **Minimal Assembly**
   - Most code is portable Rust
   - Small arch-specific core
   
6. **Clear Documentation**
   - Architecture requirements specified
   - OpenPRoT partners already working on it

**Timeline estimate:** Weeks to months, not years

---

## A8: Hubris Driver Isolation Model

### Drivers as User-Space Tasks

**Traditional OS Model:**
```
┌────────────────────────────┐
│  Kernel Space              │
│  ├─ I2C driver (linked in) │
│  ├─ UART driver            │
│  └─ Ethernet driver        │
└────────────────────────────┘
All drivers share kernel privileges
```

**Hubris Model:**
```
┌────────────────────────────┐
│ I2C Driver Task            │ ← MPU Region 1: Code/Data
│ (MPU-isolated)             │   MPU Region 2: I2C MMIO
└────────────────────────────┘

┌────────────────────────────┐
│ Ethernet Driver Task       │ ← MPU Region 3: Code/Data
│ (MPU-isolated)             │   MPU Region 4: Ethernet MAC MMIO
└────────────────────────────┘

Each driver is isolated and has explicit peripheral access
```

### Hardware Access Control via MPU

**Configuration in app.toml:**

```toml
[tasks.i2c_driver]
memory = "16KB"
peripherals = ["I2C1"]  # Maps to I2C1 MMIO region
# MPU enforces: can ONLY access I2C1 registers

[tasks.eth_driver]
memory = "32KB"
peripherals = ["EMAC"]  # Maps to Ethernet MAC MMIO region
# MPU enforces: can ONLY access EMAC registers
```

**MPU Region Allocation:**
- Drivers are just tasks (no kernel privileges)
- Each peripheral consumes one MPU region
- Sometimes adjacent peripherals can merge
- Explicit, compile-time hardware permissions

### Security Properties

**Hardware isolation prevents:**
- ❌ I2C driver misconfiguring SPI controller
- ❌ Network driver disabling crypto hardware
- ❌ Buggy UART driver corrupting system clocks
- ❌ Compromised peripheral driver accessing other hardware

**Example: I2C Driver Bug**
```
I2C driver tries to write to Ethernet MAC registers:
  → MPU violation (hardware fault)
  → Kernel notifies jefe
  → Jefe restarts I2C driver
  → Ethernet driver unaffected
  → Other peripherals protected
```

### Region Usage Example

**STM32H7 (8 MPU regions available):**
```
Region 0: Kernel code
Region 1: Kernel data
Region 2: Task A code/data
Region 3: Task A peripheral (I2C1)
Region 4: Task B code/data
Region 5: Task B peripheral (SPI1)
Region 6: Task C code/data
Region 7: Task C peripheral (UART4)
```

### Why This Matters for PRoT

**Platform Root of Trust requires:**
- ✅ Crypto peripheral access restricted to crypto task only
- ✅ Network driver cannot read crypto keys from hardware
- ✅ Compromised peripheral driver contained
- ✅ Clear audit trail of which task accesses what hardware

**Hubris's user-space driver model provides:**
- Hardware-enforced peripheral isolation
- Compile-time hardware access validation
- Runtime protection against hardware misconfiguration
- Component-level fault recovery for driver failures

**This is unique to Hubris** - most embedded OSes link drivers into kernel space with full hardware access

