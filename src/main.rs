use quartz_nbt::io::{self, Flavor};
use quartz_nbt::NbtCompound;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::convert::AsRef;
use std::path::Path;

fn main() {
    // Read CLI arguments
    let mut args = std::env::args().skip(1);
    let world_in = args.next().expect("Invalid arguments. World directory expected.");
    let world_out = format!("{world_in}_RELEASE");

    let world_in = Path::new(&world_in);
    let world_out = Path::new(&world_out);


    // Check source directory is a valid Minecraft Java Edition world
    let level_dat = Path::new("level.dat");
    let source_level_dat_file = world_in.join(&level_dat);
    if ! source_level_dat_file.is_file() {
        eprintln!("\"{}\" is not a valid Minecraft Java Edition world", world_in.display());
        return;
    }


    // Prepare the release world directory
    if world_out.is_dir() {
        std::fs::remove_dir_all(&world_out).expect(format!("Cannot remove directory \"{}\"", world_out.display()).as_ref());
        println!("- Removed previous release directory \"{}\"", world_out.display());
    }
    clone_directory_recursively(&world_in, &world_out).expect(format!("Cannot clone directory \"{}\"", world_out.display()).as_ref());
    println!("- Created \"{}\" release world", world_out.display());


    // Clean up release level.dat file
    let dest_level_dat_file = world_out.join(&level_dat);
    let mut file = File::options().read(true).write(true).open(dest_level_dat_file).expect("Cannot read level.dat file");
    let (mut root_nbt, root_name) = io::read_nbt(&mut file, Flavor::GzCompressed).expect("Error when reading level.dat file");
    let data = root_nbt.get_mut::<_, &mut NbtCompound>("Data").expect("This file doesn't contain a Data tag");

    // Remove the state of the player from level.dat in the case that this is a single-player world
    if let Some(_) = data.inner_mut().remove("Player") {
        println!("- Removed state of the player in level.dat");
    }

    // Set allowCommands to false in level.dat to disable cheats
    if let Ok(allow_commands) = data.get_mut::<_, &mut i8>("allowCommands") {
        if *allow_commands != 0 {
            *allow_commands = 0;
            println!("- Set allowCommands = 0 to disable cheats in level.dat");
        }
    }

    // Overwrite level.dat
    file.set_len(0).expect("Cannot truncate level.dat");
    file.seek(SeekFrom::Start(0)).expect("Cannot truncate level.dat");
    io::write_nbt(&mut file, Some(&root_name), &root_nbt, Flavor::GzCompressed).expect("I/O error when writing level.dat");


    // Remove unnecessary files from release
    let remove_files = [
        Path::new("level.dat_old"),
        Path::new("session.lock"),
        Path::new("data/scoreboard.dat")
    ];
    for file in remove_files {
        let file_name = file.display();
        let file = world_out.join(file);
        if file.is_file() {
            match std::fs::remove_file(&file) {
                Ok(_) => println!("- Removed file {file_name}"),
                Err(e) => {
                    eprintln!("Error when deleting file {file_name}");
                    eprintln!("{e}");
                }
            }
        }
    }

    // Empty selected directories
    let directories = [
        Path::new("advancements"),
        Path::new("playerdata"),
        Path::new("stats")
    ];
    for directory in directories {
        let directory_name = directory.display();
        let directory = world_out.join(directory);
        if directory.is_dir() {
            match empty_directory(&directory) {
                Ok(delete_count) => {
                    if delete_count > 0 {
                        println!("- Emptied directory {directory_name}");
                    }
                }
                Err(e) => {
                    eprintln!("Error when removing all files from directory {directory_name}");
                    eprintln!("{e}");
                }
            }
        }
    }

    println!("Done!");
}

fn clone_directory_recursively<T: AsRef<Path>>(source_directory: &T, destination_directory: &T) -> Result<(), String> {
    let source_directory = source_directory.as_ref();
    let destination_directory = destination_directory.as_ref();
    if ! source_directory.is_dir() {
        return Err(format!("\"{}\" is not a directory", source_directory.display()));
    }

    let entries = std::fs::read_dir(&source_directory).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_type = entry.file_type() .map_err(|e| e.to_string())?;
        if entry_type.is_file() {
            let dest_file_dir = destination_directory.join(entry.file_name());
            {
                let _ = std::fs::File::create(&dest_file_dir).map_err(|e| e.to_string())?;
                // Close file
            }
            // std::copy() copies all content from one file to another and it also copies the permission bits
            std::fs::copy(entry.path(), dest_file_dir).map_err(|e| e.to_string())?;
        } else if entry_type.is_dir() {
            let dest_dir = destination_directory.join(entry.file_name());
            let _ = std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

            // Copy all entries from this directory recursively
            let start_source = entry.path();
            let _ = clone_directory_recursively(&start_source, &dest_dir)?;
        } else {
            // Ignore symlinks and other file types
        }
    }
    Ok(())
}

fn empty_directory<T: AsRef<Path>>(directory: &T) -> Result<usize, String> {
    // Remove all files from the directory
    let entries = std::fs::read_dir(&directory).map_err(|e| e.to_string())?;
    let mut count = 0;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_type = entry.file_type() .map_err(|e| e.to_string())?;
        if entry_type.is_file() {
            std::fs::remove_file(entry.path()).map_err(|e| e.to_string())?;
            count += 1;
        }
    }
    Ok(count)
}
