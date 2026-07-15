use clap::Parser;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// orginal folder/directory
    #[arg(short, long)]
    dir: PathBuf,

    /// new name
    #[arg(short, long)]
    name: String,

    /// Starting number (default 1)
    #[arg(short, long, default_value_t = 1)]
    start: u32,

    /// dry run
    #[arg(long)]
    dry_run: bool,

    /// include subfolders
    #[arg(short, long)]
    recursive: bool,

    #[arg(long)]
    log: Option<PathBuf>,
}

const IMAGE_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "heic", "heif", "tif", "tiff", "bmp", "gif",
    "cr2", "nef", "arw", "dng", "raf", "orf", "rw2",
];

fn main() {
    let args = Args::parse();

    if !args.dir.is_dir() {
        eprintln!("Error: '{}' is not a valid directory.", args.dir.display());
        std::process::exit(1);
    }

    let (base_stem, forced_ext) = split_name(&args.name);

    let mut files = collect_photos(&args.dir, args.recursive);
    if files.is_empty() {
        eprintln!("No supported photo files found in '{}'.", args.dir.display());
        std::process::exit(1);
    }

    // Sort by filename (case-insensitive) so ordering matches what you see
    // in Finder/Explorer when sorted by name — this determines the
    // "chronological" numbering order.
    files.sort_by(|a, b| {
        a.file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase()
            .cmp(&b.file_name().unwrap().to_string_lossy().to_lowercase())
    });

    let entries = files;

    let total = entries.len() as u32;
    let end_number = args.start + total - 1;
    let width = end_number.to_string().len().max(2);

    let mut planned_targets: HashSet<PathBuf> = HashSet::new();
    let mut plan: Vec<(PathBuf, PathBuf)> = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        let number = args.start + i as u32;
        let ext = forced_ext
            .clone()
            .or_else(|| extension_of(entry))
            .unwrap_or_else(|| "jpg".to_string());

        let new_filename = format!("{}{:0width$}.{}", base_stem, number, ext, width = width);
        let new_path = entry.with_file_name(&new_filename);

        if planned_targets.contains(&new_path) {
            eprintln!(
                "Error: naming collision detected for '{}'. Aborting.",
                new_path.display()
            );
            std::process::exit(1);
        }
        planned_targets.insert(new_path.clone());
        plan.push((entry.clone(), new_path));
    }

    println!(
        "{} photo(s) found. Renaming to '{}01' .. '{}{:0width$}' based on filename order:\n",
        total, base_stem, base_stem, end_number, width = width
    );
    for (old, new) in &plan {
        println!(
            "  {}  ->  {}",
            old.file_name().unwrap().to_string_lossy(),
            new.file_name().unwrap().to_string_lossy()
        );
    }

    if args.dry_run {
        println!("\nDry run only — no files were changed. Remove --dry-run to apply.");
        return;
    }

    let mut temp_pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
    for (i, (old, _)) in plan.iter().enumerate() {
        let temp_name = format!(".__renaming_tmp_{}", i);
        let temp_path = old.with_file_name(temp_name);
        if let Err(e) = fs::rename(old, &temp_path) {
            eprintln!("Failed to stage '{}': {}", old.display(), e);
            std::process::exit(1);
        }
        temp_pairs.push((temp_path, old.clone()));
    }

    let mut log_lines = vec!["original_path,new_path".to_string()];
    for ((temp_path, original_old_path), (_, new_path)) in temp_pairs.iter().zip(plan.iter()) {
        if let Err(e) = fs::rename(temp_path, new_path) {
            eprintln!("Failed to finalize '{}': {}", new_path.display(), e);
            std::process::exit(1);
        }
        log_lines.push(format!(
            "{},{}",
            original_old_path.display(),
            new_path.display()
        ));
    }

    println!("\nDone! Renamed {} file(s).", plan.len());

    let log_path = args
        .log
        .unwrap_or_else(|| args.dir.join("rename_log.csv"));
    if let Ok(mut f) = File::create(&log_path) {
        let _ = f.write_all(log_lines.join("\n").as_bytes());
        println!("Undo log written to: {}", log_path.display());
    }
}

fn split_name(name: &str) -> (String, Option<String>) {
    let path = Path::new(name);
    match (path.file_stem(), path.extension()) {
        (Some(stem), Some(ext)) => (
            stem.to_string_lossy().to_string(),
            Some(ext.to_string_lossy().to_lowercase()),
        ),
        _ => (name.to_string(), None),
    }
}

fn extension_of(path: &Path) -> Option<String> {
    path.extension().map(|e| e.to_string_lossy().to_lowercase())
}

fn collect_photos(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let mut result = Vec::new();
    collect_photos_inner(dir, recursive, &mut result);
    result
}

fn collect_photos_inner(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_photos_inner(&path, recursive, out);
            }
            continue;
        }
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if IMAGE_EXTS.contains(&ext.as_str()) {
                out.push(path);
            }
        }
    }
}

