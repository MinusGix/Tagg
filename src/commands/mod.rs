use std::io::Write;
use std::{borrow::Cow, collections::HashMap, path::PathBuf};

use inquire::Confirm;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::{
    state::{AddedFile, COMMENT_MAIN},
    storage::FileInfo,
    tagg::Tagg,
    Commands,
};

pub mod list_all;

fn grey() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(Color::Rgb(0xA3, 0xA3, 0xA3)));
    spec
}

pub(crate) fn dispatch(tagg: &mut Tagg, command: Commands) -> eyre::Result<()> {
    match command {
        Commands::Status {} => {
            // TODO: Check if the files still exist
            // TODO: Display the tags
            let mut stdout = StandardStream::stdout(ColorChoice::Always);

            writeln!(&mut stdout, "Files in Registration-Area:")?;

            write!(&mut stdout, "  (use `")?;
            // Grey
            stdout.set_color(&grey())?;
            write!(&mut stdout, "tagg drop <file>")?;
            stdout.set_color(ColorSpec::new().set_fg(None))?;
            writeln!(&mut stdout, "` to remove it from the registration-area)")?;

            for added_file in tagg.state.registration_area.iter() {
                let name = added_file
                    .path
                    .file_name()
                    .unwrap_or(added_file.path.as_os_str())
                    .to_string_lossy();
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                write!(&mut stdout, "    {}  ", name)?;

                stdout.set_color(&grey())?;
                write!(&mut stdout, "[")?;

                for (i, tag) in added_file.tags.iter().enumerate() {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                    write!(&mut stdout, "{}", tag)?;

                    if i + 1 < added_file.tags.len() {
                        stdout.set_color(ColorSpec::new().set_fg(None))?;
                        write!(&mut stdout, ", ")?;
                    }
                }
                stdout.set_color(&grey())?;
                writeln!(&mut stdout, "]")?;
            }

            stdout.set_color(ColorSpec::new().set_fg(None))?;
        }
        Commands::Add {
            files,
            comment,
            tags,
        } => {
            println!("Adding files {:?} with tags {:?}", files, tags);

            'outer: for file in files {
                let path = PathBuf::from(file.clone());
                let path = path.canonicalize();
                let path = if let Ok(path) = path {
                    path
                } else {
                    eprintln!(
                        "WARN: Skipped {:?} because it failed to canonicalize to an absolute path",
                        file
                    );
                    continue;
                };

                let path_meta = std::fs::metadata(&path)?;
                if path_meta.is_dir() {
                    eprintln!(
                        "WARN: Skipped {:?} because it was a directory rather than a file",
                        file
                    );
                    continue;
                } else if path_meta.is_symlink() {
                    eprintln!(
                        "WARN: Skipped {:?} because it was a symlink rather than a file",
                        file
                    );
                    continue;
                }

                // TODO: Check hash?
                for added_file in tagg.state.registration_area.iter_mut() {
                    if path == added_file.path {
                        if tagg.verbose {
                            eprintln!("INFO: {:?} already existed in the registration area.", path);
                        }

                        added_file.tags.extend(tags.clone().into_iter());
                        let tag_count = added_file.tags.len();

                        // We have to sort so that dedup can work
                        added_file.tags.sort();
                        added_file.tags.dedup();

                        if added_file.tags.len() < tag_count {
                            let removed = tag_count - added_file.tags.len();
                            eprintln!(
                                "INFO: {:?} has #{} tags ignored due to being duplicates",
                                path, removed
                            );
                        }

                        if tagg.verbose && tag_count != added_file.tags.len() {
                            eprintln!("INFO: Added extra tags to already existing file {:?}", file);
                        }

                        if let Some(comment) = comment.clone() {
                            // Check if the user actually wants to replace the main-comment
                            let set_comment = if let Some(prev_comment) =
                                added_file.comment.get(COMMENT_MAIN)
                            {
                                eprintln!("WARN: You set a main-comment for an already existing file in the registration area, which also already had a main-comment.");
                                eprintln!("File: {:?}", file);
                                // TODO: Bold these prefixes
                                eprintln!("Previous Comment: {}", prev_comment);
                                eprintln!("New      Comment: {}", comment);
                                Confirm::new("Do you want to replace the previous comment with the new comment?")
                                    .with_default(true)
                                    .prompt()?
                            } else {
                                true
                            };

                            if set_comment {
                                added_file.comment.insert(COMMENT_MAIN.to_owned(), comment);
                            }
                        }

                        continue 'outer;
                    }
                }

                // If we're down here then it didn't already exist.

                // TODO: Get hash of file and store it
                let hash = None;
                let comment = {
                    let mut comments = HashMap::new();
                    if let Some(comment) = comment.clone() {
                        comments.insert(COMMENT_MAIN.to_owned(), comment);
                    }

                    comments
                };

                let added_file = AddedFile {
                    path,
                    hash,
                    comment,
                    tags: tags.clone(),
                };

                tagg.state.registration_area.push(added_file);
            }

            tagg.save_state()?;
        }
        Commands::Commit { dry, soft } => {
            if tagg.state.registration_area.is_empty() {
                eprintln!("There was no files in the registration area to commit.");
                return Ok(());
            }

            while let Some(added_file) = tagg.state.registration_area.pop() {
                commit_file(tagg, added_file, dry, soft)?;
            }
        }
        Commands::ListAll {} => {
            list_all::list_all(&tagg.state);
        }
        Commands::Find { tags } => {
            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            'outer: for file in tagg.state.storage.files.iter() {
                for tag in tags.iter() {
                    if let Some(tag) = tag.strip_prefix('-') {
                        if file.tags.iter().any(|x| x.as_str() == tag) {
                            // We found a filtered out tag
                            continue 'outer;
                        }
                    } else {
                        let tag = if let Some(tag) = tag.strip_prefix('+') {
                            tag
                        } else {
                            tag.as_str()
                        };

                        if !file.tags.iter().any(|x| x.as_str() == tag) {
                            // We didn't find the tag
                            continue 'outer;
                        }
                    }
                }

                stdout.set_color(&grey())?;
                write!(&mut stdout, "  {} ", file.filename)?;
                if let Some(original) = &file.original_filename {
                    write!(&mut stdout, "(")?;
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                    write!(&mut stdout, "{}", original)?;
                    stdout.set_color(&grey())?;
                    write!(&mut stdout, ") ")?;
                }

                stdout.set_color(&grey())?;
                write!(&mut stdout, "[")?;

                for (i, tag) in file.tags.iter().enumerate() {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                    write!(&mut stdout, "{}", tag)?;

                    if i + 1 < file.tags.len() {
                        stdout.set_color(ColorSpec::new().set_fg(None))?;
                        write!(&mut stdout, ", ")?;
                    }
                }
                stdout.set_color(&grey())?;
                writeln!(&mut stdout, "]")?;
            }
        }
    }

    Ok(())
}

fn commit_file(tagg: &mut Tagg, added_file: AddedFile, dry: bool, soft: bool) -> eyre::Result<()> {
    // TODO: check hash

    let original_filename = added_file
        .path
        .file_name()
        .map(|x| x.to_string_lossy())
        .map(Cow::into_owned);

    let extension = added_file
        .path
        .extension()
        .map(|x| x.to_string_lossy())
        .unwrap_or(Cow::Borrowed(""));

    // Unique filename
    let filename = tagg.choose_filename(&extension);
    if tagg.verbose {
        eprintln!("INFO: Committing {:?} -> {:?}", original_filename, filename);
    }

    let dest_path = tagg.get_storage_path(&filename)?;

    // Ensure that the destination doesn't exist, just in case
    assert!(!dest_path.exists(), "Unique name for file already existed! This may be a one-in-a-bazillion occurrence, but it is probably a bug. If this happens repeatedly, then it's a bug.");

    if !dry {
        if tagg.verbose {
            eprintln!(
                "INFO: Copying {:?} to storage destination {:?}",
                added_file.path, dest_path
            );
        }
        // We can just error on failure because we're adding the files one-at-a-time and then saving
        std::fs::copy(&added_file.path, dest_path)?;

        if !soft {
            if tagg.verbose {
                eprintln!("INFO: Moving original file to trash");
            }
            trash::delete(&added_file.path)?;
        }
    } else if tagg.verbose {
        eprintln!("INFO: Dry run commit, thus did not copy file or remove it");
    }

    let file_info = FileInfo {
        filename,
        original_filename,
        comments: added_file.comment,
        tags: added_file.tags,
    };

    tagg.state.storage.files.push(file_info);

    if !dry {
        // We save the state after each entry to avoid wacky duplicates
        tagg.save_state()?;
    } else if tagg.verbose {
        eprintln!("INFO: Dry run commit, thus did not save state");
    }

    Ok(())
}
