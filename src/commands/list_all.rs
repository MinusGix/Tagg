use termcolor::{ColorChoice, StandardStream};

use crate::state::State;

use super::print_file;

pub fn list_all(state: &State) -> eyre::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    for file in state.storage.files.iter() {
        print_file(
            &mut stdout,
            &file.filename,
            file.original_filename.as_deref(),
            &file.tags,
        )?;
    }

    Ok(())
}
