use crate::state::State;

pub fn list_all(state: &State) {
    let files = &state.storage.files;
    for file in files {
        let filename = &file.filename;
        let original_filename = file.original_filename.as_deref();
        let tags = file.tags.join(", ");
        println!(
            "{} - {} - {}",
            filename,
            original_filename.unwrap_or("None"),
            tags
        );
    }
}
