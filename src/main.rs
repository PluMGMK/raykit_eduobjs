extern crate pmw1;
extern crate md5;

use std::env::args;
use std::io::prelude::*;
use std::fs::{File,OpenOptions};

use pmw1::exe::Pmw1Exe;

// What code are we pulling out and where are we sticking it in?
const FRAMEFREEZE_CODE_DST:std::ops::Range<usize> = 0x537da..0x537e2;
const FRAMEFREEZE_ORIG_CODE_MD5:&str = "899302d008debf630bf09186f97132b0";

// Assembled from freezercode.asm using UASM (https://github.com/Terraspace/UASM)
// and formatted using hx (https://github.com/sitkevij/hex).
const FRAMEFREEZE_CODE_BLOB: [u8; 41] = [
    0x8a, 0x44, 0xc2, 0x02, 0x66, 0x8b, 0x59, 0x64, 0x66, 0x81, 
    0xfb, 0x03, 0x01, 0x74, 0x19, 0x66, 0x81, 0xfb, 0x04, 0x01, 
    0x74, 0x12, 0x66, 0x81, 0xfb, 0x12, 0x01, 0x74, 0x0b, 0x66, 
    0x81, 0xfb, 0x06, 0x01, 0x74, 0x04, 0xc6, 0x41, 0x6f, 0x00, 
    0xc3
];

// Assembled from colourcode.asm using UASM (https://github.com/Terraspace/UASM)
// and formatted using hx (https://github.com/sitkevij/hex).
const COLOUR_CODE_BLOB: [u8; 89] = [
    0x66, 0x3d, 0x03, 0x01, 0x74, 0x16, 0x66, 0x3d, 0x04, 0x01, 
    0x74, 0x10, 0x66, 0x3d, 0x12, 0x01, 0x74, 0x0a, 0x66, 0x3d, 
    0x05, 0x01, 0x74, 0x04, 0x66, 0x3d, 0x10, 0x01, 0xc3, 0x66, 
    0x93, 0xe8, 0xdc, 0xff, 0xff, 0xff, 0x66, 0x93, 0xc3, 0x66, 
    0x91, 0xe8, 0xd2, 0xff, 0xff, 0xff, 0x66, 0x91, 0xc3, 0x66, 
    0x92, 0xe8, 0xc8, 0xff, 0xff, 0xff, 0x66, 0x92, 0xc3, 0x66, 
    0x96, 0xe8, 0xbe, 0xff, 0xff, 0xff, 0x66, 0x96, 0xc3, 0x66, 
    0x97, 0xe8, 0xb4, 0xff, 0xff, 0xff, 0x66, 0x97, 0xc3, 0x66, 
    0x95, 0xe8, 0xaa, 0xff, 0xff, 0xff, 0x66, 0x95, 0xc3
];

fn main() -> std::io::Result<()> {
    // Assume the filename of interest is the LAST argument on the command line.
    let exe_name: String = args().next_back().unwrap();

    // Load the whole EXE into memory...
    let binary = {
        println!("Opening {}...", exe_name);

        let mut file = File::open(&exe_name)?;
        let mut buffer: Vec<u8> = Vec::with_capacity(0x100000);
        file.read_to_end(&mut buffer)?;
        buffer.shrink_to_fit();
        buffer
    };

    // Create a backup file.
    {
        let filename = format!("{}.BAK",exe_name);
        println!("");
        println!("Attempting to create NEW backup file {}", filename);
        // `create_new` to fail if the backup file already exists.
        // Don't wanna screw up an existing backup...
        let mut outfile = OpenOptions::new().write(true)
                                            .create_new(true)
                                            .open(&filename)?;
        // Write the whole binary back out
        outfile.write_all(&binary)?;
        println!("Backup successful");
    }

    println!("{} is {} bytes.", exe_name, binary.len());

    assert_eq!(binary[0..2],b"MZ"[..],
               "{} is not an MZ executable!", exe_name);
    assert!(binary.len() >= 0x1c,
            "{} doesn't appear to contain a complete MZ header!",exe_name);

    let mz_header = &binary[0x2..0x1c];
    let mz_header: Vec<u16> = (0..mz_header.len())
        .step_by(2)
        .map(|i| u16::from_le_bytes([mz_header[i], mz_header[i+1]]))
        .collect();

    // Print out some relevant info.
    println!("It begins with an MZ executable, of {} half-KiB blocks.",
             mz_header[1]);
    let total_block_size = mz_header[1] << 9; // Shift left to multiply by 512
    let actual_mz_size =
        if mz_header[0] == 0 {
            println!("Last block is fully used.");
            total_block_size
        } else {
            println!("{} bytes used in last block.", mz_header[0]);
            total_block_size - 512 + mz_header[0]
        } as usize;
    println!("Total MZ executable size is {} bytes.", actual_mz_size);

    assert!(binary.len() > actual_mz_size, "This appears to be a pure MZ executable!");

    // A slice containing just the PMW1 part.
    let mut pmw1_exe = Pmw1Exe::from_bytes(&binary[actual_mz_size..])?;

    // Deal with the frame-freezing
    println!("");
    println!("Adding frame-freezing-code...");

    // The fun part: put the code into the destination (i.e. KIT) EXE!
    pmw1_exe.entry_object_mut()
        .update_data(|old_data| {
            // We're going to hang a new function off the end of the code segment.
            // This will be its address:
            let newfunc_address = old_data.len();

            let mut new_data = old_data.to_vec(); // Turn it into a vector...
            new_data.extend_from_slice(&FRAMEFREEZE_CODE_BLOB); // And add the new code.

            // Now we need to pull out the original code and replace it with a call to our new
            // function.
            let original_code = &new_data[FRAMEFREEZE_CODE_DST];
            let original_md5 = md5::compute(original_code);
            assert_eq!(format!("{:x}", original_md5),
                        FRAMEFREEZE_ORIG_CODE_MD5,
                        "Checksum doesn't match for code to be replaced with frame-freeze in KIT EXE - you may have the wrong (version of the) {} file!", exe_name);
            // Apparently taking the MD5 moves the slice, so I have to take it again...
            let original_code = &mut new_data[FRAMEFREEZE_CODE_DST];
            original_code[0] = 0xE8; // Opcode for near call...
            // Near call takes an address relative to the *end* of the call instruction itself
            // (which is 5 bytes long - the opcode plus the 32-bit address).
            let next_eip = (FRAMEFREEZE_CODE_DST.start + 5) as u32;
            original_code[1..5].copy_from_slice(&(newfunc_address as u32 - next_eip).to_le_bytes());
            // Pad with nops.
            for i in 5..(original_code.len()) {
                original_code[i] = 0x90;
            }

            // If all that succeeds, we can return the new data vector...
            new_data
        })?;
    println!("Done!");

    // Deal with the colouring
    println!("");
    println!("Adding colouring code...");

    // The fun part: put the code into the destination (i.e. KIT) EXE!
    pmw1_exe.entry_object_mut()
        .update_data(|old_data| {
            // We're going to hang a new function off the end of the code segment.
            // This will be its address:
            let newfunc_address = old_data.len();

            let (positions, registers): (Vec<usize>, Vec<u8>) = old_data
                .windows(5)
                .enumerate()
                .filter(|(_i,window)| &window[0..2] == &[0x66,0x81] && &window[3..5] == &[0x10,0x01]) // cmp <r16>, 110h ; MS_pap
                .map(|(i,window)| (i, window[2]))
                .unzip();

            let mut new_data = old_data.to_vec(); // Turn it into a vector...
            new_data.extend_from_slice(&COLOUR_CODE_BLOB); // And add the new code.

            for (&pos,reg) in positions.iter().zip(registers.iter()) {
                // Depending on the register used to contain the object type, we want to call into different
                // offsets of the blob above, which execute different "xchg"s before and after the call to the main
                // function. The third byte of each "cmp" instruction specifies the register used, so this match
                // maps those bytes to the corresponding blob offsets.
                let blob_offset = match reg {
                    0xFB => 0x1D, // bx register --> xchg ax,bx
                    0xF9 => 0x27, // cx register --> xchg ax,cx
                    0xFA => 0x31, // dx register --> xchg ax,dx
                    0xFE => 0x3B, // si register --> xchg ax,si
                    0xFF => 0x45, // di register --> xchg ax,di
                    0xFD => 0x4F, // bp register --> xchg ax,bp
                    _ => {continue;} // This isn't the kind of instruction we're looking for…
                };

                new_data[pos] = 0xE8; // Opcode for near call...
                let func_offset = newfunc_address + blob_offset - (pos + 5); // +5 since the call instruction (5 bytes long) specifies an offset from the *end* of itself.
                new_data[pos+1..pos+5].copy_from_slice(&(func_offset as u32).to_le_bytes());
            }

            // If all that succeeds, we can return the new data vector...
            new_data
        })?;
    println!("Code added - synchronizing virtual size...");
    let new_size = pmw1_exe.entry_object().uncompressed_size() as u32;
    pmw1_exe.entry_object_mut().set_virtual_size(new_size);
    println!("Done!");

    // Write out the patched EXE.
    {
        println!("");
        println!("Attempting to write patched code back to {}", exe_name);
        let mut outfile = File::create(&exe_name)?;
        // Write the DOS stub back out
        outfile.write_all(&binary[..actual_mz_size])?;
        // And the actual PMW1 exe!
        outfile.write_all(&pmw1_exe.as_bytes())?;
        println!("Patching successful!");
    }

    Ok(())
}
