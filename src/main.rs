extern crate pmw1;
extern crate md5;

use std::io::prelude::*;
use std::fs::{File,OpenOptions};

use pmw1::exe::Pmw1Exe;

// To make this as easy-to-use as possible, don't use command line arguments.
// Just assume the two EXEs are in the working directory.
const SRC_EXE:&str = "RAYEDU.EXE";
const DST_EXE:&str = "RAYKIT.EXE";

// What code are we pulling out and where are we sticking it in?
const FRAMEFREEZE_CODE_SRC:std::ops::Range<usize> = 0x43302..0x43343;
const FRAMEFREEZE_CODE_DST:std::ops::Range<usize> = 0x537da..0x537e2;
const FRAMEFREEZE_CODE_MD5:&str = "f67896a6974c07046fc8dfbea8e13789";
const FRAMEFREEZE_ORIG_CODE_MD5:&str = "899302d008debf630bf09186f97132b0";

const COLOUR_CODE_SRC:std::ops::Range<usize> = 0x9def..0x9e12;
const COLOUR_CODE_DST:std::ops::Range<usize> = 0x94df..0x94e4;
const COLOUR_CODE_MD5:&str = "d03db3cd47749d1f1aa9fa3296a6909f";
const COLOUR_ORIG_CODE_MD5:&str = "ec1d918728e85ccd45a12c17038b40ec";

fn main() -> std::io::Result<()> {
    let exe_names = [SRC_EXE, DST_EXE];
    let binaries = exe_names.iter()
        .map(|&exe_name:&&str| -> std::io::Result<Vec<u8>> {
            // Load the whole EXE into memory...
            println!("Opening {}...", exe_name);

            let mut file = File::open(exe_name)?;
            let mut buffer: Vec<u8> = Vec::with_capacity(0x100000);
            file.read_to_end(&mut buffer)?;
            buffer.shrink_to_fit();
            Ok(buffer)
        }).collect::<std::io::Result<Vec<_>>>()?;

    // Create a backup file (only for the destination EXE).
    {
        let filename = format!("{}.BAK",DST_EXE);
        println!("");
        println!("Attempting to create NEW backup file {}", filename);
        // `create_new` to fail if the backup file already exists.
        // Don't wanna screw up an existing backup...
        let mut outfile = OpenOptions::new().write(true)
                                            .create_new(true)
                                            .open(&filename)?;
        // Write the whole binary back out
        outfile.write_all(&binaries[1])?;
        println!("Backup successful");
    }

    let mz_sizes = binaries.iter()
        .zip(exe_names.iter())
        .map(|(binary, &exe_name)| {
            println!("");
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

            Ok(actual_mz_size)
        }).collect::<std::io::Result<Vec<_>>>()?;

    let mut exes = binaries.iter()
        .zip(mz_sizes.iter())
        .map(|(binary, &actual_mz_size)| -> std::io::Result<Pmw1Exe> {
            // A slice containing just the PMW1 part.
            Pmw1Exe::from_bytes(&binary[actual_mz_size..])
        }).collect::<std::io::Result<Vec<_>>>()?;

    // Deal with the frame-freezing
    println!("");
    println!("Transferring frame-freezing-code...");

    // Get the relevant section from the source (i.e. EDU) EXE.
    let framefreeze_code = &exes[0]
        .entry_object()
        .data()?
        [FRAMEFREEZE_CODE_SRC];
    let framefreeze_md5 = md5::compute(framefreeze_code);
    assert_eq!(format!("{:x}", framefreeze_md5),
                FRAMEFREEZE_CODE_MD5,
                "Checksum doesn't match for frame-freeze code in EDU EXE - you may have the wrong (version of the) {} file!", SRC_EXE);

    // The fun part: put the code into the destination (i.e. KIT) EXE!
    exes[1].entry_object_mut()
        .update_data(|old_data| {
            // We're going to hang a new function off the end of the code segment.
            // This will be its address:
            let newfunc_address = old_data.len();

            let mut new_data = old_data.to_vec(); // Turn it into a vector...
            new_data.extend_from_slice(framefreeze_code); // Add the new code...
            new_data.extend_from_slice(&[0x66, 0x58, // A "pop ax" which I'll explain in a minute...
                                        0xC3]); // And a near return!

            // As it stands, our new function includes an instruction to move the value of the al
            // register into the object's "RuntimeCurrentAnimIndex" property. Not a problem as
            // such, but it then goes onto use the (e)ax register for other things. Meanwhile, the
            // KIT EXE sets the "RuntimeCurrentAnimIndex" property a bit later, using that same
            // register, which would wreak havoc if we allowed it to use the spoiled value from our
            // new function.
            //
            // The solution is to replace this first use of the al register in our new function
            // with a "push ax" to save the value, then pop it off at the end (already added
            // above).
            let use_of_al = &mut new_data[(newfunc_address+8)..(newfunc_address+11)];
            assert_eq!(use_of_al,
                       &[0x88, 0x41, 0x6E], // mov [ecx+6Eh], al
                       "Verification failure even though checksum was OK... You should never see this message!");
            use_of_al.copy_from_slice(&[0x66, 0x50, // push ax
                                      0x90]); // nop ;for padding

            // Now we need to pull out the original code and replace it with a call to our new
            // function.
            let original_code = &new_data[FRAMEFREEZE_CODE_DST];
            let original_md5 = md5::compute(original_code);
            assert_eq!(format!("{:x}", original_md5),
                        FRAMEFREEZE_ORIG_CODE_MD5,
                        "Checksum doesn't match for code to be replaced with frame-freeze in KIT EXE - you may have the wrong (version of the) {} file!", DST_EXE);
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
    println!("Transferring colouring code...");

    // Get the relevant section from the source (i.e. EDU) EXE.
    let colour_code = &exes[0]
        .entry_object()
        .data()?
        [COLOUR_CODE_SRC];
    let colour_md5 = md5::compute(colour_code);
    assert_eq!(format!("{:x}", colour_md5),
                COLOUR_CODE_MD5,
                "Checksum doesn't match for colouring code in EDU EXE - you may have the wrong (version of the) {} file!", SRC_EXE);

    // The fun part: put the code into the destination (i.e. KIT) EXE!
    exes[1].entry_object_mut()
        .update_data(|old_data| {
            // We're going to hang a new function off the end of the code segment.
            // This will be its address:
            let newfunc_address = old_data.len();

            let mut new_data = old_data.to_vec(); // Turn it into a vector...
            // EDU uses the cx register here, whereas KIT uses si, so we need to add an "xchg cx,
            // si" to the beginning and end of our new function to make it work correctly...
            new_data.extend_from_slice(&[0x66, 0x87, 0xF1]);
            new_data.extend_from_slice(colour_code); // Add the new code...

            // As it stands, our new function ends with a "jnz" that jumps off somewhere into
            // space. We don't want to do that, we just want to return with the zero flag set as
            // appropriate. To fix this, we just replace the jnz with nops.
            let jnz_addr = new_data.len()-2;
            let jnz_op = &mut new_data[jnz_addr..];
            assert_eq!(jnz_op,
                       &[0x75, 0x6B], // jnz short $+6Dh
                       "Verification failure even though checksum was OK... You should never see this message!");
            jnz_op.copy_from_slice(&[0x90, 0x90]); // nop nop

            // Finish off the function with another "xchg cx, si"...
            new_data.extend_from_slice(&[0x66, 0x87, 0xF1,
                                        0xC3]); // And a near return!
            // Neither of these opcodes messes with the zero flag, so the function should leave it
            // in exactly the condition we need it to be in.

            // Now we need to pull out the original code and replace it with a call to our new
            // function.
            let original_code = &new_data[COLOUR_CODE_DST];
            let original_md5 = md5::compute(original_code);
            assert_eq!(format!("{:x}", original_md5),
                        COLOUR_ORIG_CODE_MD5,
                        "Checksum doesn't match for code to be replaced with extra colouring checks in KIT EXE - you may have the wrong (version of the) {} file!", DST_EXE);
            // Apparently taking the MD5 moves the slice, so I have to take it again...
            let original_code = &mut new_data[COLOUR_CODE_DST];
            original_code[0] = 0xE8; // Opcode for near call...
            // Near call takes an address relative to the *end* of the call instruction itself
            // (which is 5 bytes long - the opcode plus the 32-bit address).
            let next_eip = COLOUR_CODE_DST.end as u32;
            original_code[1..5].copy_from_slice(&(newfunc_address as u32 - next_eip).to_le_bytes());

            // If all that succeeds, we can return the new data vector...
            new_data
        })?;
    println!("Code added - synchronizing virtual size...");
    let new_size = exes[1].entry_object().uncompressed_size() as u32;
    exes[1].entry_object_mut().set_virtual_size(new_size);
    println!("Done!");

    // Write out the patched EXE.
    {
        println!("");
        println!("Attempting to write patched code back to {}", DST_EXE);
        let mut outfile = File::create(&DST_EXE)?;
        // Write the DOS stub back out
        outfile.write_all(&binaries[1][..mz_sizes[1]])?;
        // And the actual PMW1 exe!
        outfile.write_all(&exes[1].as_bytes())?;
        println!("Patching successful!");
    }

    Ok(())
}
