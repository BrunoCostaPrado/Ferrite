use ferrite::compiler::Compiler;
use ferrite::config;
use std::time::Instant;

fn fmt_size(bytes: usize) -> String {
  if bytes >= 1024 {
    format!("{:.2} kB", bytes as f64 / 1024.0)
  } else {
    format!("{bytes} B")
  }
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let mut entry = None;
  let mut tsconfig = None;

  let mut i = 1;
  while i < args.len() {
    match args[i].as_str() {
      "--tsconfig" => {
        i += 1;
        tsconfig = args.get(i).cloned();
      }
      "build" | "compile" => {} // subcommand aliases — same as bare ferrite
      _ if entry.is_none() => entry = Some(args[i].clone()),
      _ => {}
    }
    i += 1;
  }

  // Auto-detect config: try ferrite.config.ts/js/json in cwd, then entry's dir
  let mut ferrite_cfg = None;
  let cwd = std::env::current_dir().unwrap_or_default();
  if let Some((cfg, cfg_dir)) = config::load_ferrite_config(&cwd) {
    eprintln!("\x1b[2mℹ  ferrite\x1b[0m");
    // Show actual config file found
    for name in &["ferrite.config.ts", "ferrite.config.js", "ferrite.config.json"] {
      if cfg_dir.join(name).exists() {
        eprintln!("\x1b[2mℹ  config file: {}\x1b[0m", cfg_dir.join(name).display());
        break;
      }
    }
    ferrite_cfg = Some((cfg, cfg_dir));
  }

  // If no entry given, get it from ferrite config
  if entry.is_none() {
    if let Some((ref cfg, _)) = ferrite_cfg {
      if let Some(ref entries) = cfg.entry {
        if let Some(first) = entries.first() {
          entry = Some(first.clone());
        }
      }
    }
  }

  let Some(entry) = entry else {
    eprintln!("Usage: ferrite <file.ts> [--tsconfig <path>]");
    eprintln!("Or create a ferrite.config.ts with entry points.");
    std::process::exit(1);
  };

  eprintln!("\x1b[2mℹ  entry: {entry}\x1b[0m");

  let start = Instant::now();

  let result = if let Some(cfg) = tsconfig {
    Compiler::compile_with_tsconfig(&entry, &cfg)
  } else {
    Compiler::compile(&entry)
  };

  match result {
    Ok(outputs) => {
      let mut total_bytes = 0usize;
      for (path, content) in &outputs {
        // Ensure parent dir exists (for outDir mirroring)
        if let Some(parent) = std::path::Path::new(path).parent() {
          let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(path, content) {
          eprintln!("Error writing {path}: {e}");
          std::process::exit(1);
        }
        total_bytes += content.len();
        // Format: dist/file.js    1.23 kB
        let display = std::path::Path::new(path)
          .file_name()
          .map(|f| f.to_string_lossy().to_string())
          .unwrap_or_else(|| path.clone());
        eprintln!("\x1b[2mℹ  {display:<32} {}\x1b[0m", fmt_size(content.len()));
      }
      let elapsed = start.elapsed().as_millis();
      let file_count = outputs.iter().filter(|(p, _)| p.ends_with(".js")).count();
      eprintln!("\x1b[2mℹ  {} files, total: {}\x1b[0m", file_count, fmt_size(total_bytes));
      eprintln!("\x1b[32m✔\x1b[0m Build complete in {elapsed}ms");
    }
    Err(errors) => {
      for err in &errors {
        eprintln!("{err}");
      }
      std::process::exit(1);
    }
  }
}
