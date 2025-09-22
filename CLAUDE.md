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


### Standard Step Directory Structure
Each step follows this consistent structure:
```
src/stepXX/
├── main.rs          # Main implementation with TODO-guided tasks
├── README.md        # Detailed step guide with phase breakdown
├── tests.rs         # Comprehensive test suite
└── LEARNING_LOG.md  # User's learning notes (created as needed)
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
- **TDD methodology**: Use Red→Green→Refactor cycles with phase-based testing
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
cargo fmt              # Format (MANDATORY after .rs file changes)
cargo clippy           # Static analysis
cargo test            # Test execution
sudo cargo test --test integration_tests  # Integration tests
```

### Mandatory Formatting Rules
- **Step initialization**: Run `cargo fmt` after creating all .rs template files
- **After any edit**: Run `cargo fmt` immediately after editing any .rs file
- **Before commits**: Ensure all code is properly formatted
- **Quality gate**: Unformatted code should not be left in the repository

### Debug and Testing
- **Packet capture**: `sudo tcpdump -i lo -w capture.pcap`
- **Wireshark analysis**: Guide capture file analysis
- **Log output**: Detailed logging with `env_logger` etc.
- **TDD workflow**: Use phase-based testing (Phase A-F) following step02 pattern
- **Test execution**: `cargo test phase_x_tests` for targeted phase testing
- **Red→Green→Refactor**: Each test should start with failure, then implementation, then improvement
- **Commented tests**: Use `/* */` blocks for tests that require implementation completion

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

### Module Recognition Related
- **Issue**: rust-analyzer shows "This file is not included anywhere in the module tree"
- **Solution**: Add `#[cfg(test)] mod tests;` to main.rs for proper test file integration
- **Issue**: Binary not recognized by Cargo
- **Solution**: Ensure Cargo.toml includes `[[bin]]` section for the step

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

### Code Modification Policy
- **❌ NEVER**: Modify user's code without explicit permission
- **❌ NEVER**: "Fix" user's implementation automatically
- **❌ NEVER**: Replace user's code with "correct" solutions
- **✅ DO**: Point out issues and explain problems
- **✅ DO**: Suggest fixes but let user implement them
- **✅ ONLY**: Modify code when user explicitly requests it

**Example Interaction:**
```
❌ BAD: [sees wrong code] → [fixes it automatically without asking]

✅ GOOD: "I notice the TCP flag should be ACK instead of SYN on line 224.
This would cause the packet to be incorrect. Would you like to fix this yourself,
or should I explain why ACK is the correct flag here?"
```

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
- Remember network byte order for multi-byte fields

Start with TDD:
cargo test phase_b_tests::test_socket_creation
This will guide your implementation step by step."
```

## Step Initialization Workflow

When starting a new step, follow this standardized process:

### 1. Analysis Phase
- **Read CURRICULUM.md**: Extract step objectives, RFC references, and requirements
- **Identify dependencies**: Check what previous steps have built (especially reusable components)
- **Break down complexity**: Divide implementation into 4-6 logical phases (A, B, C, D, E, F)

### 2. Structure Creation
- **Create directory**: `mkdir -p src/stepXX`
- **Update Cargo.toml**: Add binary target for the new step
  ```toml
  [[bin]]
  name = "stepXX"
  path = "src/stepXX/main.rs"
  ```
- **Generate README.md**: 
  - Phase-based task breakdown (15-25 specific tasks)
  - RFC references and implementation guidance
  - Code examples and hints (not complete solutions)
  - Completion checklist and testing guidance
- **Create main.rs template**:
  - Function signatures with TODO comments
  - Clear task references (e.g., "Task B1: implement X")
  - Basic imports and structure
  - Demo main() function
  - NO tests (all tests go in tests.rs)
- **Create tests.rs**:
  - TDD-structured phase-based tests (Phase A-F modules)
  - Unit tests for each implementation phase
  - Integration tests with real scenarios
  - Performance and error handling tests
  - Wireshark/debugging helpers
  - Follow step02 TDD pattern for consistency
  
  **Sample structure**:
  ```rust
  use super::*;
  use std::time::{Duration, Instant};

  // =============================================================================
  // Phase A: [Phase Description] - TDD Tests
  // =============================================================================

  #[cfg(test)]
  mod phase_a_tests {
      use super::*;

      // Task A1: [Task Description]
      #[test]
      fn test_basic_functionality() {
          // Red: 最初は失敗する（機能が未実装）
          // Green: 実装して成功させる
          // Refactor: コードを改善
      }

      #[test]
      fn test_edge_cases() {
          // 境界値やエラーケースのテスト
      }
  }

  // =============================================================================
  // Phase B-F: [Continue same pattern]
  // =============================================================================

  // Integration tests with #[ignore] for manual execution
  #[cfg(test)]
  mod integration_tests {
      use super::*;

      #[test]
      #[ignore] // ネットワーク接続が必要
      fn test_real_scenario() {
          // 実際のネットワーク通信テスト
      }
  }

  /*
  TDD実行手順:
  1. cargo test phase_a_tests::test_basic_functionality
  2. Red → Green → Refactor
  3. 次のテストに進む
  */
  ```
- **Run cargo fmt**: Format all newly created .rs files
- **Add module declaration**: Add `#[cfg(test)] mod tests;` to main.rs for proper test integration

### 3. Learning Guidance Integration
- **Reference previous steps**: Show how to reuse Step1/Step2 components
- **Phase progression**: Clear A→B→C→D→E→F implementation order
- **Estimation**: Realistic time estimates for each phase
- **Debug support**: Common issues and troubleshooting tips

### 4. Quality Standards
- **File consistency**: Follow step02 format and structure patterns
- **Task granularity**: Each task should be 15-30 minutes of focused work
- **RFC compliance**: Direct references to relevant specification sections
- **Test coverage**: Comprehensive validation including edge cases
- **Code formatting**: Always run `cargo fmt` after creating or editing .rs files

## Agent Usage Tips

1. **Step-by-step support**: Don't explain everything at once, be gradual
2. **Practice-focused**: Guide implementation and testing, don't provide solutions
3. **Problem-solving support**: Help diagnose issues in user's code, provide debugging tips
4. **Maintain learning motivation**: Celebrate user's achievements, encourage exploration
5. **Learning-first approach**: Always prioritize understanding over quick solutions
6. **Standardized setup**: Always follow the step initialization workflow when creating new steps

Use this guide to provide effective learning support that empowers the user to learn through hands-on implementation.
