use rvemu::{bus::DRAM_BASE, dram::DRAM_SIZE, emulator::Emulator};

pub fn setup_from_elf(elf_data: &[u8]) -> Emulator {
    let elf = goblin::elf::Elf::parse(elf_data)
        .map_err(|_| "Failed to parse ELF")
        .unwrap();

    let mem = load_mem(&elf, elf_data);

    let mut emu = Emulator::new();

    emu.initialize_dram(mem);
    emu.initialize_pc(elf.header.e_entry as u64);

    emu
}

fn load_mem(elf: &goblin::elf::Elf, elf_data: &[u8]) -> Vec<u8> {
    let mut mem = Vec::new();
    for program_header in &elf.program_headers {
        if program_header.p_type == goblin::elf::program_header::PT_LOAD {
            let start_data = program_header.p_offset as usize;
            let end_data = start_data + program_header.p_filesz as usize;
            let virtual_address = program_header.p_vaddr;
            let virtual_end = virtual_address + program_header.p_memsz;

            // The interpreter RAM is DRAM_SIZE starting at DRAM_BASE
            assert!(virtual_address >= DRAM_BASE);
            assert!(virtual_end <= DRAM_BASE + DRAM_SIZE);

            let start_vec = (virtual_address - DRAM_BASE) as usize;
            let end_vec = (virtual_end - DRAM_BASE) as usize;

            if mem.len() < end_vec {
                mem.resize(end_vec, 0);
            }

            mem[start_vec..end_vec].copy_from_slice(&elf_data[start_data..end_data]);
        }
    }
    mem
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_execute_elf() {
        let elf_data = fs::read("../asm-runtime-example/runtime").unwrap();
        let result = setup_from_elf(&elf_data).start();
        println!("Result: {result:#?}");
    }
}
