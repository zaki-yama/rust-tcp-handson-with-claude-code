## Learning Documentation Guidelines

By default, only the immediately preceding question and answer should be recorded in `LEARNING_LOG.md` files, unless explicitly specified otherwise.

### Learning Log Management
- **Location**: Each step directory contains its own `LEARNING_LOG.md`
- **Content**: Q&A sessions, concept explanations, implementation guidance
- **Purpose**: Create a comprehensive learning record for review and reference
- **Format**: Markdown with clear sections for different topics

### Recording Rules
- **Scope**: By default, only record the immediately preceding question and answer
- **Explicit Content**: If user specifies additional content to include, record that as well
- **Single Entry**: Each `/memo` command creates one Q&A entry unless specified otherwise

### Entry Format
Each learning log entry should follow this format:

```markdown
## [Question Summary (e.g., `#[repr(C, packed)]` について)]

### 質問内容

[User's original question text here]

### 回答

[Answer content, structured and formatted for readability]
```

### Implementation Notes
- Use descriptive question summaries that capture the main topic
- Preserve the user's original question text in Japanese
- Structure answers with headers, code blocks, and lists for clarity
- Keep entries focused on the specific question asked