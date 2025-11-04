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

### Two Best-in-Class Rust Embedded Operating Systems

**Hubris** (Oxide Computer Company)
- Microkernel for server management
- Static task model
- MPU-enforced isolation

**Tock** (Stanford/MIT/Academia)
- General-purpose embedded OS
- Dynamic application loading
- Capsule-based architecture

> Both are production-grade, both use Rust—but fundamentally different philosophies

---

## Round 1: Memory Protection

### How Do We Prevent One Bug From Destroying Everything?

**Hubris: Hardware-Enforced Boundaries**
- ✅ Drivers in separate MPU-protected memory spaces
- ✅ Kernel physically isolated from tasks
- ✅ Failing driver **cannot** corrupt kernel

**Tock: Software-Based Isolation**
- ⚠️ Drivers (capsules) share kernel memory space
- ⚠️ Isolation through Rust's type system
- ⚠️ Relies on compile-time checks

**Winner: Hubris** — Hardware protection provides defense-in-depth

---

## Round 2: When Things Go Wrong

### Can We Recover Without Rebooting the Entire System?

**Hubris: Component-Level Recovery**
- ✅ Supervisor can restart individual crashed tasks
- ✅ In-place reinitialization
- ✅ Memory isolation limits "blast radius"
- ✅ No system-wide reboot needed

**Tock: Process Recovery**
- ✅ Can restart user processes
- ⚠️ Kernel capsule failures more problematic
- ⚠️ Shared kernel space complicates isolation

**Winner: Hubris** — Restart the broken part, not everything

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
- ⚠️ Tasks loaded dynamically
- ⚠️ Resource allocation at runtime
- ⚠️ More surface area for failures

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
- ✅ No race conditions
- ✅ Precise fault isolation (REPLY_FAULT)
- ✅ Direct memory copy (zero-copy)
- ✅ Extends Rust ownership across tasks

**Tock: Asynchronous Callbacks**
- ⚠️ More complex kernel message queues
- ⚠️ Potential for race conditions

---

## Round 5: Attack Surface

### What Can Go Wrong?

**Hubris: Minimal by Design**
- ✅ No dynamic memory allocation
- ✅ No task creation/destruction at runtime
- ✅ No runtime resource management
- ✅ Application-specific kernel (dead code eliminated)

**Tock: Flexible but Broader**
- ⚠️ Dynamic application loading
- ⚠️ Grant-based allocation system
- ⚠️ General-purpose kernel (includes unused features)

**Winner: Hubris** — Less code = fewer vulnerabilities

---

## Round 6: Debugging Without Vulnerabilities

### How Do We Debug Without Creating Security Holes?

**Hubris: Kernel-Aware Debugger (Humility)**
- ✅ **NO** console interfaces in application
- ✅ **NO** printf formatting code
- ✅ **NO** command parsing vulnerabilities
- ✅ External debugger handles everything
- ✅ Full core dumps for post-mortem analysis

**Tock: Traditional Console**
- ⚠️ UART/USB console interfaces
- ⚠️ In-application command parsing
- ⚠️ Printf-style formatting = attack surface

---

## Visual Comparison: Architecture Philosophy

```
Hubris Philosophy: Eliminate Uncertainty
┌─────────────────────────────────────┐
│  Build Time: Validate Everything    │
│  Runtime: Execute Only              │
│  Failure: Impossible by Construction│
└─────────────────────────────────────┘

Tock Philosophy: Enable Flexibility
┌─────────────────────────────────────┐
│  Build Time: Prepare Framework      │
│  Runtime: Adapt and Allocate        │
│  Failure: Handle Gracefully         │
└─────────────────────────────────────┘
```

**For PRoT: We choose "cannot fail" over "handle failure"**

---

## Key Differentiators Summary

| Critical Feature | Hubris | Tock |
|-----------------|--------|------|
| **Memory Isolation** | Hardware (MPU) ✅ | Software (Rust) ⚠️ |
| **Fault Recovery** | Component-level ✅ | Process-level ⚠️ |
| **Composition** | Static ✅ | Dynamic ⚠️ |
| **Resource Allocation** | Compile-time ✅ | Runtime ⚠️ |
| **Scheduling** | Preemptive ✅ | Cooperative ⚠️ |
| **Debug Security** | External debugger ✅ | Console interfaces ⚠️ |

---

## The "But What About..." Slide

**Q: Doesn't Tock have production deployments in security systems?**  
A: Yes! Tock is excellent engineering. Different philosophy, different trade-offs.

**Q: What about RISC-V support?**  
A: Hubris designed with RISC-V in mind. Straightforward port (narrow scope, simple execution model, minimal assembly).

**Q: Isn't static composition too restrictive?**  
A: Not for PRoT. We know exactly what we need at build time. Flexibility adds risk without benefit.

**Q: What about the MPL 2.0 license?**  
A: Commercial use allowed. Modified MPL files must remain MPL and be shared. Works fine with proprietary code.

---

## Real-World Implications

### Scenario: Driver Crashes During Boot

**Hubris Response:**
1. 🛡️ Kernel detects fault (MPU violation)
2. 📞 Notifies supervisor task
3. 🔄 Supervisor restarts just that driver
4. ✅ System continues booting
5. ⏱️ Total impact: milliseconds

**Why This Matters:**
- Remote data center deployment
- No physical access to hardware
- Cannot afford full system restart
- Other components stay operational

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
- ✅ Research platforms
- ✅ Educational systems
- ✅ Applications requiring runtime flexibility
- ✅ Multi-tenant embedded systems
- ✅ Diverse application scenarios

**Hubris is Optimal For:**
- ✅ Server management infrastructure
- ✅ Platform root of trust
- ✅ Security-critical embedded systems
- ✅ Known-at-build-time requirements
- ✅ "Cannot fail" architectures

---

## Our Recommendation

### **Hubris is the Right Choice for OpenPRoT**

**Not because Tock is inferior—but because:**

1. 🎯 **Architectural alignment** — Static model matches PRoT requirements
2. 🛡️ **Defense in depth** — Hardware isolation + software safety
3. 📐 **Predictability** — Compile-time validation eliminates runtime unknowns
4. 🔄 **Fault containment** — Component recovery without system reboot
5. 🎪 **Simplicity** — Fewer moving parts = fewer failure modes

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
- ✅ Both OSes are excellent engineering
- ✅ Different philosophies serve different needs
- ✅ For PRoT: Static > Dynamic, Hardware > Software, Prevention > Recovery
- ✅ Hubris architecture aligns with "cannot fail" requirements

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

**Tock Cooperative Scheduling:**
```
Kernel:      ████████████████████
Process A:   ──────██████────────
Process B:   ────────────████────

Tasks yield control
Round-robin user processes
Kernel must cooperate
```

**For PRoT:** Crypto operations must preempt logging

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

