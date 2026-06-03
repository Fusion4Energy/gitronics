use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::{GitronicsError, build_model, init_logger, migrate_model};

#[derive(Parser)]
#[command(
    name = "gitronics",
    about = "Assemble MCNP neutronics models from modular components"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build an MCNP model from a project configuration.
    ///
    /// Loads the configuration file path from the project directory, assembles
    /// the envelope structure with all filler models, and writes the result
    /// to the output directory.
    Build {
        /// Path to the configuration file (relative to project root or absolute)
        config: PathBuf,

        /// Path to the output directory
        #[arg(short, long, default_value = ".")]
        output_path: PathBuf,
    },

    /// Take a traditional monolithic MCNP model and migrate it to a new Gitronics project structure.
    ///
    /// This command will divide the monolithic model into modular components while preparing a
    /// baseline configuration file that would allow the rebuild of the original model with the
    /// `build` command. The model will be divided into the following components:
    #[command(verbatim_doc_comment)]
    ///
    ///   - configurations/
    ///     - baseline.yaml: The original configuration file.
    ///   - output/
    ///     - .gitignore: To ignore the output of the build process in git.
    ///     - assembled.mcnp: Written here after a successful build.
    ///   - reference_model/
    ///     - filler_models/: A directory containing the universe filler models.
    ///     - data_cards.source: A file containing all the data cards.
    ///     - envelope_structure.mcnp: The level 0 cells of the model.
    Migrate {
        /// Path to the original monolithic MCNP input file
        mcnp_input: PathBuf,

        /// Path to the output directory
        #[arg(short, long, default_value = "./project")]
        output_path: PathBuf,
    },
}

/// Parse `args` with the clap CLI and run the requested command.
///
/// Pass `std::env::args()` for normal use, or an arbitrary iterator when
/// calling from Python (via `sys.argv`).
pub fn run_cli(args: impl IntoIterator<Item = String>) -> Result<(), GitronicsError> {
    init_logger();
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Build {
            config,
            output_path,
        } => build_model(&config, &output_path),
        Commands::Migrate {
            mcnp_input,
            output_path,
        } => migrate_model(&mcnp_input, &output_path),
    }
}
