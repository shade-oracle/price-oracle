# AI-Powered NEAR Smart Contract Migration Prompt Guide

This guide provides step-by-step prompts you can use with AI assistants (like Claude, ChatGPT, etc.) to migrate your Near smart contracts from SDK 4.x to 5.x. Each prompt is designed to tackle specific migration challenges systematically.

## Prerequisites

Before starting, ensure you have:
- A Near smart contract using SDK 4.x
- Access to an AI assistant
- Basic understanding of Rust and Near Protocol
- The [migration.md](./migration.md) document as reference

## Phase 1: Initial Assessment and Setup

### Prompt 1: Project Analysis
```
I want to migrate my Near smart contract from SDK 4.x to 5.x. Please analyze my project structure and identify:

1. Current SDK version and Rust edition
2. All dependencies that need updating
3. Potential breaking changes based on my codebase
4. Testing framework being used

Here's my Cargo.toml:
[PASTE YOUR CARGO.TOML CONTENT]

And here's my main contract file:
[PASTE YOUR MAIN CONTRACT FILE CONTENT]

Please provide a migration plan with prioritized steps.
```

### Prompt 2: Dependency Updates
```
Based on my current Cargo.toml, please update it for near-sdk 5.17.0. I need:

1. Updated dependencies for the main contract
2. Proper dev-dependencies for testing with near-workspaces
3. Correct feature flags for compilation

Current Cargo.toml:
[PASTE YOUR CURRENT CARGO.TOML]

Please provide the updated version with explanations for each change.
```

## Phase 2: Core SDK Migration

### Prompt 3: Import and Type Updates
```
I'm migrating from near-sdk 4.x to 5.x. Please update my imports and type declarations:

1. Replace Balance with NearToken
2. Update Gas API usage
3. Fix any deprecated imports
4. Update constant declarations

Current code:
[PASTE YOUR CURRENT IMPORTS AND TYPE DECLARATIONS]

Please show me the updated code with explanations for each change.
```

### Prompt 4: Macro Migration
```
Please help me migrate my contract macros from near-sdk 4.x to 5.x:

1. Replace #[near_bindgen] with the new macro syntax
2. Update struct and impl block declarations
3. Handle any PanicOnDefault issues
4. Add proper Default implementation if needed

Current contract structure:
[PASTE YOUR CONTRACT STRUCT AND IMPL BLOCKS]

Please provide the updated code with the new macro syntax.
```

### Prompt 5: Serialization Updates
```
I need to update my serialization for near-sdk 5.x. Please help me:

1. Remove custom serialization modules (u64_dec_format, u128_dec_format)
2. Update struct field serialization
3. Remove unnecessary JsonSchema derives
4. Fix any serde attributes

Current serialization code:
[PASTE YOUR CURRENT SERIALIZATION CODE]

Please show me the updated version and explain what changed.
```

## Phase 3: Function and Logic Updates

### Prompt 6: External Contract Calls
```
I need to update my external contract calls for near-sdk 5.x. Please help me migrate:

1. ext_contract macro usage
2. Promise API calls
3. Function call syntax
4. Parameter serialization

Current external call code:
[PASTE YOUR CURRENT EXTERNAL CALL CODE]

Please provide the updated code using the new Promise API.
```

### Prompt 7: Gas and Balance Operations
```
Please help me fix gas and balance arithmetic for near-sdk 5.x:

1. Update gas calculations using new Gas API
2. Fix balance operations with NearToken
3. Update any arithmetic operations
4. Handle storage cost calculations

Current gas/balance code:
[PASTE YOUR CURRENT GAS/BALANCE CODE]

Please show me the corrected version with explanations.
```

### Prompt 8: Versioned Structs and Migration
```
I have versioned structs for state migration. Please help me:

1. Add Clone derives where needed
2. Implement From traits for versioned structs
3. Update internal getter/setter methods
4. Handle any compilation errors

Current versioned struct code:
[PASTE YOUR VERSIONED STRUCT CODE]

Please provide the updated implementation.
```

## Phase 4: Testing Migration

### Prompt 9: Test Framework Setup
```
I want to migrate from near-sdk-sim to near-workspaces for testing. Please help me:

1. Set up proper test dependencies in Cargo.toml
2. Create a basic test structure
3. Set up test environment with accounts
4. Handle any compilation issues

Current test setup:
[PASTE YOUR CURRENT TEST CODE OR DESCRIBE YOUR TESTING APPROACH]

Please provide a complete near-workspaces test setup.
```

### Prompt 10: Test Data and Permissions
```
I'm having issues with my near-workspaces tests. Please help me fix:

1. JSON serialization errors (especially with large numbers)
2. Account permission issues
3. Function call syntax in tests
4. Assertion and error handling

Current test code with errors:
[PASTE YOUR CURRENT TEST CODE AND ERROR MESSAGES]

Please provide the corrected test code with explanations.
```

## Phase 5: Compilation and Error Resolution

### Prompt 11: Compilation Errors
```
I'm getting compilation errors during migration. Please help me resolve:

[PASTE YOUR COMPILATION ERROR MESSAGES]

My current code:
[PASTE THE RELEVANT CODE SECTIONS]

Please provide the fixes and explain what was wrong.
```

### Prompt 12: Runtime Errors
```
My migrated contract is running but I'm getting runtime errors. Please help me debug:

[PASTE YOUR RUNTIME ERROR MESSAGES]

Relevant code sections:
[PASTE THE RELEVANT CODE]

Please help me identify and fix the issues.
```

## Phase 6: Advanced Features

### Prompt 13: Custom Logic Migration
```
I have custom business logic that might be affected by the SDK migration. Please help me:

1. Review my custom functions for compatibility
2. Update any low-level operations
3. Handle any deprecated functionality
4. Optimize for the new SDK features

My custom logic:
[PASTE YOUR CUSTOM BUSINESS LOGIC CODE]

Please review and suggest updates for near-sdk 5.x compatibility.
```

### Prompt 14: Performance Optimization
```
Now that my contract is migrated, please help me optimize it for near-sdk 5.x:

1. Review gas usage patterns
2. Optimize storage operations
3. Improve serialization efficiency
4. Leverage new SDK features

My migrated contract:
[PASTE YOUR MIGRATED CONTRACT CODE]

Please suggest optimizations and improvements.
```

## Phase 7: Final Validation

### Prompt 15: Comprehensive Testing
```
Please help me create comprehensive tests for my migrated contract:

1. Unit tests for all functions
2. Integration tests with near-workspaces
3. Edge case testing
4. Performance testing

My contract functions:
[LIST YOUR CONTRACT FUNCTIONS OR PASTE CONTRACT INTERFACE]

Please provide a complete test suite.
```

### Prompt 16: Migration Validation
```
Please help me validate that my migration is complete and correct:

1. Compare functionality with original contract
2. Verify all features work as expected
3. Check for any missing migrations
4. Ensure best practices are followed

My migrated contract:
[PASTE YOUR COMPLETE MIGRATED CONTRACT]

Please provide a validation checklist and any remaining issues.
```

## Troubleshooting Prompts

### Common Error: "Use cargo near build"
```
I'm getting the error "Use cargo near build instead of cargo build". Please help me:

1. Install cargo-near
2. Set up proper build configuration
3. Handle any build issues

Error message:
[PASTE THE EXACT ERROR MESSAGE]

Please provide step-by-step instructions to resolve this.
```

### Common Error: "JsonSchema trait not satisfied"
```
I'm getting JsonSchema compilation errors. Please help me:

1. Remove unnecessary JsonSchema derives
2. Use proper near-sdk 5.x ABI generation
3. Fix any remaining schema issues

Error messages:
[PASTE THE ERROR MESSAGES]

My struct definitions:
[PASTE YOUR STRUCT DEFINITIONS]

Please show me the corrected code.
```

### Common Error: "TempDir::keep() method not found"
```
I'm getting TempDir errors in my tests. Please help me:

1. Fix near-workspaces version issues
2. Update test dependencies
3. Resolve any compatibility problems

Error message:
[PASTE THE ERROR MESSAGE]

My test dependencies:
[PASTE YOUR DEV-DEPENDENCIES FROM CARGO.TOML]

Please provide the fix.
```

## Best Practices Prompts

### Prompt: Code Review
```
Please review my migrated contract for best practices:

1. Security considerations
2. Gas optimization
3. Code organization
4. Documentation
5. Error handling

My migrated contract:
[PASTE YOUR COMPLETE CONTRACT CODE]

Please provide feedback and suggestions for improvement.
```

### Prompt: Documentation
```
Please help me create proper documentation for my migrated contract:

1. Function documentation
2. Usage examples
3. Migration notes
4. Testing instructions

My contract interface:
[PASTE YOUR CONTRACT'S PUBLIC FUNCTIONS]

Please provide comprehensive documentation.
```

## Usage Tips

1. **Be Specific**: Always include your current code and error messages
2. **One Issue at a Time**: Focus on one migration aspect per prompt
3. **Provide Context**: Include relevant parts of your Cargo.toml and dependencies
4. **Test Incrementally**: Test each change before moving to the next
5. **Use the Migration Guide**: Reference the migration.md document for detailed explanations

## Example Workflow

1. Start with **Prompt 1** to assess your project
2. Use **Prompt 2** to update dependencies
3. Work through **Prompts 3-8** for core migration
4. Use **Prompts 9-10** for testing setup
5. Apply **Prompts 11-12** to fix any errors
6. Finish with **Prompts 15-16** for validation

## Emergency Prompts

If you get stuck, try these:

```
I'm completely stuck with my migration. Here's my situation:
- Current SDK version: [YOUR VERSION]
- Main error: [YOUR MAIN ERROR]
- What I've tried: [WHAT YOU'VE ATTEMPTED]
- Current code: [RELEVANT CODE SECTIONS]

Please provide a step-by-step recovery plan.
```

```
My contract was working before migration but now it's broken. Please help me:
1. Identify what went wrong
2. Provide a minimal fix
3. Suggest a safer migration approach

Original working code: [PASTE ORIGINAL CODE]
Current broken code: [PASTE CURRENT CODE]
Error messages: [PASTE ERRORS]
```

---

**Remember**: This prompt guide works best when combined with the detailed [migration.md](./migration.md) document. Use the prompts to guide your AI assistant, and reference the migration guide for detailed explanations and examples.
