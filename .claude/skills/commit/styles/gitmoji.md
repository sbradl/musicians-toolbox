# Gitmoji

Subject: `<emoji> <description>` — the emoji signals the intent at a glance in the log.

- **emoji** — the Unicode character (not the `:shortcode:`), the most specific match:
  - ✨ new feature
  - 🐛 bug fix
  - 🚑️ critical hotfix
  - 🩹 simple non-critical fix
  - ♻️ refactor
  - ⚡️ performance
  - ✅ add or update tests
  - 📝 documentation
  - 🎨 code structure or formatting
  - 🔥 remove code or files
  - 🚚 move or rename files
  - 💥 breaking change
  - 🔒️ security
  - 🔧 configuration
  - ➕ / ➖ add / remove a dependency
  - ⬆️ / ⬇️ upgrade / downgrade a dependency
  - 👷 CI
  - 🚨 fix linter or compiler warnings
  - 🗑️ deprecate
  - 💄 UI or styling
  - 🌐 internationalisation
  - 🏗️ architectural change
- **description** — imperative mood, capitalised first letter, no trailing period; the whole subject within 72 characters.

```
✨ Add password reset via email

Users locked out had no self-service path and filed support tickets.

Refs: ABC-123
```
