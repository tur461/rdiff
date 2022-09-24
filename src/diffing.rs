#![allow(dead_code)]
#![allow(unused_mut)]

use std::fs::File;
use std::io::Read;
use xxh3::hash128_with_seed;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Chunk {
    end: usize,
    start: usize,
    is_present: bool,
    problem_bytes: Vec<u8>,
}

impl Chunk {
    fn new(
        idx: usize, 
        bytes: Vec<u8>, 
        c_size: usize, 
        is_present: bool
    ) -> Self {
        Self {
            is_present,
            start: idx * c_size,
            problem_bytes: bytes,
            end: (idx * c_size) + c_size,
        }
    }
}

type HashList = Vec<u128>;
type DeltaMap = HashMap<usize, Chunk>;
type PositionMap<'a> = HashMap<&'a u128, usize>;

#[derive(Debug)]
pub struct Common { 
    file1_size: usize,
    file2_size: usize,
    chunk_size: usize,
    file1_data: Vec<u8>,
    file2_data: Vec<u8>,
}

impl Common {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            file1_size: 0,
            file2_size: 0,
            file1_data: Vec::new(),
            file2_data: Vec::new(),
        }
    }

    // function to find diff between the files
    pub fn diff(
        &mut self, 
        hash_list: &HashList, 
        path: &str
    ) -> Option<(DeltaMap)> {
        let file = File::open(path);

        if let Err(e) = file {
            println!("\nFile/Path {}: {}\n", path, std::io::Error::last_os_error());  
            return None;
        }
        
        let mut file = file.unwrap();
        
        let m_dat = file.metadata().ok()?;
        let f_size = m_dat.len() as usize;
        if !self.has_at_least_2_chunks(f_size) {
            println!("\nFile {} must contain at least 2 chunks!!\n", path);
            return None;
        }
        self.file2_size = f_size;
        // trick to read char by char is here! (u8 buf of size 1)
        let mut buffer = [0u8; 1];
        let mut delta_map = DeltaMap::new();
        let mut problem_bytes = Vec::<u8>::new();
        let mut data_buf = Vec::with_capacity(self.chunk_size);
        let mut position_map = Self::hash_list_to_position_map(hash_list);
        
        // iterate over file char by char
        loop {
            let read_count = file.read(&mut buffer).ok()?;
            // when there is nothing to read
            if read_count != 1 { break; }
            // push each char into buffer
            data_buf.push(buffer[0]);
            self.file2_data.push(buffer[0]);
            // continue until data_buf is of chunk_size 
            if(data_buf.len() < self.chunk_size) { continue; }

            // if data_buf didn't got cleared on previous iteration
            // because there, was no match then this condition 
            // will become true!
            if data_buf.len() == self.chunk_size + 1 {
                // push the first byte into problem_bytes
                problem_bytes.push(data_buf[0]);
                // skip first byte from buffer to become equal to chunk_size
                data_buf = data_buf.into_iter().skip(1).collect::<Vec<u8>>();
            }

            // try get position of chunk by hash
            let res = position_map.get(
                &hash128_with_seed(
                    &data_buf[..], 
                    0u64
                )
            );
            // if there is position, means this chunk is not mis-match
            if let Some(p) = res {
                let pos = p.clone();
                delta_map.insert(
                    pos, 
                    Chunk::new(
                        pos.clone(),
                        // save also problem_bytes related to this chunk
                        // collected by previous iterations
                        problem_bytes.clone(),
                        self.chunk_size, 
                        true
                    )
                );
                // clear buffers to start anew
                data_buf.drain(..);
                problem_bytes.drain(..);
            }
        }
        self.try_fill_missing_chunks(
            &hash_list, 
            &mut delta_map
        );
        self.patch(&delta_map);
        Some(delta_map)
    }

    pub fn patch(&self, deltas: &DeltaMap) {
        let num_of_chunks = self.calculate_num_of_chunks() as usize;
        let mut end: usize;
        let f_size = self.file2_size;
        let mut pb_count = 0;
        for i in 0..num_of_chunks {
            if let Some(chunk) = deltas.get(&i) {
                // println!("{:#?}", chunk);
                end = chunk.end;
                if end > f_size {
                    end = f_size;
                }
                if chunk.problem_bytes.len() == 0 {
                    
                    // print!("{}", String::from_utf8_lossy(&self.file1_data[chunk.start..end]));
                } else {
                    pb_count += 1;
                    // println!("{:#?}", chunk);
                    // let with_patch = &self.file2_data[chunk.start..end];
                    // let without_patch = Vec::<u8>::new();
                    // let start = chunk.start + chunk.problem_bytes.len();

                    println!("\n+{}+", String::from_utf8_lossy(&chunk.problem_bytes));
                    println!("\n-{}-", String::from_utf8_lossy(&self.file2_data[chunk.start..end]));
                }
            }
        }
        if pb_count > 0 {
            // println!("\n{} patches applied!.", pb_count);
        } else {
            println!("\nNo change detected!.");
        }
    }

    // function to fill chunks we missed during diff operation
    pub fn try_fill_missing_chunks(
        &self, 
        hash_list: &HashList, 
        deltas: &mut DeltaMap
    ) {
        for i in 0..hash_list.len() {
            if deltas.get(&i).is_none() {
                deltas.insert(
                    i,
                    Chunk::new(
                        i, 
                        Vec::new(), 
                        self.chunk_size, 
                        false
                    )
                );
            }
        }
    }
    
    pub fn hash_list_to_position_map<'a>(
        hash_list: &'a HashList
    ) -> PositionMap {
        let mut position_map: HashMap<&u128, usize> = HashMap::new();
        for (i, hash) in hash_list.into_iter().enumerate() {
            position_map.insert(&hash, i);
        }
        position_map
    }

    pub fn file_to_chunk_hash_list<'a>(
        &mut self, 
        path: &'a str
    ) -> Option<Vec<u128>> {
        let mut file = File::open(path);

        if let Err(e) = file {
            println!("\nFile/Path {}: {}\n", path, std::io::Error::last_os_error());  
            return None;
        }

        let mut file = file.unwrap();
        
        let m_dat = file.metadata().ok()?;
        let f_size = m_dat.len() as usize;
        if !self.has_at_least_2_chunks(f_size) {
            println!("\nFile {} must contain at least 2 chunks!!\n", path);
            return None;
        }
        self.file1_size = f_size;

        let mut buffer = [0u8; 1];
        let mut hash_list: Vec<u128> = Vec::new();
        let mut data_buf = Vec::with_capacity(self.chunk_size);
        loop {
            let read_count = file.read(&mut buffer).ok()?;
            
            if read_count != 1 {
                hash_list.push(hash128_with_seed(&data_buf[..], 0u64));
                break;
            }

            data_buf.push(buffer[0]);
            self.file1_data.push(buffer[0]);
            // println!("len: {}", data_buf.len());
            if(data_buf.len() == self.chunk_size) {
                hash_list.push(hash128_with_seed(&data_buf[..], 0u64));
                data_buf.drain(..);
            }
        }
        Some(hash_list)
    }

    fn calculate_num_of_chunks(&self) -> f64 {
        (self.file2_size as f64 / self.chunk_size as f64).ceil()
    }

    fn has_at_least_2_chunks(&self, f_size: usize) -> bool {
        self.chunk_size <= f_size - 1 
    }
}








