# Rust Documentation Style & Quality Guide

> **Purpose**: Ensure every line of library‑facing Rust code shipped in this repository is accompanied by clear, consistent, and compiler‑verified documentation that compiles as a doctest. This guide is written for both humans and LLMs that auto‑generate doc comments. *Follow it verbatim unless a more specific team style overrides it.*

---

## 1  Philosophy

- **Accuracy over marketing** – describe *what* the code does and the guarantees it provides.
- **Minimum surprise** – examples compile and run, panics are explicit, and `unsafe` is justified.
- **Empathy for the reader** – assume they have the item’s signature but *not* your internal context.

---

## 2  Doc Comment Syntax Basics

| Goal      | Example                            |
| --------- | ---------------------------------- |
| Line doc  | `/// Parses a UTF‑8 string into …` |
| Block doc | `/** Module‑level summary */`      |
| Inner doc | `//! Crate‑level overview`         |

- Always use `///` for items; `//!` only at the top of modules or crates.
- Place docs **immediately above** the item.
- Never exceed 80 visible columns; wrap prose at word boundaries.

---

## 3  Required Coverage

| Item kind                   | Doc coverage                                      |
| --------------------------- | ------------------------------------------------- |
| Public crates/modules       | Mandatory overview & examples                     |
| Public structs/enums/traits | Summary ✚ examples, field list docs               |
| Public functions/impl items | Summary ✚ details ✚ panics/errors/safety sections |
| Macro & proc‑macro          | Complete usage docs                               |
| Private code                | Optional but encouraged if complex                |

CI forbids `#[allow(missing_docs)]` anywhere in the tree.

---

## 4  Standard Doc Block Layout

````text
Summary sentence.

Detailed description paragraph(s).

# Examples
```rust
use crate::Foo;

let foo = Foo::new();
assert_eq!(foo.bar(), 42);
````

# Panics

This function panics if …

# Errors

Returns [`Error::Io`] when …

# Safety

Safety invariants and caller obligations.

# Performance

Big‑O notes or allocation behaviour.

# Feature flags

Enabled with `feature = "fast"`.

# See also

[`Foo::bar`] · [`crate::utils`]

````
*Heading order is fixed.* Omit sections that do not apply.

---

## 5  Markdown Style Guide
* Write in **American English**, third‑person present tense.
* Use ATX headings (`#`, `##`, …) – one space after the `#`.
* Inline code with backticks, multi‑line code with fenced blocks: ```rust.
* Prefix hidden lines in examples with `# ` so the doctest compiles silently.
* Prefer lists to paragraphs for step‑wise instructions.
* Use auto‑linking: [`String`], [`crate::module::Item`].
* One blank line between paragraphs; no trailing whitespace.

---

## 6  Example & Doctest Rules
1. **Compile‑by‑default** – every example must compile under `cargo test --doc`.
2. Write *minimal* examples (≤ 15 visible lines) that still demonstrate idiomatic use.
3. Use `no_run` only for side‑effectful I/O; use `ignore` **sparingly** and always explain why.
4. Prefer `assert_eq!` or `assert!` to println for output verification.
5. Show both import path and construction where helpful.
6. Hide boilerplate with `#` comments instead of omitting context.

---

## 7  Linting & CI
Add this to the workspace root `lib.rs` (or each crate lib):
```rust
#![deny(rustdoc::all)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(clippy::doc_markdown)]
#![deny(clippy::pedantic)]
#![deny(missing_docs)]
#![deny(broken_intra_doc_links)]
````

CI must run:

```bash
cargo clippy --no-deps --all-targets --all-features -D warnings
cargo test --all-features --doc
cargo doc --no-deps -Drustdoc::all
```

Failures gate the merge.

---

## 8  Module‑ & Crate‑Level Docs

- Use `//!` at the top of the file.
- Start with a one‑sentence summary, then extended overview.
- Provide a **quick‑start** example and a *table of contents* linking to major items.
- Explain overall design decisions and feature flags.

---

## 9  Unsafe Code & FFI

- Every `unsafe fn` **must** have a `# Safety` section documenting invariants the caller must uphold.
- For FFI, document memory layout guarantees and ownership of pointers.

---

## 10  Deprecation & Stability

- Mark deprecated items with `#[deprecated(note = "use Foo::bar instead", since = "1.2.0")]` and add a **Deprecated** heading describing migration.
- Experimental APIs: add `# Stability` heading noting they may change.

---

## 11  Error Types

- Document error enums exhaustively: each variant gets its own line comment.
- State when to expect which variant.
- Provide conversion guidance (`From<io::Error>` etc.).

---

## 12  Template for LLM‑Generated Docs

````text
/// <Summary (≤ 120 chars)>
///
/// <Detailed description>
///
/// # Examples
/// ```rust
/// <compilable example>
/// ```
///
/// # Panics
/// <if any>
///
/// # Errors
/// <if any>
///
/// # Safety
/// <if any>
````

Always emit the sections in this order, omitting those not applicable.

---

## 13  Naming & Grammar Checks

- Pass `codespell` and `cargo spellcheck --code`.
- No contractions ("can't" → "cannot").
- Numbers → words up to ten.

---

## 14  Tools Configuration Snippets

`rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
comment_width = 80
normalize_doc_attributes = true
format_code_in_doc_comments = true
```

`cargo.toml` (workspace root):

```toml
[workspace.metadata.docs.rs]
all-features = true
```

---

## 15  Versioning & Changelog Hooks

- Each PR must update `CHANGELOG.md` and reference item names added or altered.
- Link to docs.rs for rendered changes.

---

## 16  Common Anti‑Patterns

- *"This function does X"* → Summary already implies *this function*; drop the phrase.
- Examples that do not import the item and rely on prelude – be explicit.
- Screenshots or ASCII art in API docs – move to `book/` instead.
