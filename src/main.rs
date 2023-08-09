use quartz_nbt::io::{self, Flavor};
use quartz_nbt::NbtCompound;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::convert::AsRef;
use std::path::{Path, PathBuf};

fn main() {
    // Read CLI arguments
    let mut args = std::env::args().skip(1);
    let world_in = args.next().expect("Invalid arguments. World directory expected.");
    let world_out = format!("{world_in}_RELEASE");


    // Check source directory is a valid Minecraft Java Edition world
    let source_level_dat_file: PathBuf = [&world_in, "level.dat"].iter().collect();
    if ! file_exists(&source_level_dat_file) {
        eprintln!("\"{world_in}\" is not a valid Minecraft Java Edition world");
        return;
    }


    // Prepare the release world directory
    if is_dir(&world_out) {
        std::fs::remove_dir_all(&world_out).expect(format!("Cannot remove directory \"{world_out}\"").as_ref());
        println!("- Removed previous release directory \"{world_out}\"");
    }
    clone_directory_recursively(&world_in, &world_out).expect(format!("Cannot clone directory \"{world_out}\"").as_ref());
    println!("- Created \"{world_out}\" release world");


    // Clean up release level.dat file
    let dest_level_dat_file: PathBuf = [&world_out, "level.dat"].iter().collect();
    let mut file = File::options().read(true).write(true).open(dest_level_dat_file).expect("Cannot read level.dat file");
    let (mut root_nbt, root_name) = io::read_nbt(&mut file, Flavor::GzCompressed).expect("Error when reading level.dat file");
    let data = root_nbt.get_mut::<_, &mut NbtCompound>("Data").expect("This file doesn't contain a Data tag");

    // Remove the state of the player from level.dat in the case that this is a single-player world
    if let Some(_) = data.inner_mut().remove("Player") {
        println!("- Removed state of the player in level.dat");
    }

    // Set allowCommands to false in level.dat to disable cheats
    if let Ok(allow_commands) = data.get_mut::<_, &mut i8>("allowCommands") {
        *allow_commands = 0;
        println!("- Set allowCommands = 0 to disable cheats");
    }

    // Overwrite level.dat
    file.set_len(0).expect("Cannot truncate level.dat");
    file.seek(SeekFrom::Start(0)).expect("Cannot truncate level.dat");
    io::write_nbt(&mut file, Some(&root_name), &root_nbt, Flavor::GzCompressed).expect("I/O error when writing level.dat");


    // Remove unnecessary files from release
    let dest_level_dat_old_file: PathBuf = [&world_out, "level.dat_old"].iter().collect();
    if file_exists(&dest_level_dat_old_file) {
        // TODO: Handle errors
        let _ = std::fs::remove_file(&dest_level_dat_old_file);
        println!("- Removed level.dat_old file");
    }

    let dest_level_dat_old_file: PathBuf = [&world_out, "session.lock"].iter().collect();
    if file_exists(&dest_level_dat_old_file) {
        // TODO: Handle errors
        let _ = std::fs::remove_file(&dest_level_dat_old_file);
        println!("- Removed session.lock file");
    }

    // Remove all files from DEST/advancements
    let dest_advancements_dir: PathBuf = [&world_out, "advancements"].iter().collect();
    if let Err(e) = remove_all_files_from_directory(&dest_advancements_dir) {
        eprintln!("Cannot empty advancements directory");
        eprintln!("{}", e.to_string());
        return;
    }
    println!("- Emptied advancements directory");

    // Remove all files from DEST/playerdata
    let dest_advancements_dir: PathBuf = [&world_out, "playerdata"].iter().collect();
    if let Err(e) = remove_all_files_from_directory(&dest_advancements_dir) {
        eprintln!("Cannot empty playerdata directory");
        eprintln!("{}", e.to_string());
        return;
    }
    println!("- Emptied playerdata directory");

    // Remove all files from DEST/stats
    let dest_advancements_dir: PathBuf = [&world_out, "stats"].iter().collect();
    if let Err(e) = remove_all_files_from_directory(&dest_advancements_dir) {
        eprintln!("Cannot empty stats directory");
        eprintln!("{}", e.to_string());
        return;
    }
    println!("- Emptied stats directory");

    // Remove DEST/data/scoreboard.dat
    let dest_scoreboard_file: PathBuf = [&world_out, "data", "scoreboard.dat"].iter().collect();
    if file_exists(&dest_scoreboard_file) {
        if let Err(e) = std::fs::remove_file(&dest_scoreboard_file) {
            eprintln!("Cannot remove \"data/scoreboard.dat\" file");
            eprintln!("{}", e.to_string());
            return;
        }
        println!("- Removed data/scoreboard.dat file");
    }

    println!("Done!");
}

fn clone_directory_recursively<T: AsRef<Path>>(source_directory: T, destination_directory: T) -> Result<(), String> {
    if ! is_dir(&source_directory) {
        return Err(format!("\"{}\" is not a directory", source_directory.as_ref().to_str().unwrap_or("")));
    }

    let entries = std::fs::read_dir(&source_directory).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_type = entry.file_type() .map_err(|e| e.to_string())?;
        if entry_type.is_file() {
            let dest_file_dir = destination_directory.as_ref().join(entry.file_name());
            {
                let _ = std::fs::File::create(&dest_file_dir).map_err(|e| e.to_string())?;
                // Close file
            }
            // std::copy() copies all content from one file to another and it also copies the permission bits
            std::fs::copy(entry.path(), dest_file_dir).map_err(|e| e.to_string())?;
        } else if entry_type.is_dir() {
            let dest_dir = destination_directory.as_ref().join(entry.file_name());
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

fn remove_all_files_from_directory<T: AsRef<Path>>(directory: &T) -> Result<(), String> {
    let entries = std::fs::read_dir(&directory).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_type = entry.file_type() .map_err(|e| e.to_string())?;
        if entry_type.is_file() {
            std::fs::remove_file(entry.path()).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn is_dir<T: AsRef<Path>>(directory: &T) -> bool {
    match std::fs::metadata(directory) {
        Ok(m) => m.is_dir(),
        Err(_) => false
    }
}

fn file_exists<T: AsRef<Path>>(file_path: &T) -> bool {
    match std::fs::metadata(file_path) {
        Ok(m) => m.is_file(),
        Err(_) => false
    }
}
