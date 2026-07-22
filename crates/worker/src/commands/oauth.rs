use crate::cli::OauthCommand;
use crate::error::WorkerError;
use crate::oauth_materials;

pub fn execute_oauth_command(command: OauthCommand) -> Result<(), WorkerError> {
    match command {
        OauthCommand::GenerateMaterials { out_dir, force } => {
            oauth_materials::generate(&out_dir, force)
        }
    }
}
