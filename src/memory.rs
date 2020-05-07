const PROGRAM_ADDRESS: usize = 0;

#[derive(Debug)]
pub struct Memory {
    pool: Vec<usize>,
    heap: Vec<Vec<u32>>,
}

impl Memory {
    // create a new Memory, comprising a pool of reusable IDs
    // and a heap of UM words, populated with the instructions
    // as segment 0
    pub fn new(instructions: Vec<u32>) -> Memory {
        Memory { pool: vec![], heap: vec![instructions] }
    }

    // allocate and initalize (as all 0s) a memory segment.
    // returns the segment ID
    pub fn allocate(&mut self, size: usize) -> usize {
        // can we reuse a previously unmapped segment id?
        match self.pool.pop() {
            None => {
                self.heap.push(vec![0; size]);
                self.heap.len() - 1
            }
            Some(address) => {
                assert!(address < self.heap.len(), "invalid address in pool");
                self.heap[address].resize(size, 0);
                address
            }
        }
    }

    // deallocate the memory at the given address.
    pub fn deallocate(&mut self, address: usize) {
        assert!(
            address < self.heap.len(),
            "invalid address {}, cannot deallocate",
            address,
        );
        self.pool.push(address);
        self.heap[address].clear();
    }

    // supply contents of the memory at the given address if
    // initialized, panics otherwise.
    pub fn load(&self, seg_id: usize, address: usize) -> u32 {
        self.heap[seg_id][address]
    }

    // get the instruction word corresponding to the given program counter
    // if it doesn't exist, then this panics
    // This may have high overhead...
    pub fn get_instruction(&self, pc: usize) -> u32 {
        // SAFETY: `heap` always has length at least 1 and PROGRAM_ADDRESS
        // is always == 0. This improves performance by about 10%.
        unsafe { self.heap.get_unchecked(PROGRAM_ADDRESS)[pc] }
    }

    // write a value into the given address of the given segment.
    pub fn store(&mut self, seg_id: usize, address: usize, value: u32) {
        let memory =
            self.heap.get_mut(seg_id).expect("Memory was unallocated");
        assert!(
            address < memory.len(),
            "invalid address {} for segment {}",
            address,
            seg_id
        );
        memory[address] = value;
    }

    // replace the program with the vector at the given address
    pub fn load_segment(&mut self, seg_id: usize) {
        let program = self
            .heap
            .get(seg_id)
            .expect("Found no program at the given address")
            .clone();
        // Never panics, PROGRAM_ADDRESS is to be present by construction.
        // (And because `heap` never shrinks.)
        self.heap[PROGRAM_ADDRESS] = program;
    }
}
