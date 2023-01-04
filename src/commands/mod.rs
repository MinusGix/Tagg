use std::io::Write;
use std::{borrow::Cow, collections::HashMap, path::PathBuf};

use inquire::Confirm;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::storage::Storage;
use crate::util::{expand_path, extract_title};
use crate::{
    state::{AddedFile, COMMENT_MAIN},
    storage::FileInfo,
    tagg::Tagg,
    Commands,
};

pub mod list_all;

pub(crate) fn grey() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(Color::Rgb(0xA3, 0xA3, 0xA3)));
    spec
}

pub(crate) fn light_grey() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(Color::Rgb(0xC3, 0xC3, 0xC3)));
    spec
}

pub(crate) fn dispatch(tagg: &mut Tagg, command: Commands) -> eyre::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    match command {
        Commands::Status {} => {
            // TODO: Check if the files still exist

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

                write_tags(&mut stdout, &added_file.tags)?;
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
        Commands::Drop { files } => {
            for file in files {
                let file = Some(Cow::Owned(file));
                let mut found = false;
                tagg.state.registration_area.retain(|added_file| {
                    if file == added_file.path.file_name().map(|x| x.to_string_lossy()) {
                        found = true;
                        false
                    } else {
                        true
                    }
                });
                if !found {
                    eprintln!("Failed to find {:?} in registration-area", file);
                }
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
        Commands::AddTags { tags, files } => {
            for file in files {
                if let Some(file) =
                    get_single_file_mut_from_prefix(&mut stdout, &mut tagg.state.storage, &file)?
                {
                    file.tags.extend(tags.iter().cloned());
                    let tag_count_after = file.tags.len();

                    file.tags.sort();
                    file.tags.dedup();

                    if file.tags.len() < tag_count_after {
                        let removed = tag_count_after - file.tags.len();
                        eprintln!(
                            "INFO: {:?} has #{} new tags ignored due to being duplicates",
                            file.filename, removed
                        );
                    }

                    // TODO: inform on no tag change
                    print_file(
                        &mut stdout,
                        &file.filename,
                        file.original_filename.as_deref(),
                        &file.tags,
                    )?;
                }
            }

            tagg.save_state()?;
        }
        Commands::SetComment {
            files,
            message,
            title,
        } => {
            let title = title.unwrap_or_else(|| COMMENT_MAIN.to_string());
            for file in files {
                if let Some(file) =
                    get_single_file_mut_from_prefix(&mut stdout, &mut tagg.state.storage, &file)?
                {
                    file.comments.insert(title.clone(), message.clone());
                }
            }

            tagg.save_state()?;
        }
        Commands::SetTitle { file, message } => {
            let title = "title".to_string();
            if let Some(file) =
                get_single_file_mut_from_prefix(&mut stdout, &mut tagg.state.storage, &file)?
            {
                file.comments.insert(title, message);
            }

            tagg.save_state()?;
        }
        Commands::GenerateTitles { dry } => {
            for file in tagg.state.storage.files.iter_mut() {
                if file.comments.contains_key("title") {
                    continue;
                }

                let mut path = expand_path(&tagg.config.storage_path);
                path.push(&file.filename);
                let extension = path
                    .extension()
                    .map(|x| x.to_string_lossy())
                    .unwrap_or(Cow::Borrowed(""));

                if let Some(title) = extract_title(&path, &extension) {
                    if tagg.verbose {
                        eprintln!("Set {}'s title to {:?}", file.filename, title);
                    }
                    file.comments.insert("title".to_string(), title);
                } else if tagg.verbose {
                    eprintln!("Failed to get title for {:?}", path);
                }
            }

            if !dry {
                tagg.save_state()?;
            }
        }
        Commands::SetDesc { file, message } => {
            let title = "desc".to_string();
            if let Some(file) =
                get_single_file_mut_from_prefix(&mut stdout, &mut tagg.state.storage, &file)?
            {
                file.comments.insert(title, message);
            }

            tagg.save_state()?;
        }
        Commands::ListAll {} => {
            list_all::list_all(&tagg.state)?;
        }
        Commands::Find { tags } => {
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

                print_file_comments_colored(
                    &mut stdout,
                    &file.filename,
                    file.original_filename.as_deref(),
                    &file.tags,
                    &file.comments,
                    tags.clone(),
                )?;
            }
        }
        // TODO: Command to open based on tags?
        // TODO: Command to open based on old filename?
        // TODO: Way of displaying clickable links to the user in search/list-all that will automatically xdg-open them?
        Commands::Open { files, using } => {
            for file in files {
                if let Some(file) =
                    get_single_file_mut_from_prefix(&mut stdout, &mut tagg.state.storage, &file)?
                {
                    let filename = file.filename.clone();
                    let path = tagg.get_storage_path(&filename)?;
                    if let Some(using) = using.as_deref() {
                        open::with(&path, using)?;
                    } else {
                        open::that(&path)?;
                        // open::that(&path)?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub(crate) fn get_single_file_mut_from_prefix<'a, 'b: 'a>(
    out: &mut impl WriteColor,
    storage: &'a mut Storage,
    file: &'b str,
) -> eyre::Result<Option<&'a mut FileInfo>> {
    let mut files = storage.find_file_mut_from_prefix(file);
    let first = files.next();
    let second = files.next();
    if first.is_none() {
        eprintln!("WARN: Failed to find file with prefix {:?}", file);
        Ok(None)
    } else if second.is_none() {
        Ok(first)
    } else {
        writeln!(
            out,
            "There was more than one entry which would match the prefix {:?}",
            file
        )?;
        for file in files {
            print_file(
                out,
                &file.filename,
                file.original_filename.as_deref(),
                &file.tags,
            )?;
        }
        Ok(None)
    }
}

pub(crate) fn print_file<T: AsRef<str>>(
    out: &mut impl WriteColor,
    filename: &str,
    original_filename: Option<&str>,
    tags: &[T],
) -> eyre::Result<()> {
    out.set_color(&grey())?;
    write!(out, "  {} ", filename)?;
    if let Some(original) = &original_filename {
        write!(out, "(")?;
        out.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
        write!(out, "{}", original)?;
        out.set_color(&grey())?;
        write!(out, ") ")?;
    }

    write_tags(out, tags)?;
    Ok(())
}

pub(crate) fn print_file_comments<T: AsRef<str>>(
    out: &mut impl WriteColor,
    filename: &str,
    original_filename: Option<&str>,
    tags: &[T],
    comments: &HashMap<String, String>,
) -> eyre::Result<()> {
    out.set_color(&grey())?;
    write!(out, "  {} ", filename)?;
    if let Some(original) = &original_filename {
        write!(out, "(")?;
        out.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
        write!(out, "{}", original)?;
        out.set_color(&grey())?;
        write!(out, ") ")?;
    }

    write_tags(out, tags)?;

    for (title, comment) in comments.iter() {
        write!(out, "    - ")?;
        if title != COMMENT_MAIN {
            out.set_color(&grey())?;
            write!(out, "{}", title)?;
            out.reset()?;
            write!(out, ": ")?;
        }

        out.set_color(&light_grey())?;
        writeln!(out, "{}", comment)?;
    }
    Ok(())
}

pub(crate) fn write_tags<T: AsRef<str>>(out: &mut impl WriteColor, tags: &[T]) -> eyre::Result<()> {
    out.set_color(&grey())?;
    write!(out, "[")?;

    for (i, tag) in tags.iter().enumerate() {
        let tag = tag.as_ref();
        out.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
        write!(out, "{}", tag)?;

        if i + 1 < tags.len() {
            out.set_color(ColorSpec::new().set_fg(None))?;
            write!(out, ", ")?;
        }
    }
    out.set_color(&grey())?;
    writeln!(out, "]")?;

    Ok(())
}

pub(crate) fn write_matched_tags<T: AsRef<str>>(out: &mut impl WriteColor, tags: &[T], matched_tags: Vec<String>) -> eyre::Result<()> {
    out.set_color(&grey())?;
    write!(out, "[")?;

    for (i, tag) in tags.iter().enumerate() {
        let tag = tag.as_ref();
        // add a clause here to color the specific taggs used with `$ tagg find tag1 tag2 ...`
        if matched_tags.contains(&tag.to_string()) {
            out.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(out, "{}", tag)?;
        } else {
            out.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            write!(out, "{}", tag)?;
        }

        if i + 1 < tags.len() {
            out.set_color(ColorSpec::new().set_fg(None))?;
            write!(out, ", ")?;
        }
    }
    out.set_color(&grey())?;
    writeln!(out, "]")?;

    Ok(())
}

pub(crate) fn print_file_comments_colored<T: AsRef<str>>(
    out: &mut impl WriteColor,
    filename: &str,
    original_filename: Option<&str>,
    tags: &[T],
    comments: &HashMap<String, String>,
    matched_tags: Vec<String>
) -> eyre::Result<()> {
    out.set_color(&grey())?;
    write!(out, "  {} ", filename)?;
    if let Some(original) = &original_filename {
        write!(out, "(")?;
        out.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
        write!(out, "{}", original)?;
        out.set_color(&grey())?;
        write!(out, ") ")?;
    }

    write_matched_tags(out, tags, matched_tags)?;

    for (title, comment) in comments.iter() {
        write!(out, "    - ")?;
        if title != COMMENT_MAIN {
            out.set_color(&grey())?;
            write!(out, "{}", title)?;
            out.reset()?;
            write!(out, ": ")?;
        }

        out.set_color(&light_grey())?;
        writeln!(out, "{}", comment)?;
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

    let mut comments = added_file.comment;
    if !comments.contains_key("title") {
        if let Some(title) = extract_title(&added_file.path, &extension) {
            comments.insert("title".to_string(), title);
        }
    }

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
        comments,
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
