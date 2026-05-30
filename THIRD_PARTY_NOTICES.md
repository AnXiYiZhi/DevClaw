# Third-Party Notices

DevClaw is based on the following open-source projects and libraries.

## CC Switch (Original Project)

DevClaw is a modified fork of [CC Switch](https://github.com/farion1231/cc-switch), originally created by **Jason Young**.

- **License**: MIT License
- **Copyright**: Copyright (c) 2025 Jason Young
- **Repository**: https://github.com/farion1231/cc-switch

The original MIT License is preserved in the [LICENSE](LICENSE) file.

### Modifications

DevClaw includes the following modifications to the original CC Switch project:

- Renamed application and UI from "CC Switch" to "DevClaw"
- Added commercial license activation system
- Modified branding, icons, and visual identity
- Added Gitee release synchronization
- Custom deep link protocol (`devclaw://`)
- Various feature enhancements and bug fixes

## Key Dependencies

### Frontend

| Package | License | Author |
|---------|---------|--------|
| React | MIT | Meta Platforms |
| Vite | MIT | Evan You |
| TypeScript | Apache-2.0 | Microsoft |
| TailwindCSS | MIT | Tailwind Labs |
| Radix UI | MIT | WorkOS |
| TanStack Query | MIT | Tanner Linsley |
| react-i18next | MIT | i18next |
| Framer Motion | MIT | Framer Motion |
| shadcn/ui | MIT | shadcn |
| lucide-react | ISC | Lucide Contributors |
| @lobehub/icons-static-svg | MIT | LobeHub |
| Recharts | MIT | Recharts |
| CodeMirror | MIT | Marijn Haverbeke |
| Zod | MIT | Colin McDonnell |
| Sonner | MIT | Emil Kowalski |

### Backend (Rust / Tauri)

| Crate | License | Author |
|-------|---------|--------|
| Tauri | MIT OR Apache-2.0 | Tauri Programme Foundation |
| tokio | MIT | Tokio Contributors |
| serde | MIT OR Apache-2.0 | David Tolnay |
| reqwest | MIT OR Apache-2.0 | Sean McArthur |
| axum | MIT | Tokio Contributors |
| rusqlite | MIT | rusqlite Contributors |
| hyper | MIT | hyper Contributors |

### Build Tools

| Tool | License |
|------|---------|
| pnpm | MIT |
| Rust | MIT OR Apache-2.0 |
| create-dmg | MIT |

---

For a complete list of all dependencies and their licenses, see:

- **npm**: Run `pnpm licenses list` in the project root
- **Cargo**: See `src-tauri/Cargo.lock` and run `cargo license` for details
