# RON: Rusty Object Notation

RON stands for **Rusty Object Notation**. It's a data serialization format designed to be compatible with Rust's data structures and syntax.

Key characteristics of RON include:

1. **Human-readable and editable**: Similar to JSON but with a syntax closer to Rust
2. **Support for Rust's enum types**: Unlike JSON, RON can directly represent Rust enums
3. **Named tuples**: RON supports named fields in tuple structures
4. **Optional trailing commas**: Making editing easier and diffs cleaner
5. **Comments**: RON supports both line and block comments, unlike JSON

Here's a simple example of RON syntax from an Idol interface definition:

```ron
Interface(
    name: "LedControl",
    ops: {
        "set_led": (
            args: {
                "led_id": (type: "u8"),
                "state": (type: "bool"),
            },
            reply: "()",
        ),
        // This is a comment - RON supports these!
    },
)
```

RON was created as a more Rust-friendly alternative to formats like JSON, TOML, or YAML, particularly for configuration files and data structures that need to be both human-editable and closely aligned with Rust's type system.

In the context of Idol, RON provides a natural and readable way to define interface contracts while maintaining compatibility with Rust's type system and serde serialization framework.
Relevant links:
- [Official RON GitHub Repository](https://github.com/ron-rs/ron)
