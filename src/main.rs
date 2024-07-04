use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};

const MH_MAGIC: u32 = 0xfeedface;
const MH_CIGAM: u32 = 0xcefaedfe;
const MH_MAGIC_64: u32 = 0xfeedfacf;
const MH_CIGAM_64: u32 = 0xcffaedfe;
const LC_RPATH: u32 = 0x8000001c;

#[derive(Debug, Clone)]
struct MachHeader {
    magic: u32,
    cputype: i32,
    cpusubtype: i32,
    filetype: u32,
    ncmds: u32,
    sizeofcmds: u32,
    flags: u32,
    reserved: u32,
}

#[derive(Debug, Clone)]
struct LoadCommand {
    cmd: u32,
    cmdsize: u32,
    data: Vec<u8>,
}

#[derive(Debug)]
struct RpathCommand {
    cmd: u32,
    cmdsize: u32,
    path_offset: u32,
    path: String,
}

fn parse_macho(data: &[u8]) -> Result<(MachHeader, Vec<LoadCommand>), std::io::Error> {
    let mut cursor = Cursor::new(data);
    let magic = cursor.read_u32::<BigEndian>()?;
    
    let (is_64, is_little_endian) = match magic {
        MH_MAGIC => (false, false),
        MH_CIGAM => (false, true),
        MH_MAGIC_64 => (true, false),
        MH_CIGAM_64 => (true, true),
        _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Not a Mach-O file")),
    };

    cursor.set_position(0);

    let header = if is_little_endian {
        read_header::<LittleEndian>(&mut cursor, is_64)?
    } else {
        read_header::<BigEndian>(&mut cursor, is_64)?
    };

    let mut load_commands = Vec::new();
    for _ in 0..header.ncmds {
        let cmd = if is_little_endian {
            cursor.read_u32::<LittleEndian>()?
        } else {
            cursor.read_u32::<BigEndian>()?
        };
        let cmdsize = if is_little_endian {
            cursor.read_u32::<LittleEndian>()?
        } else {
            cursor.read_u32::<BigEndian>()?
        };
        let mut data = vec![0u8; (cmdsize - 8) as usize];
        cursor.read_exact(&mut data)?;
        load_commands.push(LoadCommand { cmd, cmdsize, data });
    }

    Ok((header, load_commands))
}

fn read_header<T: byteorder::ByteOrder>(cursor: &mut Cursor<&[u8]>, is_64: bool) -> Result<MachHeader, std::io::Error> {
    let magic = cursor.read_u32::<T>()?;
    let cputype = cursor.read_i32::<T>()?;
    let cpusubtype = cursor.read_i32::<T>()?;
    let filetype = cursor.read_u32::<T>()?;
    let ncmds = cursor.read_u32::<T>()?;
    let sizeofcmds = cursor.read_u32::<T>()?;
    let flags = cursor.read_u32::<T>()?;
    let reserved = if is_64 { cursor.read_u32::<T>()? } else { 0 };

    Ok(MachHeader {
        magic,
        cputype,
        cpusubtype,
        filetype,
        ncmds,
        sizeofcmds,
        flags,
        reserved,
    })
}


fn add_rpath(data: &mut Vec<u8>, new_path: &str) -> Result<(), std::io::Error> {
    let (mut header, load_commands) = parse_macho(data)?;
    let mut cursor = Cursor::new(data);
    
    let header_size = if header.magic == MH_MAGIC_64 || header.magic == MH_CIGAM_64 {
        32 // 64-bit header size
    } else {
        28 // 32-bit header size
    };

    // Calculate the size of the new LC_RPATH command
    let path_len = new_path.len() + 1; // +1 for null terminator
    let cmdsize = (8 + path_len + 7) & !7; // 8 bytes for cmd and cmdsize, rounded up to 8-byte alignment

    // Find the end of the last load command
    let mut insert_offset = header_size as u64;
    for cmd in &load_commands {
        insert_offset += cmd.cmdsize as u64;
    }

    // Shift the rest of the file to make room for the new command
    let mut rest_of_file = Vec::new();
    cursor.set_position(insert_offset);
    cursor.read_to_end(&mut rest_of_file)?;
    
    // Insert the new LC_RPATH command
    cursor.set_position(insert_offset);
    cursor.write_u32::<LittleEndian>(LC_RPATH)?;
    cursor.write_u32::<LittleEndian>(cmdsize as u32)?;
    cursor.write_u32::<LittleEndian>(16)?; // path_offset is always 16 for LC_RPATH
    cursor.write_all(new_path.as_bytes())?;
    cursor.write_u8(0)?; // Null terminator
    
    // Pad to 8-byte alignment
    let padding = cmdsize - (8 + path_len);
    for _ in 0..padding {
        cursor.write_u8(0)?;
    }

    // Write the rest of the file
    cursor.write_all(&rest_of_file)?;

    // Update the Mach-O header
    header.ncmds += 1;
    header.sizeofcmds += cmdsize as u32;

    cursor.set_position(16); // Position of ncmds in header
    cursor.write_u32::<LittleEndian>(header.ncmds)?;
    cursor.write_u32::<LittleEndian>(header.sizeofcmds)?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut data = std::fs::read("helloworld")?;
    
    // Example usage: add a new LC_RPATH
    match add_rpath(&mut data, "/new/rpath") {
        Ok(()) => println!("Successfully added new LC_RPATH"),
        Err(e) => println!("Failed to add new LC_RPATH: {}", e),
    }
    
    // Write the modified data back to the file
    std::fs::write("helloworld", data)?;

    Ok(())
}


// fn something() {
//     use std::io::{Read, Cursor};
//     use std::fs::File;
//     use mach_object::{OFile, CPU_TYPE_X86_64, MachCommand, LoadCommand};

//     let mut f = File::open("test/helloworld").unwrap();
//     let mut buf = Vec::new();
//     let size = f.read_to_end(&mut buf).unwrap();
//     let mut cur = Cursor::new(&buf[..size]);
//     if let OFile::MachFile { ref header, ref commands } = OFile::parse(&mut cur).unwrap() {
//         assert_eq!(header.cputype, CPU_TYPE_X86_64);
//         assert_eq!(header.ncmds as usize, commands.len());
//         for &MachCommand(ref cmd, cmdsize) in commands {
//             if let &LoadCommand::Segment64 { ref segname, ref sections, .. } = cmd {
//                 println!("segment: {}", segname);

//                 for ref sect in sections {
//                     println!("  section: {}", sect.sectname);
//                 }
//             }

            

//             if let &LoadCommand::Rpath { ref segname, ref sections, .. } = cmd {
//                 println!("segment: {}", segname);

//                 for ref sect in sections {
//                     println!("  section: {}", sect.sectname);
//                 }
//             }
//         }
//     }

// }
