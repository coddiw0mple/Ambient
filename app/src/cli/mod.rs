use std::path::PathBuf;

use clap::{Args, Parser};

pub mod new_project;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub enum Cli {
    /// Create a new Ambient project
    New {
        #[command(flatten)]
        project_args: ProjectCli,
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Builds and runs the project locally
    Run {
        #[command(flatten)]
        project_args: ProjectCli,
        #[command(flatten)]
        host_args: HostCli,
        #[command(flatten)]
        run_args: RunCli,
    },
    /// Builds the project
    Build {
        #[command(flatten)]
        project_args: ProjectCli,
    },
    /// Builds and runs the project in server-only mode
    Serve {
        #[command(flatten)]
        project_args: ProjectCli,
        #[command(flatten)]
        host_args: HostCli,
    },
    /// View an asset
    View {
        #[command(flatten)]
        project_args: ProjectCli,
        /// Relative to the project path
        asset_path: PathBuf,
    },
    /// Join a multiplayer session
    Join {
        #[command(flatten)]
        run_args: RunCli,
        /// The server to connect to; defaults to localhost
        host: Option<String>,
    },
    /// Updates all WASM APIs with the core primitive components (not for users)
    #[cfg(not(feature = "production"))]
    #[command(hide = true)]
    UpdateInterfaceComponents,
}
#[derive(Args, Clone)]
pub struct RunCli {
    /// Whether or not debug menus should be shown
    #[arg(long)]
    pub debug: bool,

    /// Run in headless mode
    #[arg(long)]
    pub headless: bool,

    /// Take a screenshot after N seconds, compare it to the existing one and then exit with an exit code of 1 if they are different
    #[arg(long)]
    pub screenshot_test: Option<f32>,

    /// The user ID to join this server with
    #[clap(short, long)]
    pub user_id: Option<String>,
}
#[derive(Args, Clone)]
pub struct ProjectCli {
    /// The path of the project to run; if not specified, this will default to the current directory
    pub path: Option<PathBuf>,
}
#[derive(Args, Clone)]
pub struct HostCli {
    /// Provide a public address or IP to the instance, which will allow users to connect to this instance over the internet
    ///
    /// Defaults to localhost
    #[arg(long)]
    pub public_host: Option<String>,
}

impl Cli {
    /// Extract run-relevant state only
    pub fn run(&self) -> Option<&RunCli> {
        match self {
            Cli::New { .. } => None,
            Cli::Run { run_args, .. } => Some(run_args),
            Cli::Build { .. } => None,
            Cli::Serve { .. } => None,
            Cli::View { .. } => None,
            Cli::Join { run_args, .. } => Some(run_args),
            #[cfg(not(feature = "production"))]
            Cli::UpdateInterfaceComponents => None,
        }
    }
    /// Extract project-relevant state only
    pub fn project(&self) -> Option<&ProjectCli> {
        match self {
            Cli::New { project_args, .. } => Some(project_args),
            Cli::Run { project_args, .. } => Some(project_args),
            Cli::Build { project_args, .. } => Some(project_args),
            Cli::Serve { project_args, .. } => Some(project_args),
            Cli::View { project_args, .. } => Some(project_args),
            Cli::Join { .. } => None,
            #[cfg(not(feature = "production"))]
            Cli::UpdateInterfaceComponents => None,
        }
    }
    /// Extract host-relevant state only
    pub fn host(&self) -> Option<&HostCli> {
        match self {
            Cli::New { .. } => None,
            Cli::Run { host_args, .. } => Some(host_args),
            Cli::Build { .. } => None,
            Cli::Serve { host_args, .. } => Some(host_args),
            Cli::View { .. } => None,
            Cli::Join { .. } => None,
            #[cfg(not(feature = "production"))]
            Cli::UpdateInterfaceComponents => None,
        }
    }
}
