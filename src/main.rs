extern crate pmw1;
extern crate md5;

use std::env::args;
use std::io::prelude::*;
use std::fs::{File,OpenOptions};

use pmw1::exe::Pmw1Exe;
use pmw1::reloc::{Pmw1RelocBlock,Pmw1RelocEntry};

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

const FRAMESCROLL_CODE_DST:std::ops::Range<usize> = 0x2729e..0x272a3;
// So short, there's really no point in MD5ing it...
const FRAMESCROLL_ORIG_CODE: [u8; 5] = [0x66, 0x81, 0xFA, 0x10, 0x01];
// This address appears in the following code and will need to be fixed up!
const NUM_SAMPLES_PTR: u32 = 0xBA88;

// Assembled from framescrollcode.asm using UASM (https://github.com/Terraspace/UASM)
// and formatted using hx (https://github.com/sitkevij/hex).
const FRAMESCROLL_CODE_BLOB: [u8; 204] = [
    0x66, 0x81, 0xfa, 0x03, 0x01, 0x74, 0x15, 0x66, 0x81, 0xfa, 
    0x04, 0x01, 0x74, 0x0e, 0x66, 0x81, 0xfa, 0x12, 0x01, 0x74, 
    0x07, 0x66, 0x81, 0xfa, 0x06, 0x01, 0x75, 0x4c, 0x80, 0xfb, 
    0x4b, 0x75, 0x17, 0x0f, 0xb6, 0x41, 0x6e, 0x6b, 0xc0, 0x0c, 
    0x03, 0x41, 0x04, 0x0f, 0xb7, 0x70, 0x0a, 0x0f, 0xb6, 0x41, 
    0x6f, 0x03, 0xc6, 0x48, 0xeb, 0x18, 0x80, 0xfb, 0x4d, 0x75, 
    0x2b, 0x0f, 0xb6, 0x41, 0x6e, 0x6b, 0xc0, 0x0c, 0x03, 0x41, 
    0x04, 0x0f, 0xb7, 0x70, 0x0a, 0x0f, 0xb6, 0x41, 0x6f, 0x40, 
    0x8b, 0xfa, 0x33, 0xd2, 0xf7, 0xfe, 0x88, 0x51, 0x6f, 0x66, 
    0x81, 0xff, 0x06, 0x01, 0x75, 0x64, 0x88, 0x51, 0x7a, 0x88, 
    0x51, 0x7b, 0xeb, 0x5c, 0x66, 0x81, 0xfa, 0x07, 0x01, 0x75, 
    0x57, 0x0f, 0xb7, 0x41, 0x50, 0x80, 0xfb, 0x4b, 0x75, 0x0b, 
    0x66, 0x03, 0x05, 0x88, 0xba, 0x00, 0x00, 0x66, 0x48, 0xeb, 
    0x30, 0x80, 0xfb, 0x4d, 0x75, 0x04, 0x66, 0x40, 0xeb, 0x27, 
    0x66, 0x83, 0x3d, 0x88, 0xba, 0x00, 0x00, 0x32, 0x7e, 0x30, 
    0x80, 0xfb, 0x50, 0x75, 0x0d, 0x66, 0x03, 0x05, 0x88, 0xba, 
    0x00, 0x00, 0x66, 0x83, 0xe8, 0x32, 0xeb, 0x0b, 0x80, 0xfb, 
    0x48, 0x75, 0x19, 0x66, 0x83, 0xc0, 0x32, 0xeb, 0x00, 0x0f, 
    0xb7, 0x35, 0x88, 0xba, 0x00, 0x00, 0x8b, 0xfa, 0x33, 0xd2, 
    0xf7, 0xfe, 0x66, 0x89, 0x51, 0x50, 0x8b, 0xd7, 0x66, 0x81, 
    0xfa, 0x10, 0x01, 0xc3
];

const SAMPLENOTATION_CODE_DST:std::ops::Range<usize> = 0x2642b..0x26437;
const SAMPLENOTATION_ORIG_CODE_MD5:&str = "1c86cf29f168723e20237fad14571202";

// Assembled from samplenotationcode.asm using UASM (https://github.com/Terraspace/UASM)
// and formatted using hx (https://github.com/sitkevij/hex).
const SAMPLENOTATION_CODE_BLOB: [u8; 32] = [
    0x66, 0x81, 0xfa, 0x07, 0x01, 0x74, 0x05, 0x66, 0x81, 0xfa, 
    0x0a, 0x01, 0xc3, 0x33, 0xc0, 0x66, 0x81, 0xfa, 0x07, 0x01, 
    0x75, 0x06, 0x66, 0x8b, 0x43, 0x50, 0xeb, 0x03, 0x8a, 0x43, 
    0x7b, 0xc3
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

const FRAMESAVE_CODE_DST:std::ops::Range<usize> = 0x21f73..0x21f78;

// Assembled from framesavecode.asm using UASM (https://github.com/Terraspace/UASM)
// and formatted using hx (https://github.com/sitkevij/hex).
const FRAMESAVE_CODE_BLOB: [u8; 76] = [
    0x66, 0x81, 0xfa, 0x03, 0x01, 0x74, 0x29, 0x66, 0x81, 0xfa, 
    0x04, 0x01, 0x74, 0x22, 0x66, 0x81, 0xfa, 0x12, 0x01, 0x74, 
    0x1b, 0x66, 0x81, 0xfa, 0x05, 0x01, 0x74, 0x13, 0x66, 0x81, 
    0xfa, 0x06, 0x01, 0x74, 0x0c, 0x66, 0x81, 0xfa, 0x07, 0x01, 
    0x74, 0x17, 0x66, 0x81, 0xfa, 0x10, 0x01, 0xc3, 0x8a, 0x4b, 
    0x6f, 0x32, 0xed, 0x66, 0x6b, 0xc9, 0x06, 0x02, 0x4b, 0x7b, 
    0x88, 0x48, 0x17, 0xeb, 0x08, 0x66, 0x8b, 0x4b, 0x50, 0x66, 
    0x89, 0x48, 0x18, 0x85, 0xc0, 0xc3
];

const FRAMELOAD_CODE_DST:std::ops::Range<usize> = 0x389db..0x38a22;
const FRAMELOAD_ORIG_CODE_MD5:&str = "6016a0d7577299738c3d470bf8421b91";

// Assembled from frameloadcode.asm using UASM (https://github.com/Terraspace/UASM)
// and formatted using hx (https://github.com/sitkevij/hex).
const FRAMELOAD_CODE_BLOB: [u8; 86] = [
    0x88, 0x44, 0x3a, 0x7a, 0x66, 0x89, 0x44, 0x3a, 0x50, 0x8b, 
    0xd1, 0xc3, 0x66, 0x81, 0xfa, 0x03, 0x01, 0x74, 0x29, 0x66, 
    0x81, 0xfa, 0x04, 0x01, 0x74, 0x22, 0x66, 0x81, 0xfa, 0x12, 
    0x01, 0x74, 0x1b, 0x66, 0x81, 0xfa, 0x05, 0x01, 0x74, 0x0c, 
    0x66, 0x81, 0xfa, 0x06, 0x01, 0x74, 0x06, 0x66, 0x81, 0xfa, 
    0x10, 0x01, 0xc3, 0x8a, 0x50, 0x7a, 0x88, 0x50, 0x6f, 0xc3, 
    0x8b, 0xd0, 0x66, 0x0f, 0xb6, 0x42, 0x7a, 0x66, 0x51, 0xb1, 
    0x06, 0xf6, 0xf9, 0x66, 0x59, 0x88, 0x62, 0x7a, 0x88, 0x42, 
    0x6f, 0x8b, 0xc2, 0x33, 0xd2, 0xc3
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
    println!("Adding frame-freezing code...");

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

    // Deal with the frame-scrolling
    println!("");
    println!("Adding frame-scrolling code...");

    // Looks like we need to create a new fix-up block for this since it refers to a particular
    // location in the data segment (word ptr num_samples).
    let cur_len = pmw1_exe.entry_object_mut().uncompressed_size() as u32;
    let new_fixups = Pmw1RelocBlock::new(
        &mut FRAMESCROLL_CODE_BLOB
        .windows(4)
        .enumerate()
        .filter(|(_i,window)| window == &NUM_SAMPLES_PTR.to_le_bytes())
        .map(|(i,_window)| Pmw1RelocEntry::new(
                7, // Relocation type – always 7 it seems…
                2, // Target object – the data section
                cur_len + (i as u32), // The source location
                NUM_SAMPLES_PTR) // The target pointer
            )
        );
    pmw1_exe.entry_object_mut().add_reloc_block(new_fixups)?;

    // The fun part: put the code into the destination (i.e. KIT) EXE!
    pmw1_exe.entry_object_mut()
        .update_data(|old_data| {
            // We're going to hang a new function off the end of the code segment.
            // This will be its address:
            let newfunc_address = old_data.len();

            let mut new_data = old_data.to_vec(); // Turn it into a vector...
            new_data.extend_from_slice(&FRAMESCROLL_CODE_BLOB); // And add the new code.

            // Now we need to pull out the original code and replace it with a call to our new
            // function.
            let original_code = &mut new_data[FRAMESCROLL_CODE_DST];
            assert_eq!(original_code,
                        &FRAMESCROLL_ORIG_CODE,
                        "Code to be replaced with frame-scroll in KIT EXE is not as expected - you may have the wrong (version of the) {} file!", exe_name);
            original_code[0] = 0xE8; // Opcode for near call...
            // Near call takes an address relative to the *end* of the call instruction itself
            // (which is 5 bytes long - the opcode plus the 32-bit address).
            let next_eip = (FRAMESCROLL_CODE_DST.start + 5) as u32;
            original_code[1..5].copy_from_slice(&(newfunc_address as u32 - next_eip).to_le_bytes());
            /* // Pad with nops.
            for i in 5..(original_code.len()) {
                original_code[i] = 0x90;
            } */

            // If all that succeeds, we can return the new data vector...
            new_data
        })?;
    println!("Done!");

    // Add some way of tracking which sample is used in the editor.
    // It'll say "color", which isn't ideal, but better than nothing!
    println!("");
    println!("Adding sample-notation code...");

    // The fun part: put the code into the destination (i.e. KIT) EXE!
    pmw1_exe.entry_object_mut()
        .update_data(|old_data| {
            // We're going to hang a new function off the end of the code segment.
            // This will be its address:
            let newfunc_address = old_data.len();

            let mut new_data = old_data.to_vec(); // Turn it into a vector...
            new_data.extend_from_slice(&SAMPLENOTATION_CODE_BLOB); // And add the new code.

            // Now we need to pull out the original code and replace it with a call to our new
            // function.
            let original_code = &new_data[SAMPLENOTATION_CODE_DST];
            let original_md5 = md5::compute(original_code);
            assert_eq!(format!("{:x}", original_md5),
                        SAMPLENOTATION_ORIG_CODE_MD5,
                        "Checksum doesn't match for code to be replaced with frame-freeze in KIT EXE - you may have the wrong (version of the) {} file!", exe_name);
            // Apparently taking the MD5 moves the slice, so I have to take it again...
            let original_code = &mut new_data[SAMPLENOTATION_CODE_DST];
            original_code[0] = 0xE8; // Opcode for near call...
            // Near call takes an address relative to the *end* of the call instruction itself
            // (which is 5 bytes long - the opcode plus the 32-bit address).
            let next_eip = (SAMPLENOTATION_CODE_DST.start + 5) as u32;
            original_code[1..5].copy_from_slice(&(newfunc_address as u32 - next_eip).to_le_bytes());

            // The next thing is a jnz (2 bytes), which we leave intact...
            //
            // Then there are two instructions that happen to add up to five bytes, for loading the
            // hitpoints into the EAX register.
            original_code[7] = 0xE8; // Near call again.
            let next_eip = SAMPLENOTATION_CODE_DST.end as u32;
            let newfunc_address = newfunc_address + 13; // Offset for the second function in this blob (for loading hitpoints).
            original_code[8..12].copy_from_slice(&(newfunc_address as u32 - next_eip).to_le_bytes());

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
    println!("Done!");

    // Deal with the frame-saving
    println!("");
    println!("Adding frame-saving code...");

    // The fun part: put the code into the destination (i.e. KIT) EXE!
    pmw1_exe.entry_object_mut()
        .update_data(|old_data| {
            // We're going to hang a new function off the end of the code segment.
            // This will be its address:
            let newfunc_address = old_data.len();

            let mut new_data = old_data.to_vec(); // Turn it into a vector...
            new_data.extend_from_slice(&FRAMESAVE_CODE_BLOB); // And add the new code.

            // Now we need to pull out the original code and replace it with a call to our new
            // function.
            let original_code = &mut new_data[FRAMESAVE_CODE_DST];
            // This should already be a near call since we replaced it just above, for the
            // colouring code.
            assert_eq!(original_code[0], 0xE8,
                    "Near call not found at expected address for frame saving");
            // Near call takes an address relative to the *end* of the call instruction itself
            // (which is 5 bytes long - the opcode plus the 32-bit address).
            let next_eip = (FRAMESAVE_CODE_DST.start + 5) as u32;
            original_code[1..5].copy_from_slice(&(newfunc_address as u32 - next_eip).to_le_bytes());

            // If all that succeeds, we can return the new data vector...
            new_data
        })?;
    println!("Done!");

    // Deal with the frame-loading
    println!("");
    println!("Adding frame-loading code...");

    // The fun part: put the code into the destination (i.e. KIT) EXE!
    pmw1_exe.entry_object_mut()
        .update_data(|old_data| {
            // We're going to hang a new function off the end of the code segment.
            // This will be its address:
            let newfunc_address = old_data.len();

            let mut new_data = old_data.to_vec(); // Turn it into a vector...
            new_data.extend_from_slice(&FRAMELOAD_CODE_BLOB); // And add the new code.

            // Now we need to pull out the original code and replace it with a call to our new
            // function.
            let original_code = &new_data[FRAMELOAD_CODE_DST];
            // Leave out the last four bytes, since they've already been modified by this program!
            // Technically it's the last five bytes, but the first of those will always be the
            // opcode for a near call.
            let fourth_last = original_code.len()-4;
            let original_md5 = md5::compute(&original_code[..fourth_last]);
            assert_eq!(format!("{:x}", original_md5),
                        FRAMELOAD_ORIG_CODE_MD5,
                        "Checksum doesn't match for code to be replaced with frame-load in KIT EXE - you may have the wrong (version of the) {} file!", exe_name);
            // Apparently taking the MD5 moves the slice, so I have to take it again...
            let original_code = &mut new_data[FRAMELOAD_CODE_DST];
            original_code[0] = 0xE8; // Opcode for near call...
            // Near call takes an address relative to the *end* of the call instruction itself
            // (which is 5 bytes long - the opcode plus the 32-bit address).
            let next_eip = (FRAMELOAD_CODE_DST.start + 5) as u32;
            original_code[1..5].copy_from_slice(&(newfunc_address as u32 - next_eip).to_le_bytes());
            original_code[5] = 0x90; // nop out the second byte of the following instruction, since we clobbered the first byte.

            let next_eip = FRAMELOAD_CODE_DST.end as u32;
            let newfunc_address = newfunc_address + 12; // Offset for the second function in this blob.
            original_code[fourth_last..].copy_from_slice(&(newfunc_address as u32 - next_eip).to_le_bytes());

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
