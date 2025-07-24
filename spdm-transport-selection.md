## Key Design Elements

Static Message Routing: The SPDM Coordinator task contains a compile-time routing table that maps SPDM responders to their appropriate transport tasks. No runtime polymorphism needed.
Transport Specialization: Each transport task is highly optimized for its specific protocol and hardware. The PCIe task knows about DOE mailboxes, the MCTP task handles I2C bus arbitration, etc.
Message-Based Abstraction: The abstraction happens at the message boundary - all tasks speak the same SpdmRequest/SpdmResponse message format, but each transport implements it differently.
Build-Time Flexibility: You can include/exclude transport tasks at build time based on your hardware. If you don't need TCP transport, it gets compiled out entirely.



```mermaid
graph TB
    %% Application Layer
    App["Application Task<br/>(Attestation, Measurement)"]
    
    %% SPDM Core Logic
    SPDM["SPDM Coordinator Task<br/>• Protocol state machine<br/>• Message validation<br/>• Transport selection logic"]
    
    %% Transport-Specific Tasks
    PCIe["PCIe DOE Task<br/>• DOE mailbox handling<br/>• PCIe config space<br/>• Hardware interrupts"]
    
    MCTP["MCTP/I2C Task<br/>• MCTP packet framing<br/>• I2C bus arbitration<br/>• SMBus protocols"]
    
    TCP["TCP Task<br/>• Socket management<br/>• Network stack<br/>• TLS wrapper"]
    
    %% Hardware Abstraction
    HW1["PCIe Controller<br/>Hardware"]
    HW2["I2C Controller<br/>Hardware"]
    HW3["Ethernet MAC<br/>Hardware"]
    
    %% Message Flow
    App -->|"SpdmRequest::GetVersion<br/>{target: ResponderA}"| SPDM
    App -->|"SpdmRequest::GetMeasurements<br/>{target: ResponderB}"| SPDM
    
    %% Transport Selection (Compile-time routing table)
    SPDM -->|"Transport routing based on<br/>ResponderA → PCIe endpoint"| PCIe
    SPDM -->|"Transport routing based on<br/>ResponderB → MCTP address"| MCTP
    SPDM -.->|"Could route to TCP<br/>for ResponderC"| TCP
    
    %% Hardware Interface
    PCIe --> HW1
    MCTP --> HW2
    TCP --> HW3
    
    %% Response Flow
    PCIe -->|"SpdmResponse::Version"| SPDM
    MCTP -->|"SpdmResponse::Measurements"| SPDM
    TCP -.->|"SpdmResponse::*"| SPDM
    
    SPDM --> App
    
    %% Styling
    classDef app fill:#e1f5fe
    classDef spdm fill:#f3e5f5
    classDef transport fill:#e8f5e8
    classDef hardware fill:#fff3e0
    
    class App app
    class SPDM spdm
    class PCIe,MCTP,TCP transport
    class HW1,HW2,HW3 hardware
    
    %% Build-time configuration note
    subgraph Legend["Build-Time Configuration"]
        Note1["✓ All transport tasks included at build time"]
        Note2["✓ Routing table generated statically"]
        Note3["✓ Dead code elimination removes unused paths"]
        Note4["✓ Each transport task optimized for specific hardware"]
    end
    
    style Legend fill:#f9f9f9,stroke:#666,stroke-dasharray: 5 5
```
