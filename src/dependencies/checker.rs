use which::which;

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyStatus {
    Installed,
    NotFound,
    Error(String),
}

impl DependencyStatus {
    pub fn is_available(&self) -> bool {
        matches!(self, DependencyStatus::Installed)
    }

    pub fn status_text(&self) -> &str {
        match self {
            DependencyStatus::Installed => "Installed",
            DependencyStatus::NotFound => "Not Found",
            DependencyStatus::Error(_) => "Error",
        }
    }
}

pub struct DependencyChecker {
    pub cargo_generate: DependencyStatus,
    pub espflash: DependencyStatus,
    pub ravedude: DependencyStatus,
    pub probe_rs: DependencyStatus,
}

impl DependencyChecker {
    pub fn new() -> Self {
        Self {
            cargo_generate: DependencyStatus::NotFound,
            espflash: DependencyStatus::NotFound,
            ravedude: DependencyStatus::NotFound,
            probe_rs: DependencyStatus::NotFound,
        }
    }

    pub fn check_all(&mut self) {
        self.cargo_generate = Self::check_tool("cargo-generate");
        self.espflash = Self::check_tool("espflash");
        self.ravedude = Self::check_tool("ravedude");
        self.probe_rs = Self::check_tool("probe-rs");
    }

    fn check_tool(tool_name: &str) -> DependencyStatus {
        match which(tool_name) {
            Ok(_) => DependencyStatus::Installed,
            Err(which::Error::CannotFindBinaryPath) => DependencyStatus::NotFound,
            Err(e) => DependencyStatus::Error(format!("Error checking {}: {}", tool_name, e)),
        }
    }
}
