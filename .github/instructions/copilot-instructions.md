## General Copilot Instructions
- Do not make sweeping changes to existing code; prefer minimal, targeted edits.
- Do not remove or alter existing comments unless explicitly instructed.
- Do not perform any actions that would change databases, servers, or external systems.
- Do not add any new dependencies or libraries without explicit approval.
- Do not change existing function signatures unless explicitly instructed.
- Do not change application code to address issues with unit tests; instead, fix the unit tests.
- Ensure all code is secure and free from vulnerabilities.
- Avoid using deprecated functions or features.
- Ensure all code adheres to PSR-12 coding standards.
- Follow best practices for rust development.
- Prioritize performance and efficiency in code suggestions.
- Focus on code clarity and maintainability.
- When refactoring, preserve all comments and code structure unless asked otherwise.
- Only modify code where requested, and avoid unnecessary changes.
- Maintain existing coding style and conventions.
- Ensure all new code is well-documented and follows project guidelines.
- Add tests for any new functionality introduced.
- When writing tests, ensure they are isolated and do not depend on external systems unless explicitly required.
- For any new functions, include input validation and error handling as per project standards.
- When updating documentation, ensure clarity and accuracy without altering the original intent.
- Preserve all existing functionality unless explicitly instructed to change it.
- When making changes, consider backward compatibility and avoid breaking existing features.
- Ensure all changes are tested and verified before finalizing.

## Software Stack
- Rust 1.70+