//! Vendored Python interpreter configuration.
//!
//! This module is responsible for finding and configuring a vendored Python
//! interpreter at runtime so that PyO3 can locate the interpreter, standard
//! library, and installed packages.
//!
//! The function in this module is called automatically by
//! [`render_dashboard`](crate::render::render_dashboard) before acquiring the
//! Python GIL, so library users do not need to call it directly.

/// Configure the vendored Python so `PyO3` can find the interpreter, standard
/// library, and installed packages.
///
/// This is called automatically by [`render_dashboard`](crate::render_dashboard)
/// and [`Dashboard::render`](crate::Dashboard::render). It searches for a
/// vendored Python installation in several candidate directories relative to the
/// current executable, and if found, sets `PYTHONHOME`, `PYTHONPATH`, and
/// `PATH` accordingly.
pub fn configure_vendored_python() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(std::path::Path::to_path_buf));

    let candidates = [
        exe_dir.as_ref().map(|d| d.join("../../vendor/python")),
        exe_dir.as_ref().map(|d| d.join("vendor/python")),
        Some(std::path::PathBuf::from("vendor/python")),
    ];

    for candidate in candidates.iter().flatten() {
        if let Ok(mut canon) = candidate.canonicalize() {
            if cfg!(windows) {
                let s = canon.to_string_lossy().to_string();
                if let Some(stripped) = s.strip_prefix(r"\\?\") {
                    canon = std::path::PathBuf::from(stripped);
                }
            }
            if canon.join("python.exe").exists() || canon.join("bin/python3").exists() {
                std::env::set_var("PYTHONHOME", &canon);

                let site_packages = if cfg!(windows) {
                    canon.join("Lib").join("site-packages")
                } else {
                    let lib = canon.join("lib");
                    std::fs::read_dir(&lib)
                        .ok()
                        .and_then(|mut entries| {
                            entries.find_map(|e| {
                                let name = e.ok()?.file_name().to_string_lossy().to_string();
                                name.starts_with("python3")
                                    .then(|| lib.join(name).join("site-packages"))
                            })
                        })
                        .unwrap_or_else(|| lib.join("python3").join("site-packages"))
                };
                std::env::set_var("PYTHONPATH", &site_packages);

                let path_var = std::env::var_os("PATH").unwrap_or_default();
                let mut paths = std::env::split_paths(&path_var).collect::<Vec<_>>();
                paths.insert(0, canon);
                if let Ok(new_path) = std::env::join_paths(&paths) {
                    std::env::set_var("PATH", &new_path);
                }
                return;
            }
        }
    }
}
