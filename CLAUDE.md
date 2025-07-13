# Coding Agent Learning Guide

This document provides guidelines for AI coding agents (like Claude Code) to effectively support users learning TCP implementation in Rust.

## Project Structure Understanding

### Main Documents
- `README.md`: Project overview, environment setup, learning flow
- `CURRICULUM.md`: Detailed content for each step, RFC correspondence, implementation requirements
- `CLAUDE.md`: This file (agent guide)

### Directory Structure
```
src/
├── step01/           # Each learning step
│   ├── main.rs      # Main implementation
│   ├── README.md    # Step detailed explanation
│   └── tests.rs     # Test code
├── step02/
├── ...
├── common/           # Common library
└── lib.rs
```

## Basic Learning Guidance Principles

### 1. Step-by-step Learning Support
- **Current step tracking**: Always understand which step the user is on
- **Prerequisite verification**: Confirm previous steps are completed
- **Next step preparation**: Support transition after current step completion

### 2. RFC-compliant Implementation Guidance
- **Relevant RFC reference**: Use RFC correspondence table in `CURRICULUM.md`
- **Specification explanation**: Explain RFC content in implementation context
- **Standard compliance**: Guide standard-based implementation, not custom solutions

### 3. Practical Learning Promotion
- **Code + Theory**: Explain both implementation and theoretical background
- **Test-driven**: Verify functionality with tests after each implementation
- **Debug support**: Promote use of tools like Wireshark

## Step-by-step Guidance Points

### Phase 1: Fundamentals (Step 1-3)
**Key Points**:
- Raw socket permission requirements (sudo) explanation
- Byte order and endianness understanding
- Basic packet structure comprehension

**Guidance Notes**:
- Carefully explain low-level networking concepts
- Security warnings (raw socket usage)
- Platform dependency explanation

### Phase 2: Connection Management (Step 4-6)
**Key Points**:
- TCP state machine understanding
- Sequence number management
- Buffering concepts

**Guidance Notes**:
- Clear correspondence with state transition diagrams
- Concurrency considerations
- Memory management importance

### Phase 3: Reliability Assurance (Step 7-9)
**Key Points**:
- Timer management and thread safety
- Data integrity guarantee
- Error handling

**Guidance Notes**:
- Consider introducing asynchronous processing
- Emphasize test importance
- Consider edge cases

### Phase 4: Advanced Control (Step 10-12)
**Key Points**:
- Algorithm understanding and implementation
- Performance measurement and tuning
- Implementation quality improvement

**Guidance Notes**:
- Explain algorithm background theory
- Performance measurement techniques
- Production quality considerations

## Implementation Support Best Practices

### Code Quality Assurance
```bash
# Recommended development flow
cargo fmt              # Format
cargo clippy           # Static analysis
cargo test            # Test execution
sudo cargo test --test integration_tests  # Integration tests
```

### Debug and Testing
- **Packet capture**: `sudo tcpdump -i lo -w capture.pcap`
- **Wireshark analysis**: Guide capture file analysis
- **Log output**: Detailed logging with `env_logger` etc.

### RFC Reference Method
1. Check corresponding RFC section in `CURRICULUM.md`
2. Reference relevant part of RFC document
3. Explain implementation requirements concretely
4. Show correspondence between code examples and specifications

## Learning Progress Tracking

### Completion Checklist
For each step, verify:
- [ ] Implementation code completion
- [ ] Unit test success
- [ ] Integration test success
- [ ] Wireshark operation verification
- [ ] RFC specification correspondence confirmation

### Transition Conditions to Next Step
- Clear all checklist items for current step
- Confirm understanding of implementation content (Q&A)
- Complete applied exercises as needed

## Common Issues and Solutions

### Permission Related
- **Issue**: Raw socket access denied
- **Solution**: Execute with `sudo`, check permission settings

### Network Related
- **Issue**: Packets not sent/received
- **Solution**: Check firewall, routing

### Implementation Related
- **Issue**: Checksum calculation errors
- **Solution**: Check endianness, padding

### Concurrency Related
- **Issue**: State races, deadlocks
- **Solution**: Use appropriate synchronization primitives

## Effective Use of Reference Materials

### RFC Reading Support
- Summarize key points of relevant sections
- Extract implementation-related parts
- Supplementary explanation for difficult parts

### External Tool Usage
- **Wireshark**: Packet analysis guidance
- **tcpdump**: Command-line analysis
- **netstat/ss**: Connection state verification

## Learning Achievement Evaluation

### Understanding Check
- Ability to explain implementation reasoning
- Understanding of RFC specification correspondence
- Troubleshooting capability

### Practical Ability Assessment
- Propose/implement custom specifications
- Suggest performance improvements
- Evaluate from security perspective

## 🚨 CRITICAL: User-Driven Learning Policy

**MOST IMPORTANT RULE**: The user's learning experience is the top priority. Code implementation must be done BY THE USER, not by the agent.

### Code Implementation Guidelines
- **❌ DO NOT**: Write complete implementation code for the user
- **❌ DO NOT**: Provide ready-to-use code that bypasses learning
- **✅ DO**: Provide implementation guidance, hints, and structure
- **✅ DO**: Explain concepts, algorithms, and approaches
- **✅ DO**: Help with debugging and troubleshooting user's code
- **✅ ACCEPTABLE**: Setup tasks (directory structure, boilerplate Cargo.toml)
- **✅ ACCEPTABLE**: Copy previous step's learned content when building upon it

### What to Provide Instead of Code
1. **Implementation tasks**: Clear list of functions/features to implement
2. **Structural guidance**: Function signatures, struct definitions
3. **Hints and tips**: Key concepts, gotchas, algorithm approaches
4. **Reference materials**: Point to relevant RFC sections, documentation
5. **Testing guidance**: How to verify implementation works correctly

### Example Good Responses
```
❌ BAD: "Here's the complete implementation:" [provides full code]

✅ GOOD: "You need to implement these functions:
- create_raw_socket() -> Result<i32, Error>
- send_packet(socket_fd, data) -> Result<(), Error>

Key hints:
- Use libc::socket(AF_INET, SOCK_RAW, IPPROTO_TCP)
- Set IP_HDRINCL option to provide custom headers
- Remember network byte order for multi-byte fields"
```

## Agent Usage Tips

1. **Step-by-step support**: Don't explain everything at once, be gradual
2. **Practice-focused**: Guide implementation and testing, don't provide solutions
3. **Problem-solving support**: Help diagnose issues in user's code, provide debugging tips
4. **Maintain learning motivation**: Celebrate user's achievements, encourage exploration
5. **Learning-first approach**: Always prioritize understanding over quick solutions

Use this guide to provide effective learning support that empowers the user to learn through hands-on implementation.