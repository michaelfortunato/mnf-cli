  = MNF CLI — Roadmap

  // Core goal: add a real config file + daily note templating.
  // Style: small, composable modules; keep AppError/Result.

  == Config System

  - [ ] Add `paths::config_dir()` and `paths::config_file()`
      - macOS: prefer XDG ($XDG_CONFIG_HOME or ~/.config/mnf)
      - other OS: use dirs::config_dir() fallback to ~/.config/mnf
      - file: mnf.toml
  - [ ] Write helpers
      - [ ] load_config() -> Result<Config>
      - [ ] ensure_default_config() -> Result<()> (create dir/file if missing)
  - [ ] Add deps
      - [ ] serde, serde_derive, toml

  == Config Schema

  - [ ] Define:
      ```
        # ~/.config/mnf/mnf.toml
        [daily]
        # "tera" | "handlebars"
        template_engine = "tera"
        # Absolute path or "~" (tilde expanded)
        template_file = "~/.config/mnf/templates/daily.typ"
      ```
  - [ ] Rust structs:

      -
        ```
        #[derive(serde::Deserialize, Default)]
              struct Config { daily: Daily }

        #[derive(serde::Deserialize, Default)]
              struct Daily {
              template_engine: Option<TemplateEngine>,
              template_file: Option<std::path::PathBuf>,
                      }

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "lowercase")]
        enum TemplateEngine { Tera, Handlebars }
        ```
  == Daily Note Templating

  - [ ] Choose engine (default: tera; alt: handlebars)
      - [ ] Add deps: tera = "1" or handlebars = "5"
  - [ ] Add render_daily_note(template_src, ctx) -> Result<String>
      - Context keys: iso, weekday, year, month, day, title
  - [ ] Use create_new(true) when creating file; on first create:
      - [ ] If template_file configured, render with chosen engine
      - [ ] Else fallback header line with human date
  - [ ] Hook into commands
      - [ ] mnf daily → create/open today
      - [ ] mnf note new (alias) → same behavior
      - [ ] Optional CLI overrides (hidden in short help):
          - --template path, --engine {tera|handlebars}

  == Paths Module

  - [ ] Add:
      - [ ] config_dir(), config_file()
      - [ ] expand_tilde()
  - [ ] Keep existing:
      - [ ] notes_dir(), daily_dir(), gists_dir(), scratch_dir()

  == Help/UX

  - [ ] Mark advanced flags as hidden from short help
      - Use `#[arg(hide_short_help = true, help_heading = "Advanced")]`
  - [ ] Add visible aliases:
      - Note::New: `#[command(alias = "daily", visible_alias = "today")]`

  == Future Nice-to-haves

  - [ ] Templates dir support (~/.config/mnf/templates/)
  - [ ] Variables for week number, time, user, project
  - [ ] Unit test: render with fixed date; golden file compare
  - [ ] mnf gist new with petname names
  - [ ] mnf note edit optional path → random file fallback
