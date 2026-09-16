# Phiber — Zed Settings Integration Reference

> **Location**: `crates/phiber/docs/settings.md`
> Maps Phiber Settings Modal tabs to actual Zed settings keys (`settings_store.rs`).

---

## Settings System Architecture

```
Zed Settings Pipeline:
  assets/settings/default.json     <- default values
  ~/.config/zed/settings.json      <- user overrides
        |
        v
  SettingsStore (GPUI global)
        |
        v
  Settings::get_global(cx)         <- typed access
```

Phiber wraps this with:
```
store/modals/settings/def.rs       <- BSettingsType, static rows
state/settings.rs (proposed)       <- runtime active values
bui/part/settings_view.rs          <- view builder for modal content
```

---

## Full Settings Key Reference by Tab

### Tab 1 — Interface

| Phiber Row | Zed Settings Key | Type | Default | Options |
|------------|------------------|------|---------|---------|
| Display | `ui_font_family` | string | `"Zed Plex Sans"` | any system font |
| Resolution | `ui_scale` | float | `1.0` | 0.5 – 3.0 |
| Scale | `buffer_font_size` | int | `16` | 6 – 100 |
| Language | `language` | string | `"en"` | locale codes |

**Additional Interface Keys:**
```jsonc
"ui_font_family": "Inter",
"ui_font_size": 14,
"ui_font_fallbacks": [],
"show_completions_on_input": true,
"show_completion_documentation": true,
"completion_documentation_secondary_query_debounce": 300,
"preferred_line_length": 80,
"cursor_blink": true,
"cursor_shape": "bar",              // bar | block | underline | hollow
"show_inline_completions": true,
"toolbar": {
  "breadcrumbs": true,
  "quick_actions": true,
  "selections_menu": true
},
"scrollbar": {
  "show": "auto",                   // auto | system | always | never
  "cursors": true,
  "git_diff": true,
  "search_results": true,
  "selected_symbol": true,
  "diagnostics": "all"
},
"project_panel": {
  "button": true,
  "dock": "left",
  "file_icons": true,
  "folder_icons": true,
  "git_status": true,
  "indent_size": 20,
  "auto_reveal_entries": true
},
```

---

### Tab 2 — Editing

| Phiber Row | Zed Settings Key | Type | Default | Options |
|------------|------------------|------|---------|---------|
| Auto-Save | `autosave` | string | `"off"` | off, after_delay, on_focus_change, on_window_change |
| Undo Steps | `undo_depth` | int | `32` | 1-1000 |
| Snap | `tab_size` | int | `4` | 1-8 |

**Additional Editing Keys:**
```jsonc
"autosave": "on_focus_change",
"auto_indent": "always",           // none | editorconfig | always
"auto_close_brackets": "always",   // never | always | language_default
"tab_size": 4,
"hard_tabs": false,
"soft_wrap": "prefer_line",        // none | editor_width | preferred_line_length | bounded
"show_whitespace": "selection",    // all | selection | none
"indent_guides": {
  "enabled": true,
  "line_width": 1,
  "active_line_width": 1,
  "coloring": "fixed",             // fixed | indent_aware
  "background_coloring": "disabled"
},
"format_on_save": "on",
"remove_trailing_whitespace_on_save": true,
"ensure_final_newline_on_save": true,
"use_on_type_format": true,
"inlay_hints": {
  "enabled": false,
  "show_type_hints": true,
  "show_parameter_hints": true,
  "show_other_hints": true
},
"linked_edits": true,
"code_actions_on_format": {},
```

---

### Tab 3 — Navigation

| Phiber Row | Zed Settings Key | Type | Default | Options |
|------------|------------------|------|---------|---------|
| Orbit | `vim_mode` | bool | `false` | true/false |
| Zoom | `scroll_sensitivity` | float | `1.0` | 0.1-10.0 |
| Walk | `relative_line_numbers` | bool | `false` | true/false |

**Additional Navigation Keys:**
```jsonc
"vim_mode": false,
"vim": {
  "use_multiline_find": false,
  "use_smartcase_find": false,
  "custom_digraphs": {}
},
"relative_line_numbers": false,
"gutter": {
  "line_numbers": true,
  "code_actions": true,
  "runnables": true,
  "folds": true,
  "breakpoints": true
},
"scroll_sensitivity": 1.0,
"vertical_scroll_margin": 3,
"horizontal_scroll_margin": 3.0,
"multi_cursor_modifier": "alt",    // alt | cmd_or_ctrl
"click_to_multiedit": "alt",
"jump_label": { "enabled": false },
```

---

### Tab 4 — Add-ons / Extensions

| Phiber Row | Zed Settings Key | Type | Default |
|------------|------------------|------|---------|
| Installed | (from extension registry) | — | — |
| Check Updates | `auto_update` | bool | `true` |

**Extension Settings:**
```jsonc
"auto_update": true,
"auto_update_extensions": { "html": true },
"extensions": ["html", "python", "rust"],
"load_direnv": "shell_hook",       // shell_hook | direct
```

---

### Tab 5 — Themes

| Phiber Row | Zed Settings Key | Type | Default |
|------------|------------------|------|---------|
| Active Theme | `theme` | string | `"Phisia Dark"` |
| Accent | (Phiber-only) | — | `"blue"` |
| BG Effect | (Phiber-only) | — | `"fill"` |
| Corner Radius | (Phiber-only) | — | `"md"` |
| Density | `ui_density` | string | `"default"` |
| Font Family | `ui_font_family` | string | `"Inter"` |

**Theme Settings:**
```jsonc
"theme": {
  "mode": "dark",                  // system | light | dark
  "light": "One Light",
  "dark": "Phisia Dark"
},
"icon_theme": "default",
"ui_density": "default",           // compact | default | comfortable
"buffer_font_family": "Zed Plex Mono",
"buffer_font_size": 15,
"buffer_font_weight": 400,
"buffer_font_features": { "calt": false },
```

---

### Tab 6 — System

| Phiber Row | Zed Settings Key | Type | Default |
|------------|------------------|------|---------|
| GPU | `gpu_acceleration` | string | `"on"` |
| Memory | (OS managed) | — | — |
| Sound | `notification_sounds` | bool | `true` |

**System Settings:**
```jsonc
"gpu_acceleration": "on",
"telemetry": {
  "diagnostics": true,
  "metrics": true
},
"notification_sounds": true,
"when_closing_with_no_tabs": "platform_default",
"confirm_quit": false,
"restore_on_startup": "last_workspace",  // none | last_session | last_workspace
"crash_reporter": { "prompt_before_sending": false },
```

---

## Zed Settings NOT Mapped to Phiber UI (Future)

These Zed settings exist but have no Phiber settings modal UI yet:

```jsonc
// LSP / Language
"lsp": { ... }
"languages": { "Rust": { "tab_size": 4 } }
"language_servers": [ ... ]

// Git
"git": {
  "git_gutter": "tracked_files",  // tracked_files | all | never
  "inline_blame": { "enabled": true }
}

// Terminal
"terminal": {
  "font_family": "Zed Plex Mono",
  "font_size": 15,
  "line_height": "comfortable",
  "cursor_shape": "bar",
  "shell": {}
}

// Tasks
"tasks": { "auto_show_output": "never" }

// Diagnostics
"diagnostics": { "include_warnings_in_error_summary": true }

// Collaboration
"collaboration_panel": { "button": true, "dock": "left" }
"chat_panel": { "button": "never", "dock": "right" }
"message_editor": { "auto_replace_emoji_shortcode": false }
"notification_panel": { "button": true, "dock": "right" }
```
