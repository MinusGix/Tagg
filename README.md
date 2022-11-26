# Tagg

A CLI tagging program (and library) for your files.  
  
## Installing
  
## General Design
The basic implementation of Tagg is to simply have a folder of your added files, and then have the tags (and other information) stored in a separate file.  
This avoids using filesystem-specific tagging implementations, which don't work across filesystems nicely, and also avoids the issue of the files disappearing.  
Since registered-files are kept in Tagg's storage folder, this avoids using folders for organization.  
  
Another key feature of Tagg is that it allows modifying the files after they're registered. If the filename stays the same, Tagg will consider it the same.  
This has the drawback of making it harder to 'deduplicate' files, like Hydrus does with images, but it has the benefit of allowing you to store files you are actively editing within it.  

## Usage
### Adding a file 
`tagg add story.epub --tags book fiction genre:fantasy long`
This adds the command to the *registration area*. It is not yet completely added.  
`tagg add grokking.pdf deeplearningtheory.pdf --tags paper deep-learning`
You can specify multiple filenames to apply shared tags to a lot of files at once.  
`tagg add deeplearningtheory.pdf --tags book math long`

We can do
```bash
> tagg status
Files in registration-area:
 - story.epub (book fiction genre:fantasy long)
 - grokking.pdf (paper deep-learning)
 - deeplearningtheory.pdf (paper deep-learning book math long)

Use `tagg commit` to register the files into your storage.  
Use `tagg remove [path]` to remove the file from the registration-area
```
If we follow that first instruction
`tagg commit`
Then we'll see that the files have disappeared!  
They've been copied to the storage folder that tagg uses, and then the 'originals' were moved to the trash.  



## FAQ  
### What you shouldn't store
Tagg doesn't try remembering the folder structure of your added files, which means you shouldn't add a program which expects files to be in a specific location relative to it (like a Unity game).  

### Large files
Tagg currently just stores all the files in one big folder. This means it should perfectly fine for large files.  
  
### Efficiency
Tagg tries to be some amount of efficient, but is also not particularly optimizing (at the moment) for use-cases where you have a million files in your storage.  

## TODOs/Ideas
Random list of planned features and potential ideas.

- Have an `add-comment` (and maybe a separate cli arg) which just adds an untitled comment to the file
- Have an `add-tag` which just adds a tag to a file

- Store registration-area files separately from the main file registry. This would help avoid potential issues with slowing down when just adding files.


### VFS
It would be useful to have some method of opening the files and editing them within your normal file manager.  
A good idea might be to use a virtual file-system, where we essentially redirect requests to it to the originals.  
Q: How to represent the files? We remember the old names, and could display those, but we run into the issue of duplicates.
  We could check for duplicates and just add a `(1)` `(2)` at the end. Though, that has some potential issues with having them open consistently.
  We could just append the internal-name as a suffix (or a cutoff suffix, since there's less space for duplicates with the filename in tow),
    which also means that you can rely on 'recent files' in programs to work.

### GUI
It should either run in the background or (optimally) just be fast to launch.  
